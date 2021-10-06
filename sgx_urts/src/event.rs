use libc::timespec;
use libc::{self, c_int, c_void};
use std::io::Error;
use std::slice;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Mutex;
use std::sync::Once;

static mut GLOBAL_TCS_CACHE: Option<SgxTcsInfoCache> = None;
static INIT: Once = Once::new();

pub const FUTEX_WAIT: usize = 0;
pub const FUTEX_WAKE: usize = 1;

pub struct SeEvent {
    event: AtomicI32,
}

impl Default for SeEvent {
    fn default() -> Self {
        Self {
            event: AtomicI32::new(0),
        }
    }
}
impl SeEvent {
    pub fn new() -> SeEvent {
        SeEvent::default()
    }

    pub fn wait_timeout(&self, timeout: &timespec) -> i32 {
        if self.event.fetch_add(-1, Ordering::SeqCst) == 0 {
            let ret = unsafe {
                libc::syscall(
                    libc::SYS_futex,
                    self,
                    FUTEX_WAIT,
                    -1,
                    timeout as *const timespec,
                    0,
                    0,
                )
            };
            if ret < 0 {
                let err = Error::last_os_error().raw_os_error().unwrap_or(0);
                if err == libc::ETIMEDOUT {
                    let _ = self
                        .event
                        .compare_exchange(-1, 0, Ordering::SeqCst, Ordering::SeqCst);
                    return -1;
                }
            }
        }
        0
    }

    pub fn wait(&self) -> i32 {
        if self.event.fetch_add(-1, Ordering::SeqCst) == 0 {
            let ret = unsafe { libc::syscall(libc::SYS_futex, self, FUTEX_WAIT, -1, 0, 0, 0) };
            if ret < 0 {
                let _err = Error::last_os_error().raw_os_error().unwrap_or(0);
            }
        }
        0
    }

    pub fn wake(&self) -> i32 {
        if self.event.fetch_add(1, Ordering::SeqCst) != 0 {
            unsafe { libc::syscall(libc::SYS_futex, self, FUTEX_WAKE, 1, 0, 0, 0) };
        }
        0
    }
}

struct SgxTcsInfo<'a> {
    tcs: usize,
    se_event: &'a SeEvent,
}

struct SgxTcsInfoCache<'a> {
    cache: Mutex<Vec<SgxTcsInfo<'a>>>,
}

impl<'a> SgxTcsInfoCache<'a> {
    fn new() -> SgxTcsInfoCache<'a> {
        SgxTcsInfoCache {
            cache: Mutex::new(Vec::new()),
        }
    }

    pub fn get_event(&self, tcs: usize) -> &SeEvent {
        let v = &mut *self.cache.lock().unwrap();
        let op = v.as_slice().iter().position(|x| x.tcs == tcs);
        match op {
            Some(i) => v[i].se_event,
            None => {
                let event: &SeEvent = unsafe { &*Box::into_raw(Box::new(SeEvent::new())) };
                v.push(SgxTcsInfo {
                    tcs,
                    se_event: event,
                });
                let len = v.len();
                v[len - 1].se_event
            }
        }
    }
}

pub fn get_tcs_event(_tcs: usize) -> &'static SeEvent {
    unsafe {
        INIT.call_once(|| {
            GLOBAL_TCS_CACHE = Some(SgxTcsInfoCache::new());
        });
        GLOBAL_TCS_CACHE
            .as_ref()
            .expect("GLOBAL_TCS_CACHE is not initialized.")
            .get_event(_tcs)
    }
}

#[no_mangle]
pub extern "C" fn u_thread_set_event_ocall(error: *mut c_int, tcs: *const c_void) -> c_int {
    if tcs.is_null() {
        if !error.is_null() {
            unsafe {
                *error = libc::EINVAL;
            }
        }
        return -1;
    }
    let result = get_tcs_event(tcs as usize).wake();
    if result != 0 {
        if !error.is_null() {
            unsafe {
                *error = Error::last_os_error().raw_os_error().unwrap_or(0);
            }
        }
        -1
    } else {
        if !error.is_null() {
            unsafe {
                *error = 0;
            }
        }
        result as c_int
    }
}

#[no_mangle]
pub extern "C" fn u_thread_wait_event_ocall(
    error: *mut c_int,
    tcs: *const c_void,
    timeout: *const timespec,
) -> c_int {
    if tcs.is_null() {
        if !error.is_null() {
            unsafe {
                *error = libc::EINVAL;
            }
        }
        return -1;
    }

    let result = if timeout.is_null() {
        get_tcs_event(tcs as usize).wait()
    } else {
        get_tcs_event(tcs as usize).wait_timeout(unsafe { &*timeout })
    };
    if result != 0 {
        if !error.is_null() {
            unsafe {
                *error = Error::last_os_error().raw_os_error().unwrap_or(0);
            }
        }
        -1
    } else {
        if !error.is_null() {
            unsafe {
                *error = 0;
            }
        }
        result as c_int
    }
}

#[no_mangle]
pub extern "C" fn u_thread_set_multiple_events_ocall(
    error: *mut c_int,
    tcss: *const *const c_void,
    total: c_int,
) -> c_int {
    if tcss.is_null() {
        if !error.is_null() {
            unsafe {
                *error = libc::EINVAL;
            }
        }
        return -1;
    }

    let tcss_slice = unsafe { slice::from_raw_parts(tcss, total as usize) };
    let mut result = 0;
    for tcs in tcss_slice.iter() {
        result = get_tcs_event(*tcs as usize).wake();
        if result != 0 {
            if !error.is_null() {
                unsafe {
                    *error = Error::last_os_error().raw_os_error().unwrap_or(0);
                }
            }
            return -1;
        }
    }

    if !error.is_null() {
        unsafe {
            *error = 0;
        }
    }
    result as c_int
}

#[no_mangle]
pub extern "C" fn u_thread_setwait_events_ocall(
    error: *mut c_int,
    waiter_tcs: *const c_void,
    self_tcs: *const c_void,
    timeout: *const timespec,
) -> c_int {
    let result = u_thread_set_event_ocall(error, waiter_tcs);
    if result < 0 {
        result
    } else {
        u_thread_wait_event_ocall(error, self_tcs, timeout)
    }
}
