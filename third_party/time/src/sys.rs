#![allow(bad_style)]

pub use self::inner::*;
use sgx_trts::libc::{time_t, c_long, c_int, c_ulong, c_char};

const EPOCH_ADJUSTMENT_DAYS: c_long = 719468;
const ADJUSTED_EPOCH_YEAR: c_int = 0;
const ADJUSTED_EPOCH_WDAY: c_long = 3;
const DAYS_PER_ERA: c_long = (400 - 97) * 365 + 97 * 366;
const DAYS_PER_CENTURY: c_ulong = (100 - 24) * 365 + 24 * 366;
const DAYS_PER_4_YEARS: c_ulong = 3 * 365 + 366;
const DAYS_PER_YEAR: c_int = 365;
const DAYS_IN_JANUARY: c_int = 31;
const DAYS_IN_FEBRUARY: c_int = 28;
const YEARS_PER_ERA: c_int = 400;

const SECSPERMIN: c_long = 60;
const MINSPERHOUR: c_long = 60;
const HOURSPERDAY: c_long = 24;
const SECSPERHOUR: c_long = SECSPERMIN * MINSPERHOUR;
const SECSPERDAY: c_long = SECSPERHOUR * HOURSPERDAY;
const DAYSPERWEEK: c_int = 7;

const YEAR_BASE: c_int = 1900;

const UTC: *const c_char = b"UTC\0" as *const u8 as *const c_char;

struct relibc_tm {
    pub tm_sec: c_int,
    pub tm_min: c_int,
    pub tm_hour: c_int,
    pub tm_mday: c_int,
    pub tm_mon: c_int,
    pub tm_year: c_int,
    pub tm_wday: c_int,
    pub tm_yday: c_int,
    pub tm_isdst: c_int,
    pub tm_gmtoff: c_long,
    pub tm_zone: *const c_char,
}

const empty_tm : relibc_tm = relibc_tm {
    tm_sec: 0,
    tm_min: 0,
    tm_hour: 0,
    tm_mday: 0,
    tm_mon: 0,
    tm_year: 0,
    tm_wday: 0,
    tm_yday: 0,
    tm_isdst: 0,
    tm_gmtoff: 0,
    tm_zone: UTC,
};

fn civil_from_days(days: c_long) -> (c_int, c_int, c_int, c_int) {
    let (era, year): (c_int, c_int);
    let (erayear, mut yearday, mut month, day): (c_int, c_int, c_int, c_int);
    let eraday: c_ulong;

    era = (if days >= 0 {
        days
    } else {
        days - (DAYS_PER_ERA - 1)
    } / DAYS_PER_ERA) as c_int;
    eraday = (days - era as c_long * DAYS_PER_ERA) as c_ulong;
    let a = eraday / (DAYS_PER_4_YEARS - 1);
    let b = eraday / DAYS_PER_CENTURY;
    let c = eraday / (DAYS_PER_ERA as c_ulong - 1);
    erayear = ((eraday - a + b - c) / 365) as c_int;
    let d = DAYS_PER_YEAR * erayear + erayear / 4 - erayear / 100;
    yearday = (eraday - d as c_ulong) as c_int;
    month = (5 * yearday + 2) / 153;
    day = yearday - (153 * month + 2) / 5 + 1;
    month += if month < 10 { 2 } else { -10 };
    year = ADJUSTED_EPOCH_YEAR + erayear + era * YEARS_PER_ERA + (month <= 1) as c_int;
    yearday += if yearday >= DAYS_PER_YEAR - DAYS_IN_JANUARY - DAYS_IN_FEBRUARY {
        -(DAYS_PER_YEAR - DAYS_IN_JANUARY - DAYS_IN_FEBRUARY)
    } else {
        DAYS_IN_JANUARY + DAYS_IN_FEBRUARY + is_leap(erayear)
    };
    return (year, month, day, yearday);
}

#[inline(always)]
fn is_leap(y: c_int) -> c_int {
    ((y % 4 == 0 && y % 100 != 0) || y % 400 == 0) as c_int
}

fn gmtime(timer: time_t) -> relibc_tm {

    let (mut days, mut rem): (c_long, c_long);
    let mut weekday: c_int;
    let lcltime = timer;

    let mut result : relibc_tm = empty_tm;
    days = lcltime / SECSPERDAY + EPOCH_ADJUSTMENT_DAYS;
    rem = lcltime % SECSPERDAY;
    if rem < 0 {
        rem += SECSPERDAY;
        days -= 1;
    }
    result.tm_hour = (rem / SECSPERHOUR) as c_int;
    rem %= SECSPERHOUR;
    result.tm_min = (rem / SECSPERMIN) as c_int;
    result.tm_sec = (rem % SECSPERMIN) as c_int;

    weekday = ((ADJUSTED_EPOCH_WDAY + days) % DAYSPERWEEK as c_long) as c_int;
    if weekday < 0 {
        weekday += DAYSPERWEEK;
    }
    result.tm_wday = weekday;

    let (year, month, day, yearday) = civil_from_days(days);
    result.tm_yday = yearday;
    result.tm_year = year - YEAR_BASE;
    result.tm_mon = month;
    result.tm_mday = day;

    result.tm_isdst = 0;
    result.tm_gmtoff = 0;
    result.tm_zone = UTC;
    result
}

mod inner {
    use Tm;

    pub use self::unix::*;
    use sys::relibc_tm;
    use sys::gmtime;

    //fn rust_tm_to_tm(rust_tm: &Tm, tm: &mut libc::tm) {
    //    tm.tm_sec = rust_tm.tm_sec;
    //    tm.tm_min = rust_tm.tm_min;
    //    tm.tm_hour = rust_tm.tm_hour;
    //    tm.tm_mday = rust_tm.tm_mday;
    //    tm.tm_mon = rust_tm.tm_mon;
    //    tm.tm_year = rust_tm.tm_year;
    //    tm.tm_wday = rust_tm.tm_wday;
    //    tm.tm_yday = rust_tm.tm_yday;
    //    tm.tm_isdst = rust_tm.tm_isdst;
    //}

    fn tm_to_rust_tm(tm: &relibc_tm, gmtoff: i32, rust_tm: &mut Tm) {
        rust_tm.tm_sec = tm.tm_sec;
        rust_tm.tm_min = tm.tm_min;
        rust_tm.tm_hour = tm.tm_hour;
        rust_tm.tm_mday = tm.tm_mday;
        rust_tm.tm_mon = tm.tm_mon;
        rust_tm.tm_year = tm.tm_year;
        rust_tm.tm_wday = tm.tm_wday;
        rust_tm.tm_yday = tm.tm_yday;
        rust_tm.tm_isdst = tm.tm_isdst;
        rust_tm.tm_utcoff = gmtoff;
    }

    pub fn time_to_utc_tm(sec: i64, tm: &mut Tm) {
        //unsafe {
        //    let sec = sec as time_t;
        //    let mut out = mem::zeroed();
        //    if libc::gmtime_r(&sec, &mut out).is_null() {
        //        panic!("gmtime_r failed: {}", io::Error::last_os_error());
        //    }
        //    tm_to_rust_tm(&out, 0, tm);
        //}
        let rtm: relibc_tm = gmtime(sec);
        let gmtoff = rtm.tm_gmtoff;
        tm_to_rust_tm(&rtm,gmtoff as i32,tm);
    }

    pub fn time_to_local_tm(sec: i64, tm: &mut Tm) {
        //unsafe {
        //    let sec = sec as time_t;
        //    let mut out = mem::zeroed();
        //    if libc::localtime_r(&sec, &mut out).is_null() {
        //        panic!("localtime_r failed: {}", io::Error::last_os_error());
        //    }

        //    let gmtoff = out.tm_gmtoff;
        //    tm_to_rust_tm(&out, gmtoff as i32, tm);
        //}
        let rtm: relibc_tm = gmtime(sec);
        let gmtoff = rtm.tm_gmtoff;
        tm_to_rust_tm(&rtm,gmtoff as i32,tm);
    }

    pub fn utc_tm_to_time(_rust_tm: &Tm) -> i64 {
        //#[cfg(not(any(all(target_os = "android", target_pointer_width = "32"), target_os = "nacl", target_os = "solaris")))]
        //use libc::timegm;

        //let mut tm = unsafe { mem::zeroed() };
        //rust_tm_to_tm(rust_tm, &mut tm);
        //unsafe { timegm(&mut tm) as i64 }
        println!("utc_tm_to_time unsupported due to timegm unsupported");
        unimplemented!();
    }

    pub fn local_tm_to_time(_rust_tm: &Tm) -> i64 {
        //let mut tm = unsafe { mem::zeroed() };
        //rust_tm_to_tm(rust_tm, &mut tm);
        //unsafe { libc::mktime(&mut tm) as i64 }
        println!("local_tm_to_time unsupported due to timegm unsupported");
        unimplemented!();
    }

    #[cfg(test)]
    pub struct TzReset;

    #[cfg(test)]
    pub fn set_los_angeles_time_zone() -> TzReset {
        use std::env;
        env::set_var("TZ", "America/Los_Angeles");
        ::tzset();
        TzReset
    }

    #[cfg(test)]
    pub fn set_london_with_dst_time_zone() -> TzReset {
        use std::env;
        env::set_var("TZ", "Europe/London");
        ::tzset();
        TzReset
    }

    mod unix {
        use std::fmt;
        use std::cmp::Ordering;
        use std::ops::{Add, Sub};
        use sgx_trts::libc;
        use std::untrusted::time::{InstantEx, SystemTimeEx};
        use std::time;

        use Duration;

        pub fn get_time() -> (i64, i32) {
            //let mut tv = libc::timespec { tv_sec: 0, tv_nsec: 0 };
            //unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, &mut tv); }
            let st = time::SystemTime::now().get_tup();
            (st.0, st.1 as i32)
        }

        pub fn get_precise_ns() -> u64 {
            //let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
            //unsafe {
            //    libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts);
            //}
            let st = time::SystemTime::now().get_tup();
            st.0 as u64 * 1000000000 + st.1 as u64
        }

        #[derive(Copy)]
        pub struct SteadyTime {
            t: libc::timespec,
        }

        impl fmt::Debug for SteadyTime {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "SteadyTime {{ tv_sec: {:?}, tv_nsec: {:?} }}",
                       self.t.tv_sec, self.t.tv_nsec)
            }
        }

        impl Clone for SteadyTime {
            fn clone(&self) -> SteadyTime {
                SteadyTime { t: self.t }
            }
        }

        impl SteadyTime {
            pub fn now() -> SteadyTime {
                //let mut t = SteadyTime {
                //    t: libc::timespec {
                //        tv_sec: 0,
                //        tv_nsec: 0,
                //    }
                //};
                //unsafe {
                //    assert_eq!(0, libc::clock_gettime(libc::CLOCK_MONOTONIC,
                //                                      &mut t.t));
                //}
                let (sec,nsec) = time::Instant::now().get_tup();
                let t = SteadyTime {
                    t: libc::timespec {
                        tv_sec: sec,
                        tv_nsec: nsec,
                    },
                };
                t
            }
        }

        impl Sub for SteadyTime {
            type Output = Duration;
            fn sub(self, other: SteadyTime) -> Duration {
                if self.t.tv_nsec >= other.t.tv_nsec {
                    Duration::seconds(self.t.tv_sec as i64 - other.t.tv_sec as i64) +
                        Duration::nanoseconds(self.t.tv_nsec as i64 - other.t.tv_nsec as i64)
                } else {
                    Duration::seconds(self.t.tv_sec as i64 - 1 - other.t.tv_sec as i64) +
                        Duration::nanoseconds(self.t.tv_nsec as i64 + ::NSEC_PER_SEC as i64 -
                                              other.t.tv_nsec as i64)
                }
            }
        }

        impl Sub<Duration> for SteadyTime {
            type Output = SteadyTime;
            fn sub(self, other: Duration) -> SteadyTime {
                self + -other
            }
        }

        impl Add<Duration> for SteadyTime {
            type Output = SteadyTime;
            fn add(mut self, other: Duration) -> SteadyTime {
                let seconds = other.num_seconds();
                let nanoseconds = other - Duration::seconds(seconds);
                let nanoseconds = nanoseconds.num_nanoseconds().unwrap();

                type nsec = libc::c_long;

                self.t.tv_sec += seconds as libc::time_t;
                self.t.tv_nsec += nanoseconds as nsec;
                if self.t.tv_nsec >= ::NSEC_PER_SEC as nsec {
                    self.t.tv_nsec -= ::NSEC_PER_SEC as nsec;
                    self.t.tv_sec += 1;
                } else if self.t.tv_nsec < 0 {
                    self.t.tv_sec -= 1;
                    self.t.tv_nsec += ::NSEC_PER_SEC as nsec;
                }
                self
            }
        }

        impl PartialOrd for SteadyTime {
            fn partial_cmp(&self, other: &SteadyTime) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for SteadyTime {
            fn cmp(&self, other: &SteadyTime) -> Ordering {
                match self.t.tv_sec.cmp(&other.t.tv_sec) {
                    Ordering::Equal => self.t.tv_nsec.cmp(&other.t.tv_nsec),
                    ord => ord
                }
            }
        }

        impl PartialEq for SteadyTime {
            fn eq(&self, other: &SteadyTime) -> bool {
                self.t.tv_sec == other.t.tv_sec &&
                    self.t.tv_nsec == other.t.tv_nsec
            }
        }

        impl Eq for SteadyTime {}

    }
}
