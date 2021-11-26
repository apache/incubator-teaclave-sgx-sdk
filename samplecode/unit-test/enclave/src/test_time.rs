use std::time::*;
use std::untrusted::time::{InstantEx, SystemTimeEx};

pub fn test_std_time() {
    macro_rules! assert_almost_eq {
        ($a:expr, $b:expr) => {{
            let (a, b) = ($a, $b);
            if a != b {
                let (a, b) = if a > b { (a, b) } else { (b, a) };
                assert!(a - Duration::new(0, 100) <= b);
            }
        }};
    }

    {
        let a = Instant::now();
        let b = Instant::now();
        assert!(b >= a);
    }

    {
        let a = Instant::now();
        a.elapsed();
    }

    {
        let a = Instant::now();
        let b = Instant::now();
        let dur = b.duration_since(a);
        assert_almost_eq!(b - dur, a);
        assert_almost_eq!(a + dur, b);

        let second = Duration::new(1, 0);
        assert_almost_eq!(a - second + second, a);
    }

    {
        let a = Instant::now();
        should_panic!((a - Duration::new(1, 0)).duration_since(a));
    }

    {
        let a = SystemTime::now();
        let b = SystemTime::now();
        match b.duration_since(a) {
            Ok(dur) if dur == Duration::new(0, 0) => {
                assert_almost_eq!(a, b);
            }
            Ok(dur) => {
                assert!(b > a);
                assert_almost_eq!(b - dur, a);
                assert_almost_eq!(a + dur, b);
            }
            Err(dur) => {
                let dur = dur.duration();
                assert!(a > b);
                assert_almost_eq!(b + dur, a);
                assert_almost_eq!(a - dur, b);
            }
        }

        let second = Duration::new(1, 0);
        assert_almost_eq!(a.duration_since(a - second).unwrap(), second);
        assert_almost_eq!(a.duration_since(a + second).unwrap_err().duration(), second);

        assert_almost_eq!(a - second + second, a);

        // A difference of 80 and 800 years cannot fit inside a 32-bit time_t
        //if !(cfg!(unix) && ::mem::size_of::<::libc::time_t>() <= 4) {
        //    let eighty_years = second * 60 * 60 * 24 * 365 * 80;
        //    assert_almost_eq!(a - eighty_years + eighty_years, a);
        //    assert_almost_eq!(a - (eighty_years * 10) + (eighty_years * 10), a);
        //}

        let one_second_from_epoch = UNIX_EPOCH + Duration::new(1, 0);
        let one_second_from_epoch2 =
            UNIX_EPOCH + Duration::new(0, 500_000_000) + Duration::new(0, 500_000_000);
        assert_eq!(one_second_from_epoch, one_second_from_epoch2);
    }

    {
        let a = SystemTime::now();
        drop(a.elapsed());
    }

    {
        let ts = SystemTime::now();
        let a = ts.duration_since(UNIX_EPOCH).unwrap();
        let b = ts.duration_since(UNIX_EPOCH - Duration::new(1, 0)).unwrap();
        assert!(b > a);
        assert_eq!(b - a, Duration::new(1, 0));

        let thirty_years = Duration::new(1, 0) * 60 * 60 * 24 * 365 * 30;

        // Right now for CI this test is run in an emulator, and apparently the
        // aarch64 emulator's sense of time is that we're still living in the
        // 70s.
        //
        // Otherwise let's assume that we're all running computers later than
        // 2000.
        if !cfg!(target_arch = "aarch64") {
            assert!(a > thirty_years);
        }

        // let's assume that we're all running computers earlier than 2090.
        // Should give us ~70 years to fix this!
        let hundred_twenty_years = thirty_years * 4;
        assert!(a < hundred_twenty_years);
    }
}
