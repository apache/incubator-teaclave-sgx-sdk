// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

use crate::arch::{self, Tcs};
use crate::edmm;
use crate::enclave::state::{self, State};
use crate::error;
use crate::tcs::tc::{self, ThreadControl};
use crate::trts;
use crate::veh::list;
use crate::veh::MAX_REGISTER_COUNT;
use crate::veh::{ExceptionHandler, ExceptionInfo, ExceptionType, ExceptionVector, HandleResult};
use core::convert::TryFrom;
use core::mem;
use sgx_types::error::{SgxResult, SgxStatus};

macro_rules! try_error {
    ($cond:expr) => {
        if $cond {
            state::set_state(State::Crashed);
            bail!(SgxStatus::EnclaveCrashed);
        }
    };
}

pub fn handle(tcs: &mut Tcs) -> SgxResult {
    let mut tc = ThreadControl::from_tcs(tcs);
    try_error!(!tc.is_initialized());
    try_error!(!tc::check_static_stack_guard(tcs));

    try_error!(state::get_state() != State::InitDone);
    try_error!(tc != tc::current());

    let tds = tc.tds_mut();

    // check if the exception is raised from 2nd phrase
    try_error!(tds.exception_flag == -1);
    try_error!(((tds.first_ssa_gpr & (!0xFFF)) - arch::SE_PAGE_SIZE) != tcs as *const _ as usize);

    // no need to check the result of ssa_gpr because thread_data is always trusted
    let mut sp = {
        let ssa_gpr = tds.ssa_gpr();
        let sp_u = ssa_gpr.rsp_u as usize;
        let sp = ssa_gpr.rsp as usize;

        try_error!(!trts::is_within_host(
            sp_u as *const u8,
            mem::size_of::<u64>()
        ));
        try_error!(sp_u == sp);
        sp
    };

    // check stack overrun only, alignment will be checked after exception handled
    try_error!(!tds.is_stack_addr(sp, 0));

    let mut size = 0_usize;
    // x86_64 requires a 128-bytes red zone, which begins directly
    // after the return addr and includes func's arguments
    size += arch::RED_ZONE_SIZE;

    // decrease the stack to give space for info
    size += mem::size_of::<ExceptionInfo>();

    sp -= size;
    sp &= !0xF;

    // check the decreased sp to make sure it is in the trusted stack range
    try_error!(!tds.is_stack_addr(sp, 0));

    let info = unsafe { &mut *(sp as *mut ExceptionInfo) };

    // decrease the stack to save the SSA[0]->ip
    size = mem::size_of::<usize>();
    sp -= size;
    try_error!(!tds.is_stack_addr(sp, size));

    if sp < tds.stack_commit {
        // EDMM:
        // stack expand
        let mut result = SgxResult::<()>::Err(SgxStatus::StackOverRun);
        let page_aligned_delta = round_to_page!(tds.stack_commit - sp);

        // try to allocate memory dynamically
        if (tds.stack_commit > page_aligned_delta)
            && ((tds.stack_commit - page_aligned_delta) >= tds.stack_limit)
        {
            result = edmm::mem::expand_stack_epc_pages(
                tds.stack_commit - page_aligned_delta,
                page_aligned_delta >> arch::SE_PAGE_SHIFT,
            )
        }
        if result.is_ok() {
            tds.stack_commit -= page_aligned_delta;
        } else {
            state::set_state(State::Crashed);
        }
        return result;
    }

    let ssa_gpr = tds.ssa_gpr();

    #[cfg(all(not(feature = "sim"), not(feature = "hyper")))]
    unsafe {
        use crate::arch::Enclu;
        use crate::inst;
        extern "C" {
            // static Lereport_inst: u8;
            static Leverifyreport2_inst: u8;
        }
        // if (&Lereport_inst as *const _ as u64 == ssa_gpr.rip)  && (ssa_gpr.rax == Enclu::EReport as u64) {
        //     // Handle the exception raised by EREPORT instruction
        //     // Skip ENCLU, which is always a 3-byte instruction
        //     ssa_gpr.rip += 3;
        //     // Set CF to indicate error condition, see implementation of ereport()
        //     ssa_gpr.rflags |= 1;
        //     return Ok(());
        // }
        if (&Leverifyreport2_inst as *const _ as u64 == ssa_gpr.rip)
            && (ssa_gpr.rax == Enclu::EVerifyReport2 as u64)
        {
            // Handle the exception raised by everifyreport2 instruction
            // Skip ENCLU, which is always a 3-byte instruction
            ssa_gpr.rip += 3;
            // Set ZF to indicate error condition, see implementation of everify_report2()
            ssa_gpr.rflags |= 64;
            ssa_gpr.rax = inst::INVALID_LEAF as u64;
            return Ok(());
        }
    }

    // exception handlers are not allowed to call in a non-exception state
    try_error!(ssa_gpr.exit_info.valid() != 1);

    // initialize the info with SSA[0]
    let vector = ExceptionVector::try_from(ssa_gpr.exit_info.vector());
    let exception_type = ExceptionType::try_from(ssa_gpr.exit_info.exit_type());
    try_error!(vector.is_err() || exception_type.is_err());
    info.vector = vector.unwrap();
    info.exception_type = exception_type.unwrap();

    info.context.rax = ssa_gpr.rax;
    info.context.rcx = ssa_gpr.rcx;
    info.context.rdx = ssa_gpr.rdx;
    info.context.rbx = ssa_gpr.rbx;
    info.context.rsp = ssa_gpr.rsp;
    info.context.rbp = ssa_gpr.rbp;
    info.context.rsi = ssa_gpr.rsi;
    info.context.rdi = ssa_gpr.rdi;
    info.context.rflags = ssa_gpr.rflags;
    info.context.rip = ssa_gpr.rip;
    info.context.r8 = ssa_gpr.r8;
    info.context.r9 = ssa_gpr.r9;
    info.context.r10 = ssa_gpr.r10;
    info.context.r11 = ssa_gpr.r11;
    info.context.r12 = ssa_gpr.r12;
    info.context.r13 = ssa_gpr.r13;
    info.context.r14 = ssa_gpr.r14;
    info.context.r15 = ssa_gpr.r15;

    let new_sp = sp as *mut u64;
    // prepare the ip for 2nd phrase handling
    ssa_gpr.rip = internal_handle as usize as u64;
    // new stack for internal_handle_exception
    ssa_gpr.rsp = new_sp as u64;
    // 1st parameter (info) for LINUX32
    ssa_gpr.rax = info as *mut _ as u64;
    // 1st parameter (info) for LINUX64, LINUX32 also uses it while restoring the context
    ssa_gpr.rdi = info as *mut _ as u64;
    unsafe {
        // for debugger to get call trace
        *new_sp = info.context.rip;
    }

    // mark valid to 0 to prevent eenter again
    ssa_gpr.exit_info.set_valid(0);

    Ok(())
}

macro_rules! try_abort {
    ($cond:expr, $tds:ident) => {
        if $cond {
            $tds.exception_flag = -1;
            error::abort();
        }
    };
}

macro_rules! abort {
    ($tds:ident) => {
        $tds.exception_flag = -1;
        error::abort();
    };
}

extern "C" fn internal_handle(info: &mut ExceptionInfo) {
    extern "C" {
        fn continue_execution(info: *mut ExceptionInfo);
    }

    let mut tc = tc::current();
    let tds = tc.tds_mut();

    try_abort!(tds.exception_flag < 0, tds);
    tds.exception_flag += 1;

    let (handlers, len) = {
        let list_guard = list::EXCEPTION_LIST.lock();
        if list_guard.len() == 0 {
            drop(list_guard);
            tds.exception_flag = -1;
            unsafe {
                continue_execution(info);
            }
            // Should not come here
            error::abort();
        }

        let mut handlers: [ExceptionHandler; MAX_REGISTER_COUNT] = unsafe { mem::zeroed() };
        let mut len = 0_usize;
        for (i, f) in list_guard.iter().enumerate().take(MAX_REGISTER_COUNT) {
            handlers[i] = f;
            len += 1;
        }
        (handlers, len)
    };

    tds.exception_flag -= 1;

    let mut result = HandleResult::Search;
    for f in &handlers[..len] {
        result = (*f)(info);
        if result == HandleResult::Execution {
            break;
        }
    }

    // call default handler
    // ignore invalid return value, treat to HandleResult::Search
    // check SP to be written on SSA is pointing to the trusted stack
    let rsp = info.context.rsp as usize;
    try_abort!(!tds.is_valid_sp(rsp), tds);

    if result != HandleResult::Execution {
        tds.exception_flag = -1;
    }

    //instruction triggering the exception will be executed again.
    unsafe {
        continue_execution(info);
    }

    abort!(tds);
}
