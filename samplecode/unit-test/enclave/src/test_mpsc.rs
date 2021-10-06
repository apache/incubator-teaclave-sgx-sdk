use std::boxed::Box;
use std::env;
use std::sync::mpsc::{
    channel, sync_channel, Receiver, RecvTimeoutError, SendError, Sender, SyncSender, TryRecvError,
    TrySendError,
};
use std::thread;
use std::time::{Duration, Instant};
use std::untrusted::time::InstantEx;

pub fn stress_factor() -> usize {
    match env::var("SGXRUST_TEST_STRESS") {
        Ok(val) => val.parse().unwrap(),
        Err(..) => 1,
    }
}

pub fn test_mpsc_smoke() {
    let (tx, rx) = channel::<i32>();
    tx.send(1).unwrap();
    assert_eq!(rx.recv().unwrap(), 1);
}

pub fn test_mpsc_drop_full() {
    let (tx, _rx) = channel::<Box<isize>>();
    tx.send(box 1).unwrap();
}

pub fn test_mpsc_drop_full_shared() {
    let (tx, _rx) = channel::<Box<isize>>();
    drop(tx.clone());
    drop(tx.clone());
    tx.send(box 1).unwrap();
}

pub fn test_mpsc_smoke_shared() {
    let (tx, rx) = channel::<i32>();
    tx.send(1).unwrap();
    assert_eq!(rx.recv().unwrap(), 1);
    let tx = tx.clone();
    tx.send(1).unwrap();
    assert_eq!(rx.recv().unwrap(), 1);
}

pub fn test_mpsc_smoke_threads() {
    let (tx, rx) = channel::<i32>();
    let _t = thread::spawn(move || {
        tx.send(1).unwrap();
    });
    assert_eq!(rx.recv().unwrap(), 1);
}

pub fn test_mpsc_smoke_port_gone() {
    let (tx, rx) = channel::<i32>();
    drop(rx);
    assert!(tx.send(1).is_err());
}

pub fn test_mpsc_smoke_shared_port_gone() {
    let (tx, rx) = channel::<i32>();
    drop(rx);
    assert!(tx.send(1).is_err())
}

pub fn test_mpsc_smoke_shared_port_gone2() {
    let (tx, rx) = channel::<i32>();
    drop(rx);
    let tx2 = tx.clone();
    drop(tx);
    assert!(tx2.send(1).is_err());
}

pub fn test_mpsc_port_gone_concurrent() {
    let (tx, rx) = channel::<i32>();
    let _t = thread::spawn(move || {
        rx.recv().unwrap();
    });
    while tx.send(1).is_ok() {}
}

pub fn test_mpsc_port_gone_concurrent_shared() {
    let (tx, rx) = channel::<i32>();
    let tx2 = tx.clone();
    let _t = thread::spawn(move || {
        rx.recv().unwrap();
    });
    while tx.send(1).is_ok() && tx2.send(1).is_ok() {}
}

pub fn test_mpsc_smoke_chan_gone() {
    let (tx, rx) = channel::<i32>();
    drop(tx);
    assert!(rx.recv().is_err());
}

pub fn test_mpsc_smoke_chan_gone_shared() {
    let (tx, rx) = channel::<()>();
    let tx2 = tx.clone();
    drop(tx);
    drop(tx2);
    assert!(rx.recv().is_err());
}

pub fn test_mpsc_chan_gone_concurrent() {
    let (tx, rx) = channel::<i32>();
    let _t = thread::spawn(move || {
        tx.send(1).unwrap();
        tx.send(1).unwrap();
    });
    while rx.recv().is_ok() {}
}

pub fn test_mpsc_stress() {
    let (tx, rx) = channel::<i32>();
    let t = thread::spawn(move || {
        for _ in 0..100 {
            tx.send(1).unwrap();
        }
    });
    for _ in 0..100 {
        assert_eq!(rx.recv().unwrap(), 1);
    }
    t.join().ok().expect("thread panicked");
}

pub fn test_mpsc_stress_shared() {
    const AMT: u32 = 2;
    const NTHREADS: u32 = 2;
    let (tx, rx) = channel::<i32>();

    let t = thread::spawn(move || {
        for _ in 0..AMT * NTHREADS {
            assert_eq!(rx.recv().unwrap(), 1);
        }
        match rx.try_recv() {
            Ok(..) => panic!(),
            _ => {}
        }
    });

    for _ in 0..NTHREADS {
        let tx = tx.clone();
        thread::spawn(move || {
            for _ in 0..AMT {
                tx.send(1).unwrap();
            }
        });
    }
    drop(tx);
    t.join().ok().expect("thread panicked");
}

pub fn test_mpsc_send_from_outside_runtime() {
    let (tx1, rx1) = channel::<()>();
    let (tx2, rx2) = channel::<i32>();
    let t1 = thread::spawn(move || {
        tx1.send(()).unwrap();
        for _ in 0..40 {
            assert_eq!(rx2.recv().unwrap(), 1);
        }
    });
    rx1.recv().unwrap();
    let t2 = thread::spawn(move || {
        for _ in 0..40 {
            tx2.send(1).unwrap();
        }
    });
    t1.join().ok().expect("thread panicked");
    t2.join().ok().expect("thread panicked");
}

pub fn test_mpsc_recv_from_outside_runtime() {
    let (tx, rx) = channel::<i32>();
    let t = thread::spawn(move || {
        for _ in 0..40 {
            assert_eq!(rx.recv().unwrap(), 1);
        }
    });
    for _ in 0..40 {
        tx.send(1).unwrap();
    }
    t.join().ok().expect("thread panicked");
}

pub fn test_mpsc_no_runtime() {
    let (tx1, rx1) = channel::<i32>();
    let (tx2, rx2) = channel::<i32>();
    let t1 = thread::spawn(move || {
        assert_eq!(rx1.recv().unwrap(), 1);
        tx2.send(2).unwrap();
    });
    let t2 = thread::spawn(move || {
        tx1.send(1).unwrap();
        assert_eq!(rx2.recv().unwrap(), 2);
    });
    t1.join().ok().expect("thread panicked");
    t2.join().ok().expect("thread panicked");
}

pub fn test_mpsc_oneshot_single_thread_close_port_first() {
    // Simple test of closing without sending
    let (_tx, rx) = channel::<i32>();
    drop(rx);
}

pub fn test_mpsc_oneshot_single_thread_close_chan_first() {
    // Simple test of closing without sending
    let (tx, _rx) = channel::<i32>();
    drop(tx);
}
pub fn test_mpsc_oneshot_single_thread_send_port_close() {
    // Testing that the sender cleans up the payload if receiver is closed
    let (tx, rx) = channel::<Box<i32>>();
    drop(rx);
    assert!(tx.send(box 0).is_err());
}

#[allow(dead_code)]
pub fn test_mpsc_oneshot_single_thread_recv_chan_close() {
    // Receiving on a closed chan will panic
    let res = thread::spawn(move || {
        let (tx, rx) = channel::<i32>();
        drop(tx);
        rx.recv().unwrap();
    })
    .join();
    // What is our res?
    assert!(res.is_err());
}

pub fn test_mpsc_oneshot_single_thread_send_then_recv() {
    let (tx, rx) = channel::<Box<i32>>();
    tx.send(box 10).unwrap();
    assert!(*rx.recv().unwrap() == 10);
}

pub fn test_mpsc_oneshot_single_thread_try_send_open() {
    let (tx, rx) = channel::<i32>();
    assert!(tx.send(10).is_ok());
    assert!(rx.recv().unwrap() == 10);
}

pub fn test_mpsc_oneshot_single_thread_try_send_closed() {
    let (tx, rx) = channel::<i32>();
    drop(rx);
    assert!(tx.send(10).is_err());
}

pub fn test_mpsc_oneshot_single_thread_try_recv_open() {
    let (tx, rx) = channel::<i32>();
    tx.send(10).unwrap();
    assert!(rx.recv() == Ok(10));
}

pub fn test_mpsc_oneshot_single_thread_try_recv_closed() {
    let (tx, rx) = channel::<i32>();
    drop(tx);
    assert!(rx.recv().is_err());
}

pub fn test_mpsc_oneshot_single_thread_peek_data() {
    let (tx, rx) = channel::<i32>();
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    tx.send(10).unwrap();
    assert_eq!(rx.try_recv(), Ok(10));
}

pub fn test_mpsc_oneshot_single_thread_peek_close() {
    let (tx, rx) = channel::<i32>();
    drop(tx);
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
}

pub fn test_mpsc_oneshot_single_thread_peek_open() {
    let (_tx, rx) = channel::<i32>();
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

pub fn test_mpsc_oneshot_multi_task_recv_then_send() {
    let (tx, rx) = channel::<Box<i32>>();
    let _t = thread::spawn(move || {
        assert!(*rx.recv().unwrap() == 10);
    });

    tx.send(box 10).unwrap();
}

#[allow(dead_code)]
pub fn test_mpsc_oneshot_multi_task_recv_then_close() {
    let (tx, rx) = channel::<Box<i32>>();
    let _t = thread::spawn(move || {
        drop(tx);
    });
    let res = thread::spawn(move || {
        assert!(*rx.recv().unwrap() == 10);
    })
    .join();
    assert!(res.is_err());
}

pub fn test_mpsc_oneshot_multi_thread_close_stress() {
    for _ in 0..stress_factor() {
        let (tx, rx) = channel::<i32>();
        let _t = thread::spawn(move || {
            drop(rx);
        });
        drop(tx);
    }
}

#[allow(dead_code)]
pub fn test_mpsc_oneshot_multi_thread_send_close_stress() {
    for _ in 0..stress_factor() {
        let (tx, rx) = channel::<i32>();
        let _t = thread::spawn(move || {
            drop(rx);
        });
        let _ = thread::spawn(move || {
            tx.send(1).unwrap();
        })
        .join();
    }
}

#[allow(dead_code)]
pub fn test_mpsc_oneshot_multi_thread_recv_close_stress() {
    for _ in 0..stress_factor() {
        let (tx, rx) = channel::<i32>();
        thread::spawn(move || {
            let res = thread::spawn(move || {
                rx.recv().unwrap();
            })
            .join();
            assert!(res.is_err());
        });
        let _t = thread::spawn(move || {
            thread::spawn(move || {
                drop(tx);
            });
        });
    }
}

pub fn test_mpsc_oneshot_multi_thread_send_recv_stress() {
    for _ in 0..stress_factor() {
        let (tx, rx) = channel::<Box<isize>>();
        let _t = thread::spawn(move || {
            tx.send(box 10).unwrap();
            thread::sleep_ms(60000);
        });
        assert!(*rx.recv().unwrap() == 10);
    }
}

pub fn test_mpsc_stream_send_recv_stress() {
    for _ in 0..stress_factor() {
        let (tx, rx) = channel();

        send(tx, 0);
        recv(rx, 0);

        fn send(tx: Sender<Box<i32>>, i: i32) {
            if i == 10 {
                return;
            }

            thread::spawn(move || {
                tx.send(box i).unwrap();
                send(tx, i + 1);
            });
        }

        fn recv(rx: Receiver<Box<i32>>, i: i32) {
            if i == 10 {
                return;
            }

            thread::spawn(move || {
                assert!(*rx.recv().unwrap() == i);
                recv(rx, i + 1);
            });
        }
    }
}

pub fn test_mpsc_oneshot_single_thread_recv_timeout() {
    let (tx, rx) = channel();
    tx.send(()).unwrap();
    assert_eq!(rx.recv_timeout(Duration::from_millis(1)), Ok(()));
    assert_eq!(
        rx.recv_timeout(Duration::from_millis(1)),
        Err(RecvTimeoutError::Timeout)
    );
    tx.send(()).unwrap();
    assert_eq!(rx.recv_timeout(Duration::from_millis(1)), Ok(()));
}

pub fn test_mpsc_stress_recv_timeout_two_threads() {
    let (tx, rx) = channel();
    let stress = stress_factor() + 3;
    let timeout = Duration::from_millis(100);

    thread::spawn(move || {
        for i in 0..stress {
            if i % 2 == 0 {
                thread::sleep(timeout * 2);
            }
            tx.send(1usize).unwrap();
        }
    });

    let mut recv_count = 0;
    loop {
        match rx.recv_timeout(timeout) {
            Ok(n) => {
                assert_eq!(n, 1usize);
                recv_count += 1;
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    assert_eq!(recv_count, stress);
}

pub fn test_mpsc_recv_timeout_upgrade() {
    let (tx, rx) = channel::<()>();
    let timeout = Duration::from_millis(1);
    let _tx_clone = tx.clone();

    let start = Instant::now();
    assert_eq!(rx.recv_timeout(timeout), Err(RecvTimeoutError::Timeout));
    assert!(Instant::now() >= start + timeout);
}

pub fn test_mpsc_stress_recv_timeout_shared() {
    let (tx, rx) = channel();
    let stress = stress_factor() + 3;

    for i in 0..stress {
        let tx = tx.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(i as u64 * 10));
            tx.send(1usize).unwrap();
        });
    }

    drop(tx);

    let mut recv_count = 0;
    loop {
        match rx.recv_timeout(Duration::from_millis(10)) {
            Ok(n) => {
                assert_eq!(n, 1usize);
                recv_count += 1;
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    assert_eq!(recv_count, stress);
}

pub fn test_mpsc_very_long_recv_timeout_wont_panic() {
    let (tx, rx) = channel::<()>();
    let join_handle = thread::spawn(move || rx.recv_timeout(Duration::from_secs(u64::MAX)));
    thread::sleep(Duration::from_secs(1));
    assert!(tx.send(()).is_ok());
    assert_eq!(join_handle.join().unwrap(), Ok(()));
}

pub fn test_mpsc_recv_a_lot() {
    // Regression test that we don't run out of stack in scheduler context
    let (tx, rx) = channel();
    for _ in 0..10000 {
        tx.send(()).unwrap();
    }
    for _ in 0..10000 {
        rx.recv().unwrap();
    }
}

pub fn test_mpsc_shared_recv_timeout() {
    let (tx, rx) = channel();
    let total = 5;
    for _ in 0..total {
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(()).unwrap();
        });
    }

    for _ in 0..total {
        rx.recv().unwrap();
    }

    assert_eq!(
        rx.recv_timeout(Duration::from_millis(1)),
        Err(RecvTimeoutError::Timeout)
    );
    tx.send(()).unwrap();
    assert_eq!(rx.recv_timeout(Duration::from_millis(1)), Ok(()));
}

pub fn test_mpsc_shared_chan_stress() {
    let (tx, rx) = channel();
    let total = stress_factor() + 3;
    for _ in 0..total {
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(()).unwrap();
        });
    }

    for _ in 0..total {
        rx.recv().unwrap();
    }
}
pub fn test_mpsc_test_nested_recv_iter() {
    let (tx, rx) = channel::<i32>();
    let (total_tx, total_rx) = channel::<i32>();

    let _t = thread::spawn(move || {
        let mut acc = 0;
        for x in rx.iter() {
            acc += x;
        }
        total_tx.send(acc).unwrap();
    });

    tx.send(3).unwrap();
    tx.send(1).unwrap();
    tx.send(2).unwrap();
    drop(tx);
    assert_eq!(total_rx.recv().unwrap(), 6);
}

pub fn test_mpsc_test_recv_iter_break() {
    let (tx, rx) = channel::<i32>();
    let (count_tx, count_rx) = channel();

    let _t = thread::spawn(move || {
        let mut count = 0;
        for x in rx.iter() {
            if count >= 3 {
                break;
            } else {
                count += x;
            }
        }
        count_tx.send(count).unwrap();
    });

    tx.send(2).unwrap();
    tx.send(2).unwrap();
    tx.send(2).unwrap();
    let _ = tx.send(2);
    drop(tx);
    assert_eq!(count_rx.recv().unwrap(), 4);
}

pub fn test_mpsc_test_recv_try_iter() {
    let (request_tx, request_rx) = channel();
    let (response_tx, response_rx) = channel();

    // Request `x`s until we have `6`.
    let t = thread::spawn(move || {
        let mut count = 0;
        loop {
            for x in response_rx.try_iter() {
                count += x;
                if count == 6 {
                    return count;
                }
            }
            request_tx.send(()).unwrap();
        }
    });

    for _ in request_rx.iter() {
        if response_tx.send(2).is_err() {
            break;
        }
    }

    assert_eq!(t.join().unwrap(), 6);
}

pub fn test_mpsc_test_recv_into_iter_owned() {
    let mut iter = {
        let (tx, rx) = channel::<i32>();
        tx.send(1).unwrap();
        tx.send(2).unwrap();

        rx.into_iter()
    };
    assert_eq!(iter.next().unwrap(), 1);
    assert_eq!(iter.next().unwrap(), 2);
    assert_eq!(iter.next().is_none(), true);
}

pub fn test_mpsc_try_recv_states() {
    let (tx1, rx1) = channel::<i32>();
    let (tx2, rx2) = channel::<()>();
    let (tx3, rx3) = channel::<()>();
    let _t = thread::spawn(move || {
        rx2.recv().unwrap();
        tx1.send(1).unwrap();
        tx3.send(()).unwrap();
        rx2.recv().unwrap();
        drop(tx1);
        tx3.send(()).unwrap();
    });

    assert_eq!(rx1.try_recv(), Err(TryRecvError::Empty));
    tx2.send(()).unwrap();
    rx3.recv().unwrap();
    assert_eq!(rx1.try_recv(), Ok(1));
    assert_eq!(rx1.try_recv(), Err(TryRecvError::Empty));
    tx2.send(()).unwrap();
    rx3.recv().unwrap();
    assert_eq!(rx1.try_recv(), Err(TryRecvError::Disconnected));
}

// This bug used to end up in a livelock inside of the Receiver destructor
// because the internal state of the Shared packet was corrupted
pub fn test_mpsc_destroy_upgraded_shared_port_when_sender_still_active() {
    let (tx, rx) = channel();
    let (tx2, rx2) = channel();
    let _t = thread::spawn(move || {
        rx.recv().unwrap(); // wait on a oneshot
        drop(rx); // destroy a shared
        tx2.send(()).unwrap();
    });
    // make sure the other thread has gone to sleep
    for _ in 0..5000 {
        thread::yield_now();
    }

    // upgrade to a shared chan and send a message
    let t = tx.clone();
    drop(tx);
    t.send(()).unwrap();

    // wait for the child thread to exit before we exit
    rx2.recv().unwrap();
}

pub fn test_mpsc_issue_32114() {
    let (tx, _) = channel();
    let _ = tx.send(123);
    assert_eq!(tx.send(123), Err(SendError(123)));
}

pub fn test_mpsc_sync_smoke() {
    let (tx, rx) = sync_channel::<i32>(1);
    tx.send(1).unwrap();
    assert_eq!(rx.recv().unwrap(), 1);
}

pub fn test_mpsc_sync_drop_full() {
    let (tx, _rx) = sync_channel::<Box<isize>>(1);
    tx.send(box 1).unwrap();
}

pub fn test_mpsc_sync_smoke_shared() {
    let (tx, rx) = sync_channel::<i32>(1);
    tx.send(1).unwrap();
    assert_eq!(rx.recv().unwrap(), 1);
    let tx = tx.clone();
    tx.send(1).unwrap();
    assert_eq!(rx.recv().unwrap(), 1);
}

pub fn test_mpsc_sync_recv_timeout() {
    let (tx, rx) = sync_channel::<i32>(1);
    assert_eq!(
        rx.recv_timeout(Duration::from_millis(1)),
        Err(RecvTimeoutError::Timeout)
    );
    tx.send(1).unwrap();
    assert_eq!(rx.recv_timeout(Duration::from_millis(1)), Ok(1));
}

pub fn test_mpsc_sync_smoke_threads() {
    let (tx, rx) = sync_channel::<i32>(0);
    let _t = thread::spawn(move || {
        tx.send(1).unwrap();
    });
    assert_eq!(rx.recv().unwrap(), 1);
}

pub fn test_mpsc_sync_smoke_port_gone() {
    let (tx, rx) = sync_channel::<i32>(0);
    drop(rx);
    assert!(tx.send(1).is_err());
}

pub fn test_mpsc_sync_smoke_shared_port_gone2() {
    let (tx, rx) = sync_channel::<i32>(0);
    drop(rx);
    let tx2 = tx.clone();
    drop(tx);
    assert!(tx2.send(1).is_err());
}

pub fn test_mpsc_sync_port_gone_concurrent() {
    let (tx, rx) = sync_channel::<i32>(0);
    let _t = thread::spawn(move || {
        rx.recv().unwrap();
    });
    while tx.send(1).is_ok() {}
}

pub fn test_mpsc_sync_port_gone_concurrent_shared() {
    let (tx, rx) = sync_channel::<i32>(0);
    let tx2 = tx.clone();
    let _t = thread::spawn(move || {
        rx.recv().unwrap();
    });
    while tx.send(1).is_ok() && tx2.send(1).is_ok() {}
}

pub fn test_mpsc_sync_smoke_chan_gone() {
    let (tx, rx) = sync_channel::<i32>(0);
    drop(tx);
    assert!(rx.recv().is_err());
}

pub fn test_mpsc_sync_smoke_chan_gone_shared() {
    let (tx, rx) = sync_channel::<()>(0);
    let tx2 = tx.clone();
    drop(tx);
    drop(tx2);
    assert!(rx.recv().is_err());
}

pub fn test_mpsc_sync_chan_gone_concurrent() {
    let (tx, rx) = sync_channel::<i32>(0);
    thread::spawn(move || {
        tx.send(1).unwrap();
        tx.send(1).unwrap();
    });
    while rx.recv().is_ok() {}
}

pub fn test_mpsc_sync_stress() {
    let (tx, rx) = sync_channel::<i32>(0);
    thread::spawn(move || {
        for _ in 0..10 {
            tx.send(1).unwrap();
        }
    });
    for _ in 0..10 {
        assert_eq!(rx.recv().unwrap(), 1);
    }
}

pub fn test_mpsc_sync_stress_recv_timeout_two_threads() {
    let (tx, rx) = sync_channel::<i32>(0);

    thread::spawn(move || {
        for _ in 0..10 {
            tx.send(1).unwrap();
        }
    });

    let mut recv_count = 0;
    loop {
        match rx.recv_timeout(Duration::from_millis(1)) {
            Ok(v) => {
                assert_eq!(v, 1);
                recv_count += 1;
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    assert_eq!(recv_count, 10);
}

pub fn test_mpsc_sync_stress_recv_timeout_shared() {
    const AMT: u32 = 2;
    const NTHREADS: u32 = 2;
    let (tx, rx) = sync_channel::<i32>(0);
    let (dtx, drx) = sync_channel::<()>(0);

    thread::spawn(move || {
        let mut recv_count = 0;
        loop {
            match rx.recv_timeout(Duration::from_millis(10)) {
                Ok(v) => {
                    assert_eq!(v, 1);
                    recv_count += 1;
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }

        assert_eq!(recv_count, AMT * NTHREADS);
        assert!(rx.try_recv().is_err());

        dtx.send(()).unwrap();
    });

    for _ in 0..NTHREADS {
        let tx = tx.clone();
        thread::spawn(move || {
            for _ in 0..AMT {
                tx.send(1).unwrap();
            }
        });
    }

    drop(tx);

    drx.recv().unwrap();
}

pub fn test_mpsc_sync_stress_shared() {
    const AMT: u32 = 2;
    const NTHREADS: u32 = 2;
    let (tx, rx) = sync_channel::<i32>(0);
    let (dtx, drx) = sync_channel::<()>(0);

    thread::spawn(move || {
        for _ in 0..AMT * NTHREADS {
            assert_eq!(rx.recv().unwrap(), 1);
        }
        match rx.try_recv() {
            Ok(..) => panic!(),
            _ => {}
        }
        dtx.send(()).unwrap();
    });

    for _ in 0..NTHREADS {
        let tx = tx.clone();
        thread::spawn(move || {
            for _ in 0..AMT {
                tx.send(1).unwrap();
            }
        });
    }
    drop(tx);
    drx.recv().unwrap();
}

pub fn test_mpsc_sync_oneshot_single_thread_close_port_first() {
    // Simple test of closing without sending
    let (_tx, rx) = sync_channel::<i32>(0);
    drop(rx);
}

pub fn test_mpsc_sync_oneshot_single_thread_send_port_close() {
    // Testing that the sender cleans up the payload if receiver is closed
    let (tx, rx) = sync_channel::<Box<i32>>(0);
    drop(rx);
    assert!(tx.send(box 0).is_err());
}

#[allow(dead_code)]
pub fn test_mpsc_sync_oneshot_single_thread_recv_chan_close() {
    // Receiving on a closed chan will panic
    let res = thread::spawn(move || {
        let (tx, rx) = sync_channel::<i32>(0);
        drop(tx);
        rx.recv().unwrap();
    })
    .join();
    // What is our res?
    assert!(res.is_err());
}

pub fn test_mpsc_sync_oneshot_single_thread_send_then_recv() {
    let (tx, rx) = sync_channel::<Box<i32>>(1);
    tx.send(box 10).unwrap();
    assert!(*rx.recv().unwrap() == 10);
}

pub fn test_mpsc_sync_oneshot_single_thread_try_send_open() {
    let (tx, rx) = sync_channel::<i32>(1);
    assert_eq!(tx.try_send(10), Ok(()));
    assert!(rx.recv().unwrap() == 10);
}

pub fn test_mpsc_sync_oneshot_single_thread_try_send_closed() {
    let (tx, rx) = sync_channel::<i32>(0);
    drop(rx);
    assert_eq!(tx.try_send(10), Err(TrySendError::Disconnected(10)));
}

pub fn test_mpsc_sync_oneshot_single_thread_try_send_closed2() {
    let (tx, _rx) = sync_channel::<i32>(0);
    assert_eq!(tx.try_send(10), Err(TrySendError::Full(10)));
}

pub fn test_mpsc_sync_oneshot_single_thread_try_recv_open() {
    let (tx, rx) = sync_channel::<i32>(1);
    tx.send(10).unwrap();
    assert!(rx.recv() == Ok(10));
}

pub fn test_mpsc_sync_oneshot_single_thread_try_recv_closed() {
    let (tx, rx) = sync_channel::<i32>(0);
    drop(tx);
    assert!(rx.recv().is_err());
}

pub fn test_mpsc_sync_oneshot_single_thread_try_recv_closed_with_data() {
    let (tx, rx) = sync_channel::<i32>(1);
    tx.send(10).unwrap();
    drop(tx);
    assert_eq!(rx.try_recv(), Ok(10));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
}

pub fn test_mpsc_sync_oneshot_single_thread_peek_data() {
    let (tx, rx) = sync_channel::<i32>(1);
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
    tx.send(10).unwrap();
    assert_eq!(rx.try_recv(), Ok(10));
}

pub fn test_mpsc_sync_oneshot_single_thread_peek_close() {
    let (tx, rx) = sync_channel::<i32>(0);
    drop(tx);
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
    assert_eq!(rx.try_recv(), Err(TryRecvError::Disconnected));
}

pub fn test_mpsc_sync_oneshot_single_thread_peek_open() {
    let (_tx, rx) = sync_channel::<i32>(0);
    assert_eq!(rx.try_recv(), Err(TryRecvError::Empty));
}

pub fn test_mpsc_sync_oneshot_multi_task_recv_then_send() {
    let (tx, rx) = sync_channel::<Box<i32>>(0);
    let _t = thread::spawn(move || {
        assert!(*rx.recv().unwrap() == 10);
    });

    tx.send(box 10).unwrap();
}

#[allow(dead_code)]
pub fn test_mpsc_sync_oneshot_multi_task_recv_then_close() {
    let (tx, rx) = sync_channel::<Box<i32>>(0);
    let _t = thread::spawn(move || {
        drop(tx);
    });
    let res = thread::spawn(move || {
        assert!(*rx.recv().unwrap() == 10);
    })
    .join();
    assert!(res.is_err());
}

pub fn test_mpsc_sync_oneshot_multi_thread_close_stress() {
    for _ in 0..stress_factor() {
        let (tx, rx) = sync_channel::<i32>(0);
        let _t = thread::spawn(move || {
            drop(rx);
        });
        drop(tx);
    }
}

#[allow(dead_code)]
pub fn test_mpsc_sync_oneshot_multi_thread_send_close_stress() {
    for _ in 0..stress_factor() {
        let (tx, rx) = sync_channel::<i32>(0);
        let _t = thread::spawn(move || {
            drop(rx);
        });
        let _ = thread::spawn(move || {
            tx.send(1).unwrap();
        })
        .join();
    }
}

#[allow(dead_code)]
pub fn test_mpsc_sync_oneshot_multi_thread_recv_close_stress() {
    for _ in 0..stress_factor() {
        let (tx, rx) = sync_channel::<i32>(0);
        let _t = thread::spawn(move || {
            let res = thread::spawn(move || {
                rx.recv().unwrap();
            })
            .join();
            assert!(res.is_err());
        });
        let _t = thread::spawn(move || {
            thread::spawn(move || {
                drop(tx);
            });
        });
    }
}

pub fn test_mpsc_sync_oneshot_multi_thread_send_recv_stress() {
    for _ in 0..stress_factor() {
        let (tx, rx) = sync_channel::<Box<i32>>(0);
        let _t = thread::spawn(move || {
            tx.send(box 10).unwrap();
        });
        assert!(*rx.recv().unwrap() == 10);
    }
}

pub fn test_mpsc_sync_stream_send_recv_stress() {
    for _ in 0..stress_factor() {
        let (tx, rx) = sync_channel::<Box<i32>>(0);

        send(tx, 0);
        recv(rx, 0);

        fn send(tx: SyncSender<Box<i32>>, i: i32) {
            if i == 10 {
                return;
            }

            thread::spawn(move || {
                tx.send(box i).unwrap();
                send(tx, i + 1);
            });
        }

        fn recv(rx: Receiver<Box<i32>>, i: i32) {
            if i == 10 {
                return;
            }

            thread::spawn(move || {
                assert!(*rx.recv().unwrap() == i);
                recv(rx, i + 1);
            });
        }
    }
}

pub fn test_mpsc_sync_recv_a_lot() {
    // Regression test that we don't run out of stack in scheduler context
    let (tx, rx) = sync_channel(10000);
    for _ in 0..10000 {
        tx.send(()).unwrap();
    }
    for _ in 0..10000 {
        rx.recv().unwrap();
    }
}

pub fn test_mpsc_sync_shared_sync_chan_stress() {
    let (tx, rx) = sync_channel(0);
    let total = stress_factor() + 2;
    for _ in 0..total {
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(()).unwrap();
        });
    }

    for _ in 0..total {
        rx.recv().unwrap();
    }
}

pub fn test_mpsc_sync_test_nested_recv_iter() {
    let (tx, rx) = sync_channel::<i32>(0);
    let (total_tx, total_rx) = sync_channel::<i32>(0);

    let _t = thread::spawn(move || {
        let mut acc = 0;
        for x in rx.iter() {
            acc += x;
        }
        total_tx.send(acc).unwrap();
    });

    tx.send(3).unwrap();
    tx.send(1).unwrap();
    tx.send(2).unwrap();
    drop(tx);
    assert_eq!(total_rx.recv().unwrap(), 6);
}
pub fn test_mpsc_sync_test_recv_iter_break() {
    let (tx, rx) = sync_channel::<i32>(0);
    let (count_tx, count_rx) = sync_channel(0);

    let _t = thread::spawn(move || {
        let mut count = 0;
        for x in rx.iter() {
            if count >= 3 {
                break;
            } else {
                count += x;
            }
        }
        count_tx.send(count).unwrap();
    });

    tx.send(2).unwrap();
    tx.send(2).unwrap();
    tx.send(2).unwrap();
    let _ = tx.try_send(2);
    drop(tx);
    assert_eq!(count_rx.recv().unwrap(), 4);
}

pub fn test_mpsc_sync_try_recv_states() {
    let (tx1, rx1) = sync_channel::<i32>(1);
    let (tx2, rx2) = sync_channel::<()>(1);
    let (tx3, rx3) = sync_channel::<()>(1);
    let _t = thread::spawn(move || {
        rx2.recv().unwrap();
        tx1.send(1).unwrap();
        tx3.send(()).unwrap();
        rx2.recv().unwrap();
        drop(tx1);
        tx3.send(()).unwrap();
    });

    assert_eq!(rx1.try_recv(), Err(TryRecvError::Empty));
    tx2.send(()).unwrap();
    rx3.recv().unwrap();
    assert_eq!(rx1.try_recv(), Ok(1));
    assert_eq!(rx1.try_recv(), Err(TryRecvError::Empty));
    tx2.send(()).unwrap();
    rx3.recv().unwrap();
    assert_eq!(rx1.try_recv(), Err(TryRecvError::Disconnected));
}

// This bug used to end up in a livelock inside of the Receiver destructor
// because the internal state of the Shared packet was corrupted
pub fn test_mpsc_sync_destroy_upgraded_shared_port_when_sender_still_active() {
    let (tx, rx) = sync_channel::<()>(0);
    let (tx2, rx2) = sync_channel::<()>(0);
    let _t = thread::spawn(move || {
        rx.recv().unwrap(); // wait on a oneshot
        drop(rx); // destroy a shared
        tx2.send(()).unwrap();
    });
    // make sure the other thread has gone to sleep
    for _ in 0..5000 {
        thread::yield_now();
    }

    // upgrade to a shared chan and send a message
    let t = tx.clone();
    drop(tx);
    t.send(()).unwrap();

    // wait for the child thread to exit before we exit
    rx2.recv().unwrap();
}

pub fn test_mpsc_sync_send1() {
    let (tx, rx) = sync_channel::<i32>(0);
    let _t = thread::spawn(move || {
        rx.recv().unwrap();
    });
    assert_eq!(tx.send(1), Ok(()));
}

pub fn test_mpsc_sync_send2() {
    let (tx, rx) = sync_channel::<i32>(0);
    let _t = thread::spawn(move || {
        drop(rx);
    });
    assert!(tx.send(1).is_err());
}

pub fn test_mpsc_sync_send3() {
    let (tx, rx) = sync_channel::<i32>(1);
    assert_eq!(tx.send(1), Ok(()));
    let _t = thread::spawn(move || {
        drop(rx);
    });
    assert!(tx.send(1).is_err());
}

pub fn test_mpsc_sync_send4() {
    let (tx, rx) = sync_channel::<i32>(0);
    let tx2 = tx.clone();
    let (done, donerx) = channel();
    let done2 = done.clone();
    let _t = thread::spawn(move || {
        assert!(tx.send(1).is_err());
        done.send(()).unwrap();
    });
    let _t = thread::spawn(move || {
        assert!(tx2.send(2).is_err());
        done2.send(()).unwrap();
    });
    drop(rx);
    donerx.recv().unwrap();
    donerx.recv().unwrap();
}

pub fn test_mpsc_sync_try_send1() {
    let (tx, _rx) = sync_channel::<i32>(0);
    assert_eq!(tx.try_send(1), Err(TrySendError::Full(1)));
}

pub fn test_mpsc_sync_try_send2() {
    let (tx, _rx) = sync_channel::<i32>(1);
    assert_eq!(tx.try_send(1), Ok(()));
    assert_eq!(tx.try_send(1), Err(TrySendError::Full(1)));
}

pub fn test_mpsc_sync_try_send3() {
    let (tx, rx) = sync_channel::<i32>(1);
    assert_eq!(tx.try_send(1), Ok(()));
    drop(rx);
    assert_eq!(tx.try_send(1), Err(TrySendError::Disconnected(1)));
}

pub fn test_mpsc_sync_issue_15761() {
    fn repro() {
        let (tx1, rx1) = sync_channel::<()>(3);
        let (tx2, rx2) = sync_channel::<()>(3);

        let _t = thread::spawn(move || {
            rx1.recv().unwrap();
            tx2.try_send(()).unwrap();
        });

        tx1.try_send(()).unwrap();
        rx2.recv().unwrap();
    }

    for _ in 0..5 {
        repro()
    }
}
