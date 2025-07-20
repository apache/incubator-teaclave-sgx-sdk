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

#[link_section = ".nipd"]
#[no_mangle]
pub static mut ENCLAVE_STATE: u32 = 0;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u32)]
pub enum State {
    NotStarted = 0,
    InProgress = 1,
    InitDone = 2,
    Crashed = 3,
}

impl State {
    #[link_section = ".nipx"]
    pub fn is_done(&self) -> bool {
        *self == Self::InitDone
    }

    #[link_section = ".nipx"]
    pub fn is_crashed(&self) -> bool {
        *self == Self::Crashed
    }

    #[link_section = ".nipx"]
    pub fn is_not_started(&self) -> bool {
        *self == Self::NotStarted
    }
}

#[link_section = ".nipx"]
pub fn get_state() -> State {
    unsafe {
        match ENCLAVE_STATE {
            0 => State::NotStarted,
            1 => State::InProgress,
            2 => State::InitDone,
            3 => State::Crashed,
            _ => unreachable!(),
        }
    }
}

#[link_section = ".nipx"]
pub fn set_state(state: State) {
    let state = match state {
        State::NotStarted => 0,
        State::InProgress => 1,
        State::InitDone => 2,
        State::Crashed => 3,
    };
    unsafe {
        ENCLAVE_STATE = state;
    }
}

#[link_section = ".nipx"]
pub fn lock_state() -> State {
    const NOT_STARTED: u32 = 0;
    const IN_PROGRESS: u32 = 1;

    let state = unsafe {
        let (state, _) = core::intrinsics::atomic_cxchg_seqcst_seqcst(
            &mut ENCLAVE_STATE,
            NOT_STARTED,
            IN_PROGRESS,
        );
        state
    };
    match state {
        0 => State::NotStarted,
        1 => State::InProgress,
        2 => State::InitDone,
        3 => State::Crashed,
        _ => unreachable!(),
    }
}

#[link_section = ".nipx"]
pub fn is_crashed() -> bool {
    get_state().is_crashed()
}
