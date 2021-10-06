use sgx_libc::ocall::getpid;
use sgx_libc::{c_int, pid_t, siginfo_t, SIGILL, SIGINT, SIGTERM, SIGUSR1, SIGUSR2};
use sgx_signal::signal::{
    raise_signal, register, register_sigaction, unregister, unregister_signal,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn test_signal_forbidden() {
    let ret = register(SIGILL, || ());
    assert_eq!(ret.is_ok(), false);
}

pub fn test_signal_without_pid() {
    let status = Arc::new(AtomicUsize::new(0));
    let action = {
        let status = Arc::clone(&status);
        move || {
            status.store(1, Ordering::Relaxed);
        }
    };

    register(SIGUSR2, action).unwrap();
    raise_signal(SIGUSR2);

    for _ in 0..10 {
        thread::sleep(Duration::from_millis(100));
        let current = status.load(Ordering::Relaxed);
        match current {
            // Not yet
            0 => continue,
            // Good, we are done with the correct result
            _ if current == 1 => return,
            _ => panic!("Wrong result value {}", current),
        }
    }
    panic!("Timed out waiting for the signal");
}

pub fn test_signal_with_pid() {
    let status = Arc::new(AtomicUsize::new(0));
    let action = {
        let status = Arc::clone(&status);
        move |siginfo: &siginfo_t| {
            // Hack: currently, libc exposes only the first 3 fields of siginfo_t. The pid
            // comes somewhat later on. Therefore, we do a Really Ugly Hack and define our
            // own structure (and hope it is correct on all platforms). But hey, this is
            // only the tests, so we are going to get away with this.
            #[repr(C)]
            struct SigInfo {
                _fields: [c_int; 3],
                #[cfg(all(target_pointer_width = "64", target_os = "linux"))]
                _pad: c_int,
                pid: pid_t,
            }
            let info: &SigInfo =
                unsafe { (siginfo as *const _ as *const SigInfo).as_ref().unwrap() };

            status.store(info.pid as usize, Ordering::Relaxed);
        }
    };

    let pid = unsafe { getpid() };
    register_sigaction(SIGUSR2, action).unwrap();
    raise_signal(SIGUSR2);

    for _ in 0..10 {
        thread::sleep(Duration::from_millis(100));
        let current = status.load(Ordering::Relaxed);
        match current {
            // Not yet (PID == 0 doesn't happen)
            0 => continue,
            // Good, we are done with the correct result
            _ if current == pid as usize => return,
            _ => panic!("Wrong status value {}", current),
        }
    }
    panic!("Timed out waiting for the signal");
}

// Check that registration works as expected and that unregister tells if it did or not.
pub fn test_signal_register_unregister() {
    let signal = register(SIGUSR1, || ()).unwrap();
    // It was there now, so we can unregister
    assert!(unregister(signal));
    // The next time unregistering does nothing and tells us so.
    assert!(!unregister(signal));
}

pub fn test_signal_register_unregister1() {
    let called = Arc::new(AtomicUsize::new(0));
    let action = {
        let called = Arc::clone(&called);
        move || {
            called.fetch_add(1, Ordering::Relaxed);
        }
    };

    register(SIGTERM, action.clone()).unwrap();
    register(SIGTERM, action.clone()).unwrap();
    register(SIGINT, action.clone()).unwrap();

    raise_signal(SIGTERM);
    // The closure is run twice.
    assert_eq!(2, called.load(Ordering::Relaxed));

    assert!(unregister_signal(SIGTERM));

    raise_signal(SIGTERM);
    // Second one unregisters nothing.
    assert!(!unregister_signal(SIGTERM));

    // After unregistering (both), it is no longer run at all.
    assert_eq!(2, called.load(Ordering::Relaxed));

    // The SIGINT one is not disturbed.
    raise_signal(SIGINT);
    assert_eq!(3, called.load(Ordering::Relaxed));

    // But it's possible to register it again just fine.
    register(SIGTERM, action).unwrap();

    raise_signal(SIGTERM);
    assert_eq!(4, called.load(Ordering::Relaxed));
}
