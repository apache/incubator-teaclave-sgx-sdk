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

use crate::convert::{TryFrom, TryInto};
use crate::ffi::OsStr;
use crate::fmt;
use crate::io;
use crate::marker::PhantomData;
use crate::num::NonZeroI32;
use crate::path::Path;
use crate::sys::fd::FileDesc;
use crate::sys::fs::File;
use crate::sys::unsupported::pipe::AnonPipe;
use crate::sys::unsupported::{unsupported, unsupported_err};
use crate::sys_common::IntoInner;
use crate::sys_common::process::{CommandEnv, CommandEnvs};
use core::ffi::NonZero_c_int;

use sgx_oc as libc;
use libc::{c_int, gid_t, pid_t, uid_t};

pub use crate::ffi::OsString as EnvKey;

////////////////////////////////////////////////////////////////////////////////
// Command
////////////////////////////////////////////////////////////////////////////////

pub struct Command {
    env: CommandEnv,
}

// passed back to std::process with the pipes connected to the child, if any
// were requested
pub struct StdioPipes {
    pub stdin: Option<AnonPipe>,
    pub stdout: Option<AnonPipe>,
    pub stderr: Option<AnonPipe>,
}

pub enum Stdio {
    Inherit,
    Null,
    MakePipe,
    Fd(FileDesc),
}

impl Command {
    pub fn new(_program: &OsStr) -> Command {
        Command { env: Default::default() }
    }

    pub fn set_arg_0(&mut self, _arg: &OsStr) {}

    pub fn arg(&mut self, _arg: &OsStr) {}

    pub fn env_mut(&mut self) -> &mut CommandEnv {
        &mut self.env
    }

    pub fn cwd(&mut self, _dir: &OsStr) {}

    pub fn uid(&mut self, _id: uid_t) {}

    pub fn gid(&mut self, _id: gid_t) {}

    pub fn groups(&mut self, _groups: &[gid_t]) {}

    pub fn pgroup(&mut self, _pgroup: pid_t) {}

    pub fn stdin(&mut self, _stdin: Stdio) {}

    pub fn stdout(&mut self, _stdout: Stdio) {}

    pub fn stderr(&mut self, _stderr: Stdio) {}

    pub fn get_program(&self) -> &OsStr {
        panic!("unsupported")
    }

    pub fn get_args(&self) -> CommandArgs<'_> {
        CommandArgs { _p: PhantomData }
    }

    pub fn get_envs(&self) -> CommandEnvs<'_> {
        self.env.iter()
    }

    pub fn get_current_dir(&self) -> Option<&Path> {
        None
    }

    pub unsafe fn pre_exec(&mut self, _f: Box<dyn FnMut() -> io::Result<()> + Send + Sync>) {
        panic!("unsupported")
    }

    pub fn exec(&mut self, _default: Stdio) -> io::Error {
        unsupported_err()
    }

    pub fn spawn(
        &mut self,
        _default: Stdio,
        _needs_stdin: bool,
    ) -> io::Result<(Process, StdioPipes)> {
        unsupported()
    }
}

impl From<AnonPipe> for Stdio {
    fn from(pipe: AnonPipe) -> Stdio {
        Stdio::Fd(pipe.into_inner())
    }
}

impl From<File> for Stdio {
    fn from(file: File) -> Stdio {
        Stdio::Fd(file.into_inner())
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

/// Unix exit statuses
//
// This is not actually an "exit status" in Unix terminology.  Rather, it is a "wait status".
// See the discussion in comments and doc comments for `std::process::ExitStatus`.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ExitStatus(c_int);

impl fmt::Debug for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("unix_wait_status").field(&self.0).finish()
    }
}

impl ExitStatus {
    pub fn new(status: c_int) -> ExitStatus {
        ExitStatus(status)
    }

    fn exited(&self) -> bool {
        libc::WIFEXITED(self.0)
    }

    pub fn exit_ok(&self) -> Result<(), ExitStatusError> {
        // This assumes that WIFEXITED(status) && WEXITSTATUS==0 corresponds to status==0.  This is
        // true on all actual versions of Unix, is widely assumed, and is specified in SuS
        // https://pubs.opengroup.org/onlinepubs/9699919799/functions/wait.html .  If it is not
        // true for a platform pretending to be Unix, the tests (our doctests, and also
        // procsss_unix/tests.rs) will spot it.  `ExitStatusError::code` assumes this too.
        match NonZero_c_int::try_from(self.0) {
            /* was nonzero */ Ok(failure) => Err(ExitStatusError(failure)),
            /* was zero, couldn't convert */ Err(_) => Ok(()),
        }
    }

    pub fn code(&self) -> Option<i32> {
        self.exited().then(|| libc::WEXITSTATUS(self.0))
    }

    pub fn signal(&self) -> Option<i32> {
        libc::WIFSIGNALED(self.0).then(|| libc::WTERMSIG(self.0))
    }

    pub fn core_dumped(&self) -> bool {
        libc::WIFSIGNALED(self.0) && libc::WCOREDUMP(self.0)
    }

    pub fn stopped_signal(&self) -> Option<i32> {
        libc::WIFSTOPPED(self.0).then(|| libc::WSTOPSIG(self.0))
    }

    pub fn continued(&self) -> bool {
        libc::WIFCONTINUED(self.0)
    }

    pub fn into_raw(&self) -> c_int {
        self.0
    }
}

/// Converts a raw `c_int` to a type-safe `ExitStatus` by wrapping it without copying.
impl From<c_int> for ExitStatus {
    fn from(a: c_int) -> ExitStatus {
        ExitStatus(a)
    }
}

/// Convert a signal number to a readable, searchable name.
///
/// This string should be displayed right after the signal number.
/// If a signal is unrecognized, it returns the empty string, so that
/// you just get the number like "0". If it is recognized, you'll get
/// something like "9 (SIGKILL)".
fn signal_string(signal: i32) -> &'static str {
    match signal {
        libc::SIGHUP => " (SIGHUP)",
        libc::SIGINT => " (SIGINT)",
        libc::SIGQUIT => " (SIGQUIT)",
        libc::SIGILL => " (SIGILL)",
        libc::SIGTRAP => " (SIGTRAP)",
        libc::SIGABRT => " (SIGABRT)",
        libc::SIGBUS => " (SIGBUS)",
        libc::SIGFPE => " (SIGFPE)",
        libc::SIGKILL => " (SIGKILL)",
        libc::SIGUSR1 => " (SIGUSR1)",
        libc::SIGSEGV => " (SIGSEGV)",
        libc::SIGUSR2 => " (SIGUSR2)",
        libc::SIGPIPE => " (SIGPIPE)",
        libc::SIGALRM => " (SIGALRM)",
        libc::SIGTERM => " (SIGTERM)",
        libc::SIGCHLD => " (SIGCHLD)",
        libc::SIGCONT => " (SIGCONT)",
        libc::SIGSTOP => " (SIGSTOP)",
        libc::SIGTSTP => " (SIGTSTP)",
        libc::SIGTTIN => " (SIGTTIN)",
        libc::SIGTTOU => " (SIGTTOU)",
        libc::SIGURG => " (SIGURG)",
        libc::SIGXCPU => " (SIGXCPU)",
        libc::SIGXFSZ => " (SIGXFSZ)",
        libc::SIGVTALRM => " (SIGVTALRM)",
        libc::SIGPROF => " (SIGPROF)",
        libc::SIGWINCH => " (SIGWINCH)",
        libc::SIGIO => " (SIGIO)",
        libc::SIGSYS => " (SIGSYS)",
        libc::SIGSTKFLT => " (SIGSTKFLT)",
        libc::SIGPWR => " (SIGPWR)",
        _ => "",
    }
}

impl fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(code) = self.code() {
            write!(f, "exit status: {code}")
        } else if let Some(signal) = self.signal() {
            let signal_string = signal_string(signal);
            if self.core_dumped() {
                write!(f, "signal: {signal}{signal_string} (core dumped)")
            } else {
                write!(f, "signal: {signal}{signal_string}")
            }
        } else if let Some(signal) = self.stopped_signal() {
            let signal_string = signal_string(signal);
            write!(f, "stopped (not terminated) by signal: {signal}{signal_string}")
        } else if self.continued() {
            write!(f, "continued (WIFCONTINUED)")
        } else {
            write!(f, "unrecognised wait status: {} {:#x}", self.0, self.0)
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct ExitStatusError(NonZero_c_int);

#[allow(clippy::from_over_into)]
impl Into<ExitStatus> for ExitStatusError {
    fn into(self) -> ExitStatus {
        ExitStatus(self.0.into())
    }
}

impl fmt::Debug for ExitStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("unix_wait_status").field(&self.0).finish()
    }
}

impl ExitStatusError {
    pub fn code(self) -> Option<NonZeroI32> {
        ExitStatus(self.0.into()).code().map(|st| st.try_into().unwrap())
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ExitCode(bool);

impl ExitCode {
    pub const SUCCESS: ExitCode = ExitCode(false);
    pub const FAILURE: ExitCode = ExitCode(true);

    pub fn as_i32(&self) -> i32 {
        self.0 as i32
    }
}

impl From<u8> for ExitCode {
    fn from(code: u8) -> Self {
        match code {
            0 => Self::SUCCESS,
            1..=255 => Self::FAILURE,
        }
    }
}

pub struct Process(!);

impl Process {
    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.0
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.0
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.0
    }
}

pub struct CommandArgs<'a> {
    _p: PhantomData<&'a ()>,
}

impl<'a> Iterator for CommandArgs<'a> {
    type Item = &'a OsStr;
    fn next(&mut self) -> Option<&'a OsStr> {
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

impl<'a> ExactSizeIterator for CommandArgs<'a> {}

impl<'a> fmt::Debug for CommandArgs<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().finish()
    }
}
