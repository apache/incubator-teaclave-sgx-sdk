use std::sync::Mutex;
use std::time::Duration;
use std::sync::atomic::{AtomicIsize, Ordering};
use libc::{self, c_int, c_void};
use std::io::Error;
use std::slice;
use std::sync::Once;

static mut GLOBAL_TCS_CACHE: *mut SgxTcsInfoCache = 0 as *mut SgxTcsInfoCache;
static INIT: Once = Once::new();

pub const FUTEX_WAIT: usize = 0;
pub const FUTEX_WAKE: usize = 1;

pub struct SeEvent {
    event: AtomicIsize,
}

impl SeEvent {
    pub fn new() -> Self {
        SeEvent{
            event: AtomicIsize::new(0),
        }
    }

    pub fn wait_timeout(&self, dur: Duration) -> i32 {
        if self.event.fetch_add(-1, Ordering::SeqCst) == 0 {

            if dur.as_millis() == 0 {
                unsafe {
                    libc::syscall(libc::SYS_futex,
                                  self.event.load(Ordering::SeqCst),
                                  FUTEX_WAIT,
                                  -1,
                                  0,
                                  0,
                                  0);
                }
            } else {
                let timeout: libc::timespec = libc::timespec {
                    tv_sec: dur.as_secs() as i64, tv_nsec: dur.subsec_nanos() as i64
                };

                let ret = unsafe {
                    libc::syscall(libc::SYS_futex,
                                  self.event.load(Ordering::SeqCst),
                                  FUTEX_WAIT,
                                  -1,
                                  &timeout,
                                  0,
                                  0)
                };
                if ret < 0 {
                    let _err = Error::last_os_error().raw_os_error().unwrap_or(0);
                    if _err == libc::ETIMEDOUT {
                        return _err;
                    }
                }
            }
        }
        0
    }

    pub fn wake(&self) -> i32 {
        if self.event.fetch_add(1, Ordering::SeqCst) != 0 {
            unsafe {
                libc::syscall(libc::SYS_futex,
                              self.event.load(Ordering::SeqCst),
                              FUTEX_WAKE,
                              1,
                              0,
                              0,
                              0)
            };
        }
        0
    }
}

struct SgxTcsInfo<'a> {
    tcs: usize,
    se_event: &'a SeEvent,
}

struct SgxTcsInfoCache<'a> {
    cache:Mutex<Vec<SgxTcsInfo<'a>>>
}

impl<'a> SgxTcsInfoCache<'a> {
    fn new() -> Self {
        SgxTcsInfoCache {
            cache: Mutex::new(Vec::new()),
        }
    }

    pub fn get_event(&self, tcs: usize) -> &SeEvent {
        let v = &mut *self.cache.lock().unwrap();
        let op = v.as_slice().iter().position(|ref x| x.tcs == tcs);
        match op {
            Some(i) => v[i].se_event,
            None => {
                let event: &SeEvent = unsafe { &*Box::into_raw(Box::new(SeEvent::new())) };
                v.push(SgxTcsInfo{
                        tcs: tcs,
                        se_event: event});
                let len = v.len();
                v[len -1].se_event
            },
        }
    }
}

pub fn get_tcs_event(_tcs: usize) -> & 'static SeEvent {
    unsafe {
        INIT.call_once(|| {
            GLOBAL_TCS_CACHE = Box::into_raw(Box::new(SgxTcsInfoCache::new())) as * mut SgxTcsInfoCache;
        });
        (*GLOBAL_TCS_CACHE).get_event(_tcs)
    }
}

#[no_mangle]
pub	extern	"C" fn u_thread_set_event_ocall(error: * mut c_int, tcs: * const c_void) -> c_int {
    if tcs.is_null() {
        if !error.is_null() {
            unsafe { *error = libc::EINVAL; }
        }
        return -1;
    }
    let result = get_tcs_event(tcs as usize).wake();
    if result != 0 {
        if !error.is_null() {
            unsafe { *error = Error::last_os_error().raw_os_error().unwrap_or(0); }
            return -1;
        }
    }
    if !error.is_null() {
        unsafe { *error = 0; }
    }
    result as c_int
}

#[no_mangle]
pub	extern	"C" fn u_thread_wait_event_ocall(error: * mut c_int, tcs: * const c_void, timeout: c_int) -> c_int {
    if tcs.is_null() {
        if !error.is_null() {
            unsafe { *error = libc::EINVAL; }
        }
        return -1;
    }
    let result = get_tcs_event(tcs as usize).wait_timeout(Duration::from_millis(timeout as u64));
    if result != 0 {
        if !error.is_null() {
            unsafe { *error = Error::last_os_error().raw_os_error().unwrap_or(0); }
            return -1;
        }
    }

    if !error.is_null() {
        unsafe { *error = 0; }
    }

    result as c_int
}

#[no_mangle]
pub extern "C" fn u_thread_set_multiple_events_ocall(error: * mut c_int, tcss: * const * const c_void, total: c_int) -> c_int {

    if tcss.is_null() {
        if !error.is_null() {
            unsafe { *error = libc::EINVAL; }
        }
        return -1;
    }

    let tcss_slice = unsafe { slice::from_raw_parts(tcss, total as usize) };
    let mut result = 0;
    for tcs in tcss_slice.iter() {
        result = get_tcs_event(*tcs as usize).wake();
        if result != 0 {
            if !error.is_null() {
                unsafe { *error = Error::last_os_error().raw_os_error().unwrap_or(0); }
                return -1;
            }
         }
    }

    if !error.is_null() {
        unsafe { *error = 0; }
    }
    result as c_int
}

#[no_mangle]
pub extern "C" fn u_thread_setwait_events_ocall(error: * mut c_int,
                                                waiter_tcs: * const c_void,
                                                self_tcs: * const c_void,
                                                timeout: c_int) -> c_int {

    let result = u_thread_set_event_ocall(error, waiter_tcs);
    if result < 0 {
        result
    } else {
        u_thread_wait_event_ocall(error, self_tcs, timeout)
    }
}
