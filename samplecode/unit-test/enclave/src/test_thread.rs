use std::any::Any;
use std::boxed::Box;
use std::mem;
use std::panic;
use std::result;
use std::string::String;
use std::string::ToString;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::thread::sleep;
use std::thread::Builder;
use std::thread::ThreadId;
use std::time::Duration;
use std::u32;

pub fn test_thread_unnamed_thread() {
    thread::spawn(move || {
        assert!(thread::current().name().is_none());
    })
    .join()
    .ok()
    .expect("thread panicked");
}

pub fn test_thread_named_thread() {
    Builder::new()
        .name("ada lovelace".to_string())
        .spawn(move || {
            assert!(thread::current().name().unwrap() == "ada lovelace".to_string());
        })
        .unwrap()
        .join()
        .unwrap();
}

pub fn test_thread_run_basic() {
    let (tx, rx) = channel();
    thread::spawn(move || {
        tx.send(()).unwrap();
    });
    rx.recv().unwrap();
}

pub fn test_thread_spawn_sched() {
    let (tx, rx) = channel();
    fn f(i: i32, tx: Sender<()>) {
        let tx = tx.clone();
        thread::spawn(move || {
            if i == 0 {
                tx.send(()).unwrap();
            } else {
                f(i - 1, tx);
            }
        });
    }
    f(9, tx);
    rx.recv().unwrap();
}

pub fn test_thread_spawn_sched_childs_on_default_sched() {
    let (tx, rx) = channel();

    thread::spawn(move || {
        thread::spawn(move || {
            tx.send(()).unwrap();
        });
    });
    rx.recv().unwrap();
}

fn avoid_copying_the_body<F>(spawnfn: F)
where
    F: FnOnce(Box<dyn Fn() + Send>),
{
    let (tx, rx) = channel();

    let x: Box<_> = Box::new(1);
    let x_in_parent = (&*x) as *const i32 as usize;

    spawnfn(Box::new(move || {
        let x_in_child = (&*x) as *const i32 as usize;
        tx.send(x_in_child).unwrap();
    }));

    let x_in_child = rx.recv().unwrap();
    assert_eq!(x_in_parent, x_in_child);
}

pub fn test_thread_test_avoid_copying_the_body_spawn() {
    avoid_copying_the_body(|v| {
        thread::spawn(move || v());
    });
}

pub fn test_thread_test_avoid_copying_the_body_thread_spawn() {
    avoid_copying_the_body(|f| {
        thread::spawn(move || {
            f();
        });
    })
}

pub fn test_thread_test_avoid_copying_the_body_join() {
    avoid_copying_the_body(|f| {
        let _ = thread::spawn(move || f()).join();
    })
}

pub fn test_thread_invalid_named_thread() {
    should_panic!(Builder::new()
        .name("ada l\0velace".to_string())
        .spawn(|| {}));
}

pub fn test_thread_join_panic() {
    match thread::spawn(move || panic!()).join() {
        result::Result::Err(_) => (),
        result::Result::Ok(()) => panic!(),
    }
}

pub fn test_thread_child_doesnt_ref_parent() {
    // If the child refcounts the parent thread, this will stack overflow when
    // climbing the thread tree to dereference each ancestor. (See #1789)
    // (well, it would if the constant were 8000+ - I lowered it to be more
    // valgrind-friendly. try this at home, instead..!)
    const GENERATIONS: u32 = 16;
    fn child_no(x: u32) -> Box<dyn Fn() + Send> {
        return Box::new(move || {
            if x < GENERATIONS {
                thread::spawn(move || child_no(x + 1)());
            }
        });
    }
    thread::spawn(|| child_no(0)());
}

pub fn test_thread_simple_newsched_spawn() {
    let j = thread::spawn(move || println!("test"));
    let _res = j.join();
}

pub fn test_thread_try_panic_message_static_str() {
    match thread::spawn(move || {
        panic!("static string");
    })
    .join()
    {
        Err(e) => {
            type T = &'static str;
            assert!(e.is::<T>());
            assert_eq!(*e.downcast::<T>().unwrap(), "static string");
        }
        Ok(()) => panic!(),
    }
}

pub fn test_thread_try_panic_message_owned_str() {
    match thread::spawn(move || {
        panic::panic_any("owned string".to_string());
    })
    .join()
    {
        Err(e) => {
            type T = String;
            assert!(e.is::<T>());
            assert_eq!(*e.downcast::<T>().unwrap(), "owned string".to_string());
        }
        Ok(()) => panic!(),
    }
}

pub fn test_thread_try_panic_message_any() {
    match thread::spawn(move || {
        panic::panic_any(Box::new(413u16) as Box<dyn Any + Send>);
    })
    .join()
    {
        Err(e) => {
            type T = Box<dyn Any + Send>;
            assert!(e.is::<T>());
            let any = e.downcast::<T>().unwrap();
            assert!(any.is::<u16>());
            assert_eq!(*any.downcast::<u16>().unwrap(), 413);
        }
        Ok(()) => panic!(),
    }
}
pub fn test_thread_try_panic_message_unit_struct() {
    struct Juju;
    match thread::spawn(move || panic::panic_any(Juju)).join() {
        Err(ref e) if e.is::<Juju>() => {}
        Err(_) | Ok(()) => panic!(),
    }
}

pub fn test_thread_park_timeout_unpark_before() {
    for _ in 0..8 {
        thread::current().unpark();
        thread::park_timeout(Duration::from_millis(u32::MAX as u64));
    }
}

pub fn test_thread_park_timeout_unpark_not_called() {
    for i in 0..8 {
        println!("{:?} timeout in ", i);
        thread::park_timeout(Duration::from_millis(1000));
        println!("{:?} timeout", i);
    }
}

pub fn test_thread_park_timeout_unpark_called_other_thread() {
    for _ in 0..8 {
        let th = thread::current();

        let _guard = thread::spawn(move || {
            sleep(Duration::from_millis(50));
            println!("unpark");
            th.unpark();
        });
        thread::park_timeout(Duration::from_millis(u32::MAX as u64));
    }
}

pub fn test_thread_sleep_ms_smoke() {
    thread::sleep(Duration::from_millis(10));
}

pub fn test_thread_size_of_option_thread_id() {
    assert_eq!(
        mem::size_of::<Option<ThreadId>>(),
        mem::size_of::<ThreadId>()
    );
}

pub fn test_thread_id_equal() {
    assert!(thread::current().id() == thread::current().id());
}

pub fn test_thread_id_not_equal() {
    let spawned_id = thread::spawn(|| thread::current().id()).join().unwrap();
    assert!(thread::current().id() != spawned_id);
}
