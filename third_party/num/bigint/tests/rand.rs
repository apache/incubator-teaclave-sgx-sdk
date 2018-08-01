#![cfg(feature = "rand")]

extern crate num_bigint;
extern crate num_traits;
extern crate rand;

mod biguint {
    use num_bigint::{BigUint, RandBigInt, RandomBits};
    use num_traits::Zero;
    use rand::thread_rng;
    use rand::Rng;
    use rand::distributions::Uniform;

    #[test]
    fn test_rand() {
        let mut rng = thread_rng();
        let n: BigUint = rng.gen_biguint(137);
        assert!(n.bits() <= 137);
        assert!(rng.gen_biguint(0).is_zero());
    }

    #[test]
    fn test_rand_bits() {
        let mut rng = thread_rng();
        let n: BigUint = rng.sample(&RandomBits::new(137));
        assert!(n.bits() <= 137);
        let z: BigUint = rng.sample(&RandomBits::new(0));
        assert!(z.is_zero());
    }

    #[test]
    fn test_rand_range() {
        let mut rng = thread_rng();

        for _ in 0..10 {
            assert_eq!(
                rng.gen_biguint_range(&BigUint::from(236u32), &BigUint::from(237u32)),
                BigUint::from(236u32)
            );
        }

        let l = BigUint::from(403469000u32 + 2352);
        let u = BigUint::from(403469000u32 + 3513);
        for _ in 0..1000 {
            let n: BigUint = rng.gen_biguint_below(&u);
            assert!(n < u);

            let n: BigUint = rng.gen_biguint_range(&l, &u);
            assert!(n >= l);
            assert!(n < u);
        }
    }

    #[test]
    #[should_panic]
    fn test_zero_rand_range() {
        thread_rng().gen_biguint_range(&BigUint::from(54u32), &BigUint::from(54u32));
    }

    #[test]
    #[should_panic]
    fn test_negative_rand_range() {
        let mut rng = thread_rng();
        let l = BigUint::from(2352u32);
        let u = BigUint::from(3513u32);
        // Switching u and l should fail:
        let _n: BigUint = rng.gen_biguint_range(&u, &l);
    }

    #[test]
    fn test_rand_uniform() {
        let mut rng = thread_rng();

        let tiny = Uniform::new(BigUint::from(236u32), BigUint::from(237u32));
        for _ in 0..10 {
            assert_eq!(rng.sample(&tiny), BigUint::from(236u32));
        }

        let l = BigUint::from(403469000u32 + 2352);
        let u = BigUint::from(403469000u32 + 3513);
        let below = Uniform::new(BigUint::zero(), u.clone());
        let range = Uniform::new(l.clone(), u.clone());
        for _ in 0..1000 {
            let n: BigUint = rng.sample(&below);
            assert!(n < u);

            let n: BigUint = rng.sample(&range);
            assert!(n >= l);
            assert!(n < u);
        }
    }
}

mod bigint {
    use num_bigint::{BigInt, RandBigInt, RandomBits};
    use num_traits::Zero;
    use rand::thread_rng;
    use rand::Rng;
    use rand::distributions::Uniform;

    #[test]
    fn test_rand() {
        let mut rng = thread_rng();
        let n: BigInt = rng.gen_bigint(137);
        assert!(n.bits() <= 137);
        assert!(rng.gen_bigint(0).is_zero());
    }

    #[test]
    fn test_rand_bits() {
        let mut rng = thread_rng();
        let n: BigInt = rng.sample(&RandomBits::new(137));
        assert!(n.bits() <= 137);
        let z: BigInt = rng.sample(&RandomBits::new(0));
        assert!(z.is_zero());
    }

    #[test]
    fn test_rand_range() {
        let mut rng = thread_rng();

        for _ in 0..10 {
            assert_eq!(
                rng.gen_bigint_range(&BigInt::from(236), &BigInt::from(237)),
                BigInt::from(236)
            );
        }

        fn check(l: BigInt, u: BigInt) {
            let mut rng = thread_rng();
            for _ in 0..1000 {
                let n: BigInt = rng.gen_bigint_range(&l, &u);
                assert!(n >= l);
                assert!(n < u);
            }
        }
        let l: BigInt = BigInt::from(403469000 + 2352);
        let u: BigInt = BigInt::from(403469000 + 3513);
        check(l.clone(), u.clone());
        check(-l.clone(), u.clone());
        check(-u.clone(), -l.clone());
    }

    #[test]
    #[should_panic]
    fn test_zero_rand_range() {
        thread_rng().gen_bigint_range(&BigInt::from(54), &BigInt::from(54));
    }

    #[test]
    #[should_panic]
    fn test_negative_rand_range() {
        let mut rng = thread_rng();
        let l = BigInt::from(2352);
        let u = BigInt::from(3513);
        // Switching u and l should fail:
        let _n: BigInt = rng.gen_bigint_range(&u, &l);
    }

    #[test]
    fn test_rand_uniform() {
        let mut rng = thread_rng();

        let tiny = Uniform::new(BigInt::from(236u32), BigInt::from(237u32));
        for _ in 0..10 {
            assert_eq!(rng.sample(&tiny), BigInt::from(236u32));
        }

        fn check(l: BigInt, u: BigInt) {
            let mut rng = thread_rng();
            let range = Uniform::new(l.clone(), u.clone());
            for _ in 0..1000 {
                let n: BigInt = rng.sample(&range);
                assert!(n >= l);
                assert!(n < u);
            }
        }
        let l: BigInt = BigInt::from(403469000 + 2352);
        let u: BigInt = BigInt::from(403469000 + 3513);
        check(l.clone(), u.clone());
        check(-l.clone(), u.clone());
        check(-u.clone(), -l.clone());
    }
}
