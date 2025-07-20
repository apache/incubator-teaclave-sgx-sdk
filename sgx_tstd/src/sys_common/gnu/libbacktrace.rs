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
// under the License.

use crate::marker;
use crate::mem;
use crate::ptr;
use crate::slice;
use crate::sys;
use crate::sys::backtrace::{self, Bomb, BytesOrWideString};
use crate::sys_common::backtrace::{ResolveWhat, SymbolName};

use sgx_trts::trts::MmLayout;
use sgx_unwind as uw;

use sgx_backtrace_sys as bt;
use sgx_oc::{c_char, c_int, c_void, uintptr_t};


pub enum Symbol<'a> {
    Syminfo {
        pc: uintptr_t,
        symname: *const c_char,
        _marker: marker::PhantomData<&'a ()>,
    },
    Pcinfo {
        pc: uintptr_t,
        filename: *const c_char,
        lineno: c_int,
        function: *const c_char,
        symname: *const c_char,
    },
}

impl Symbol<'_> {
    pub fn name(&self) -> Option<SymbolName<'_>> {
        let symbol = |ptr: *const c_char| unsafe {
            if ptr.is_null() {
                None
            } else {
                let len = sys::strlen(ptr);
                Some(SymbolName::new(slice::from_raw_parts(
                    ptr as *const u8,
                    len,
                )))
            }
        };
        match *self {
            Symbol::Syminfo { symname, .. } => symbol(symname),
            Symbol::Pcinfo {
                function, symname, ..
            } => {
                // If possible prefer the `function` name which comes from
                // debuginfo and can typically be more accurate for inline
                // frames for example. If that's not present though fall back to
                // the symbol table name specified in `symname`.
                //
                // Note that sometimes `function` can feel somewhat less
                // accurate, for example being listed as `try<i32,closure>`
                // isntead of `std::panicking::try::do_call`. It's not really
                // clear why, but overall the `function` name seems more accurate.
                if let Some(sym) = symbol(function) {
                    return Some(sym);
                }
                symbol(symname)
            }
        }
    }

    pub fn name_bytes(&self) -> Option<&[u8]> {
        let symbol = |ptr: *const c_char| unsafe {
            if ptr.is_null() {
                None
            } else {
                let len = sys::strlen(ptr);
                Some(slice::from_raw_parts(ptr as *const u8, len))
            }
        };
        match *self {
            Symbol::Syminfo { symname, .. } => symbol(symname),
            Symbol::Pcinfo {
                function, symname, ..
            } => {
                // If possible prefer the `function` name which comes from
                // debuginfo and can typically be more accurate for inline
                // frames for example. If that's not present though fall back to
                // the symbol table name specified in `symname`.
                //
                // Note that sometimes `function` can feel somewhat less
                // accurate, for example being listed as `try<i32,closure>`
                // isntead of `std::panicking::try::do_call`. It's not really
                // clear why, but overall the `function` name seems more accurate.
                if let Some(sym) = symbol(function) {
                    return Some(sym);
                }
                symbol(symname)
            }
        }
    }

    pub fn addr(&self) -> Option<*mut c_void> {
        let pc = match *self {
            Symbol::Syminfo { pc, .. } => pc,
            Symbol::Pcinfo { pc, .. } => pc,
        };
        if pc == 0 {
            None
        } else {
            Some(pc as *mut _)
        }
    }

    fn filename_bytes(&self) -> Option<&[u8]> {
        match *self {
            Symbol::Syminfo { .. } => None,
            Symbol::Pcinfo { filename, .. } => {
                let ptr = filename as *const u8;
                if ptr.is_null() {
                    return None;
                }
                unsafe {
                    let len = sys::strlen(filename);
                    Some(slice::from_raw_parts(ptr, len))
                }
            }
        }
    }

    pub fn filename_raw(&self) -> Option<BytesOrWideString<'_>> {
        self.filename_bytes().map(BytesOrWideString::Bytes)
    }

    pub fn lineno(&self) -> Option<u32> {
        match *self {
            Symbol::Syminfo { .. } => None,
            Symbol::Pcinfo { lineno, .. } => Some(lineno as u32),
        }
    }

    pub fn colno(&self) -> Option<u32> {
        None
    }
}

extern "C" fn error_cb(_data: *mut c_void, _msg: *const c_char, _errnum: c_int) {
    // do nothing for now
}

/// Type of the `data` pointer passed into `syminfo_cb`
struct SyminfoState<'a> {
    cb: &'a mut (dyn FnMut(&super::Symbol) + 'a),
    pc: usize,
}

static ENCLAVE_ENTRY_NAME: &str = "enclave_entry\0";

extern "C" fn syminfo_cb(
    data: *mut c_void,
    pc: uintptr_t,
    symname: *const c_char,
    _symval: uintptr_t,
    _symsize: uintptr_t,
) {
    let state = unsafe { init_state() };
    if state.is_null() {
        return;
    }

    let mut bomb = Bomb::new(true);

    // Once this callback is invoked from `backtrace_syminfo` when we start
    // resolving we go further to call `backtrace_pcinfo`. The
    // `backtrace_pcinfo` function will consult debug information and attemp tto
    // do things like recover file/line information as well as inlined frames.
    // Note though that `backtrace_pcinfo` can fail or not do much if there's
    // not debug info, so if that happens we're sure to call the callback with
    // at least one symbol from the `syminfo_cb`.
    unsafe {
        let syminfo_state = &mut *(data as *mut SyminfoState<'_>);
        let mut pcinfo_state = PcinfoState {
            symname,
            called: false,
            cb: syminfo_state.cb,
        };
        bt::backtrace_pcinfo(
            state,
            syminfo_state.pc as uintptr_t,
            pcinfo_cb,
            error_cb,
            &mut pcinfo_state as *mut _ as *mut _,
        );
        if !pcinfo_state.called {
            let mut symname = symname;
            if symname.is_null() {
                let sym_address =
                    uw::_Unwind_FindEnclosingFunction((pc + 1) as *mut c_void) as usize;
                if sym_address == MmLayout::entry_address() {
                    symname = ENCLAVE_ENTRY_NAME as *const _ as *const c_char
                }
            }

            let inner = Symbol::Syminfo {
                pc,
                symname,
                _marker: marker::PhantomData,
            };
            (pcinfo_state.cb)(&super::Symbol {
                name: inner.name_bytes().map(|m| m.to_vec()),
                addr: inner.addr(),
                filename: inner.filename_bytes().map(|m| m.to_vec()),
                lineno: inner.lineno(),
                colno: inner.colno(),
            });
        }
    }

    bomb.set(false);
}

/// Type of the `data` pointer passed into `pcinfo_cb`
struct PcinfoState<'a> {
    cb: &'a mut (dyn FnMut(&super::Symbol) + 'a),
    symname: *const c_char,
    called: bool,
}

extern "C" fn pcinfo_cb(
    data: *mut c_void,
    pc: uintptr_t,
    filename: *const c_char,
    lineno: c_int,
    function: *const c_char,
) -> c_int {
    if filename.is_null() || function.is_null() {
        return -1;
    }
    let mut bomb = Bomb::new(true);

    unsafe {
        let state = &mut *(data as *mut PcinfoState);
        state.called = true;
        let inner = Symbol::Pcinfo {
            pc,
            filename,
            lineno,
            symname: state.symname,
            function,
        };
        (state.cb)(&super::Symbol {
            name: inner.name_bytes().map(|m| m.to_vec()),
            addr: inner.addr(),
            filename: inner.filename_bytes().map(|m| m.to_vec()),
            lineno: inner.lineno(),
            colno: inner.colno(),
        });
    }

    bomb.set(false);
    0
}

// The libbacktrace API supports creating a state, but it does not
// support destroying a state. I personally take this to mean that a
// state is meant to be created and then live forever.
//
// I would love to register an at_exit() handler which cleans up this
// state, but libbacktrace provides no way to do so.
//
// With these constraints, this function has a statically cached state
// that is calculated the first time this is requested. Remember that
// backtracing all happens serially (one global lock).
//
// Note the lack of synchronization here is due to the requirement that
// `resolve` is externally synchronized.
unsafe fn init_state() -> *mut bt::backtrace_state {
    static mut STATE: *mut bt::backtrace_state = 0 as *mut _;

    if !STATE.is_null() {
        return STATE;
    }

    let filename = match backtrace::gnu::get_enclave_filename() {
        Ok(filename) => {
            // filename is purposely leaked here since libbacktrace requires
            // it to stay allocated permanently.
            let filename_ptr = filename.as_ptr();
            mem::forget(filename);
            filename_ptr
        }
        Err(_) => return ptr::null_mut(),
    };

    STATE = bt::backtrace_create_state(
        filename.cast(),
        // Don't exercise threadsafe capabilities of libbacktrace since
        // we're always calling it in a synchronized fashion.
        0,
        error_cb,
        ptr::null_mut(), // no extra data
    );

    STATE
}

pub unsafe fn resolve(what: ResolveWhat<'_>, cb: &mut dyn FnMut(&super::Symbol)) {
    let symaddr = what.address_or_ip() as usize;

    // backtrace errors are currently swept under the rug
    let state = init_state();
    if state.is_null() {
        return;
    }

    // Call the `backtrace_syminfo` API which (from reading the code)
    // should call `syminfo_cb` exactly once (or fail with an error
    // presumably). We then handle more within the `syminfo_cb`.
    //
    // Note that we do this since `syminfo` will consult the symbol table,
    // finding symbol names even if there's no debug information in the binary.
    let mut syminfo_state = SyminfoState { pc: symaddr, cb };
    bt::backtrace_syminfo(
        state,
        symaddr as uintptr_t,
        syminfo_cb,
        error_cb,
        &mut syminfo_state as *mut _ as *mut _,
    );
}

pub unsafe fn clear_symbol_cache() {}
