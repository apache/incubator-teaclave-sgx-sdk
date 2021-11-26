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

use sgx_rand::*;

// pub use os::SgxRng
pub fn test_rand_os_sgxrng() {
    let mut os_rng = os::SgxRng::new().unwrap();
    let mut checksum_u32: u32 = 0;
    let mut checksum_u64: u64 = 0;
    let testcount = 1000;
    for _ in 0..testcount {
        checksum_u32 |= os_rng.next_u32();
        checksum_u64 |= os_rng.next_u64();
    }
    assert_ne!(checksum_u32, 0);
    assert_ne!(checksum_u64, 0);

    let mut rand_arr = [0; 1000];
    os_rng.fill_bytes(&mut rand_arr[..]);
    let cmp = [0; 1000].iter().zip(rand_arr.iter()).all(|(x, y)| x == y);
    assert_ne!(cmp, true);
}

// pub mod distribution
// Too hard to test
pub fn test_rand_distributions() {
    use sgx_rand::distributions::Range;
    use sgx_rand::distributions::*;
    use sgx_rand::*;

    // From rust-src/librand/distributions/rand.rs
    should_panic!(Range::new(10, 10));
    should_panic!(Range::new(10, 5));
    let mut rng = thread_rng();
    macro_rules! t {
        ($($ty:ident),*) => {{
            $(
               let v: &[($ty, $ty)] = &[(0, 10),
                                        (10, 127),
                                        ($ty::min_value(), $ty::MAX)];
               for &(low, high) in v {
                    let mut sampler: Range<$ty> = Range::new(low, high);
                    for _ in 0..1000 {
                        let v = sampler.sample(&mut rng);
                        assert!(low <= v && v < high);
                        let v = sampler.ind_sample(&mut rng);
                        assert!(low <= v && v < high);
                    }
                }
             )*
        }}
    }
    t!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

    macro_rules! t {
        ($($ty:ty),*) => {{
            $(
               let v: &[($ty, $ty)] = &[(0.0, 100.0),
                                        (-1e35, -1e25),
                                        (1e-35, 1e-25),
                                        (-1e35, 1e35)];
               for &(low, high) in v {
                    let mut sampler: Range<$ty> = Range::new(low, high);
                    for _ in 0..1000 {
                        let v = sampler.sample(&mut rng);
                        assert!(low <= v && v < high);
                        let v = sampler.ind_sample(&mut rng);
                        assert!(low <= v && v < high);
                    }
                }
             )*
        }}
    }

    t!(f32, f64);

    let mut exp = Exp::new(10.0);
    for _ in 0..1000 {
        assert!(exp.sample(&mut rng) >= 0.0);
        assert!(exp.ind_sample(&mut rng) >= 0.0);
    }

    should_panic!(Exp::new(0.0));
    should_panic!(Exp::new(-10.0));

    // gamma.rs
    use sgx_rand::distributions::{ChiSquared, FisherF, StudentT};
    let mut rng = thread_rng();
    let mut chi = ChiSquared::new(1.0);
    for _ in 0..1000 {
        chi.sample(&mut rng);
        chi.ind_sample(&mut rng);
    }

    let mut chi = ChiSquared::new(0.5);
    for _ in 0..1000 {
        chi.sample(&mut rng);
        chi.ind_sample(&mut rng);
    }

    let mut chi = ChiSquared::new(30.0);
    for _ in 0..1000 {
        chi.sample(&mut rng);
        chi.ind_sample(&mut rng);
    }

    should_panic!(ChiSquared::new(-1.0));
    let mut rng = StdRng::new().unwrap();
    let mut f = FisherF::new(2.0, 32.0);
    for _ in 0..1000 {
        f.sample(&mut rng);
        f.ind_sample(&mut rng);
    }

    let mut t = StudentT::new(11.0);
    for _ in 0..1000 {
        t.sample(&mut rng);
        t.ind_sample(&mut rng);
    }

    // normal.rs
    use sgx_rand::distributions::{LogNormal, Normal};

    let mut norm = Normal::new(10.0, 10.0);
    for _ in 0..1000 {
        norm.sample(&mut rng);
        norm.ind_sample(&mut rng);
    }

    should_panic!(Normal::new(10.0, -1.0));

    let mut lnorm = LogNormal::new(10.0, 10.0);
    for _ in 0..1000 {
        lnorm.sample(&mut rng);
        lnorm.ind_sample(&mut rng);
    }

    should_panic!(LogNormal::new(10.0, -1.0));

    use sgx_rand::distributions::{
        IndependentSample, RandSample, Sample, Weighted, WeightedChoice,
    };

    #[derive(PartialEq, Debug)]
    struct ConstRand(usize);
    impl Rand for ConstRand {
        fn rand<R: Rng>(_: &mut R) -> ConstRand {
            ConstRand(0)
        }
    }

    // 0, 1, 2, 3, ...
    struct CountingRng {
        i: u32,
    }
    impl Rng for CountingRng {
        fn next_u32(&mut self) -> u32 {
            self.i += 1;
            self.i - 1
        }
        fn next_u64(&mut self) -> u64 {
            self.next_u32() as u64
        }
    }

    let mut rand_sample = RandSample::<ConstRand>::new();

    assert_eq!(rand_sample.sample(&mut rng), ConstRand(0));
    assert_eq!(rand_sample.ind_sample(&mut rng), ConstRand(0));

    macro_rules! t {
        ($items:expr, $expected:expr) => {{
            let mut items = $items;
            let wc = WeightedChoice::new(&mut items);
            let expected = $expected;

            let mut rng = CountingRng { i: 0 };

            for &val in &expected {
                assert_eq!(wc.ind_sample(&mut rng), val)
            }
        }};
    }

    t!(
        vec![Weighted {
            weight: 1,
            item: 10
        }],
        [10]
    );

    // skip some
    t!(
        vec![
            Weighted {
                weight: 0,
                item: 20
            },
            Weighted {
                weight: 2,
                item: 21
            },
            Weighted {
                weight: 0,
                item: 22
            },
            Weighted {
                weight: 1,
                item: 23
            }
        ],
        [21, 21, 23]
    );

    // different weights
    t!(
        vec![
            Weighted {
                weight: 4,
                item: 30
            },
            Weighted {
                weight: 3,
                item: 31
            }
        ],
        [30, 30, 30, 30, 31, 31, 31]
    );

    // check that we're binary searching
    // correctly with some vectors of odd
    // length.
    t!(
        vec![
            Weighted {
                weight: 1,
                item: 40
            },
            Weighted {
                weight: 1,
                item: 41
            },
            Weighted {
                weight: 1,
                item: 42
            },
            Weighted {
                weight: 1,
                item: 43
            },
            Weighted {
                weight: 1,
                item: 44
            }
        ],
        [40, 41, 42, 43, 44]
    );
    t!(
        vec![
            Weighted {
                weight: 1,
                item: 50
            },
            Weighted {
                weight: 1,
                item: 51
            },
            Weighted {
                weight: 1,
                item: 52
            },
            Weighted {
                weight: 1,
                item: 53
            },
            Weighted {
                weight: 1,
                item: 54
            },
            Weighted {
                weight: 1,
                item: 55
            },
            Weighted {
                weight: 1,
                item: 56
            }
        ],
        [50, 51, 52, 53, 54, 55, 56]
    );

    should_panic!(WeightedChoice::<isize>::new(&mut []));

    should_panic!({
        let mut t = [
            Weighted { weight: 0, item: 0 },
            Weighted { weight: 0, item: 1 },
        ];
        WeightedChoice::new(&mut t);
    });

    let x = (!0) as u32 / 2; // x + x + 2 is the overflow
    should_panic!({
        WeightedChoice::new(&mut [
            Weighted { weight: x, item: 0 },
            Weighted { weight: 1, item: 1 },
            Weighted { weight: x, item: 2 },
            Weighted { weight: 1, item: 3 },
        ]);
    });
}

// pub use isaac::{IsaacRng, Isaac64Rng};
// We cannot test because methods are private
pub fn test_rand_isaac_isaacrng() {
    let mut unseeded_32 = IsaacRng::new_unseeded();
    let mut u32_arr = [0; 32];
    for i in 0..32 {
        u32_arr[i] = unseeded_32.next_u32();
    }
    assert_eq!(
        u32_arr,
        [
            1909923794, 3041581799, 3564668249, 3277924858, 568073897, 1018722762, 3627754367,
            4198294973, 2365883657, 3626931337, 3678742600, 3391154246, 1343199022, 250607936,
            14072211, 3155345791, 3971560663, 4015545006, 1861263481, 249057481, 1397017464,
            4117866301, 1748577758, 2418980344, 2932544972, 3384981086, 3677243665, 3631899496,
            1984325203, 1082540525, 1970086400, 1634359601
        ]
    );

    let mut unseeded_64 = Isaac64Rng::new_unseeded();
    let mut u64_arr = [0; 32];
    for i in 0..32 {
        u64_arr[i] = unseeded_64.next_u64();
    }
    assert_eq!(
        u64_arr,
        [
            17761629189777429372,
            9558052838949843840,
            18366607087662878139,
            5554737638101975254,
            9657764456686032042,
            1042790411024899657,
            18326046843346171767,
            6155338025429880858,
            604674934004978847,
            15021509919615502100,
            3524189203627442317,
            3491674822202243807,
            2812011890713530556,
            6197411209201204330,
            3423587972865533405,
            13333204927595598570,
            8278113110980066893,
            16728828598150967100,
            4739548913818718572,
            12247507049266305179,
            13400017821939308292,
            14607632686415099658,
            9600795998696455796,
            5780363980783644600,
            4671476687197924946,
            1273203271378292130,
            11668772324252115302,
            5778980740914384907,
            3192360781399669869,
            8925632526356214095,
            13227689911971237637,
            16127447699671226298
        ]
    );

    unseeded_32.reseed(&[1, 2, 3, 4]);
    for i in 0..32 {
        u32_arr[i] = unseeded_32.next_u32();
    }
    assert_eq!(
        u32_arr,
        [
            3673720382, 1957022519, 2949967219, 2273082436, 2412264859, 1616913581, 4286187434,
            1337573575, 3564981768, 3931377724, 180676801, 4234301014, 3123903540, 804531392,
            1687941800, 3180870208, 3007460037, 1643580469, 1601966685, 385631690, 3412228283,
            2633128738, 2786818767, 1385157572, 3925804522, 4251266711, 4219473709, 3054074232,
            4185095632, 2891236088, 1792921516, 1421268925
        ]
    );

    unseeded_64.reseed(&[5, 6, 7, 8]);
    for i in 0..32 {
        u64_arr[i] = unseeded_64.next_u64();
    }
    assert_eq!(
        u64_arr,
        [
            1519553293257924390,
            8496251504967724708,
            4814852879883820893,
            701662672566014897,
            4253212733272379994,
            14710390856535061408,
            13525913188857210438,
            4176812646507392158,
            5274347845703478634,
            7494101755291139543,
            12433257569627749800,
            13555030297923086622,
            12912035290107031759,
            10277213319518338357,
            13212221592643678736,
            14214333261821823450,
            13451082182892489478,
            17471061581144953786,
            663937856451252907,
            7794799705525010541,
            8878465633657608553,
            11684451817656999723,
            3085412699425654134,
            7996244368322668557,
            9264264892592899720,
            18202308129527220178,
            14762563311397316905,
            7967643698755143779,
            13033364966409644229,
            2619244012980533217,
            2251084057656619285,
            16661636750341142669
        ]
    );

    let mut from_seed_obj = IsaacRng::from_seed(&[1, 2, 3, 4]);
    for i in 0..32 {
        u32_arr[i] = from_seed_obj.next_u32();
    }
    assert_eq!(
        u32_arr,
        [
            3673720382, 1957022519, 2949967219, 2273082436, 2412264859, 1616913581, 4286187434,
            1337573575, 3564981768, 3931377724, 180676801, 4234301014, 3123903540, 804531392,
            1687941800, 3180870208, 3007460037, 1643580469, 1601966685, 385631690, 3412228283,
            2633128738, 2786818767, 1385157572, 3925804522, 4251266711, 4219473709, 3054074232,
            4185095632, 2891236088, 1792921516, 1421268925
        ]
    );

    let mut from_seed_obj_64 = Isaac64Rng::from_seed(&[5, 6, 7, 8]);
    for i in 0..32 {
        u64_arr[i] = from_seed_obj_64.next_u64();
    }
    assert_eq!(
        u64_arr,
        [
            1519553293257924390,
            8496251504967724708,
            4814852879883820893,
            701662672566014897,
            4253212733272379994,
            14710390856535061408,
            13525913188857210438,
            4176812646507392158,
            5274347845703478634,
            7494101755291139543,
            12433257569627749800,
            13555030297923086622,
            12912035290107031759,
            10277213319518338357,
            13212221592643678736,
            14214333261821823450,
            13451082182892489478,
            17471061581144953786,
            663937856451252907,
            7794799705525010541,
            8878465633657608553,
            11684451817656999723,
            3085412699425654134,
            7996244368322668557,
            9264264892592899720,
            18202308129527220178,
            14762563311397316905,
            7967643698755143779,
            13033364966409644229,
            2619244012980533217,
            2251084057656619285,
            16661636750341142669
        ]
    );

    let mut isaac_rand = IsaacRng::rand(&mut unseeded_32);
    for i in 0..32 {
        u32_arr[i] = isaac_rand.next_u32();
    }
    assert_eq!(
        u32_arr,
        [
            3731735955, 1060971802, 2060608277, 2017700212, 2906693947, 983683489, 2442131015,
            1804959108, 133350985, 1854001588, 1536802647, 1681331746, 3926133099, 2418741330,
            2933560360, 3445159402, 372862460, 584055518, 3889338902, 2010948997, 156911533,
            1516660418, 2985909906, 2363648761, 3441715164, 1549975272, 1594543237, 4051120660,
            4197709030, 3816779293, 3783690383, 2208018815
        ]
    );

    let mut isaac_rand64 = Isaac64Rng::rand(&mut unseeded_64);
    for i in 0..32 {
        u64_arr[i] = isaac_rand64.next_u64();
    }
    assert_eq!(
        u64_arr,
        [
            9641310933002597978,
            1377462969700517843,
            505161414013526504,
            15945732720326764686,
            15418802195980990528,
            16470443181580299993,
            2257820220688421011,
            11605056212052353438,
            16446934715429265221,
            2703881146695451334,
            13779048927678904385,
            3096257807787648519,
            8078370161234145075,
            11428361090362647425,
            11163353455881119221,
            2479568798082945555,
            12997537085833811642,
            15719812629115354877,
            11384722140468426949,
            1034436110230828879,
            13933827272989341125,
            16841927942905074390,
            18283030736011842727,
            18161628412401034420,
            5992070539860306822,
            14657980260306047343,
            4240530741486241664,
            16128750684074755458,
            13382816331913570910,
            1450784801074240917,
            12172859719381448420,
            11561024719408488519
        ]
    );
}

// pub use chacha::ChaChaRng;
pub fn test_rand_chacharng() {
    let mut chacha_obj = ChaChaRng::new_unseeded();
    let mut arr_32 = [0; 32];
    let mut arr_64 = [0; 32];
    for i in 0..32 {
        arr_32[i] = chacha_obj.next_u32();
        arr_64[i] = chacha_obj.next_u64();
    }
    assert_eq!(
        arr_32,
        [
            2917185654, 683509331, 3438229160, 2370328401, 4105716586, 2254827186, 2090318488,
            1768285000, 1981921065, 3577337972, 520806828, 3781961315, 2576249230, 1572053161,
            4248444456, 3230977993, 1486888979, 180343358, 882336707, 772637944, 4294454303,
            2115397425, 3755235835, 720939292, 261668430, 2229903491, 3478228973, 1178947740,
            3612365086, 3675722242, 2702218887, 562719562
        ]
    );
    assert_eq!(
        arr_64,
        [
            10393729188386987328,
            13265865887038934432,
            14343251829215281626,
            4602718913620007096,
            2062956584559347651,
            13755971968358175061,
            939050496938282955,
            4491151038014484018,
            4850067409370051029,
            2891290281678408273,
            8020199878315477293,
            635428310198806553,
            14295220693338866608,
            7509513789169826591,
            2214742853367878259,
            7093718481497530944,
            7734567148921168085,
            2355867238271518203,
            11858361224218268717,
            1394059717204795920,
            6203934356551920360,
            8397145010625398571,
            12762635603290365669,
            15798090700413956747,
            14488165283315532822,
            14534818617999799127,
            5587461883115671776,
            438217751243844519,
            12085977715001316377,
            14462731156256673612,
            5957300949303581564,
            9621633472041422470
        ]
    );

    let mut chacha_obj = ChaChaRng::new_unseeded();
    chacha_obj.set_counter(0, 1234567890u64);
    for i in 0..32 {
        arr_32[i] = chacha_obj.next_u32();
        arr_64[i] = chacha_obj.next_u64();
    }
    assert_eq!(
        arr_32,
        [
            3689073239, 2242961698, 417536494, 1503493075, 2585895478, 1865454281, 2938464816,
            3998462791, 1427673087, 2023308892, 3934458784, 2154405649, 4261926467, 944974180,
            1017947150, 1858486886, 2081637745, 6698551, 2593531725, 989113434, 952425310,
            2821646029, 2012236296, 2466857171, 1705021287, 2712374590, 2467996873, 2529275143,
            3691921567, 4164253542, 88339832, 3139246687
        ]
    );
    assert_eq!(
        arr_64,
        [
            5208214742801571470,
            12560379081283373091,
            16366572874613089836,
            5810973028396244379,
            5062530146601697561,
            217785109116347749,
            10945155018052356834,
            3830068552016855006,
            8631305173908678621,
            2826768761480962968,
            10354302052483333752,
            14791155778847304997,
            9236871951464247547,
            10565693803420516117,
            2303174007018927082,
            11829861525625440339,
            17157070760956407958,
            18121029959197227032,
            15895614391418239344,
            16692872359552745709,
            364895533855401959,
            15289680275263377042,
            14444259489219757497,
            6497689978530420464,
            7473989280813969277,
            3566235157971589227,
            5502428546971923888,
            15258817180873710847,
            1169964352668698116,
            5863926115043645599,
            15610749783507155661,
            5502704700771473503
        ]
    );

    chacha_obj.reseed(&[1, 2, 3, 4]);
    for i in 0..32 {
        arr_32[i] = chacha_obj.next_u32();
    }
    assert_eq!(
        arr_32,
        [
            3931434082, 34647919, 2091071039, 2892552082, 3564830854, 314912736, 3076587290,
            3191577189, 2464328121, 2779997206, 4263592728, 2125004658, 1054360779, 3051661270,
            3060579487, 3656634963, 1388834776, 581640397, 1637469323, 1640397641, 2995955648,
            4048658223, 1600465837, 1968152875, 1014202063, 3686500314, 3021394822, 2541451384,
            2305439270, 1469542314, 3487304956, 2286598508
        ]
    );

    let mut chacha_obj = ChaChaRng::from_seed(&[5, 6, 7, 8]);
    for i in 0..32 {
        arr_32[i] = chacha_obj.next_u32();
    }
    assert_eq!(
        arr_32,
        [
            337897705, 1044901876, 2470587838, 3671677287, 1941090964, 2010394378, 3290836538,
            1347698920, 2248500576, 2681774419, 1002813011, 65452860, 1850555370, 3036027713,
            2306938851, 3450488430, 2195520755, 38533316, 692482625, 3410200303, 1052930979,
            4087429478, 2501438173, 2495319620, 277097193, 1034107694, 2396644634, 3334027496,
            3808708279, 3705622889, 874306690, 1237186790
        ]
    );

    let mut chacha_obj = ChaChaRng::rand(&mut chacha_obj);
    for i in 0..32 {
        arr_32[i] = chacha_obj.next_u32();
    }
    assert_eq!(
        arr_32,
        [
            1225539984, 2122523891, 4046438709, 2772996924, 2441149875, 4253841608, 2837944766,
            1256279550, 134014938, 192621923, 4187133245, 45039689, 3455715024, 3744176230,
            3194372814, 3469543760, 1107844861, 3990938641, 4095630261, 3461287666, 321560749,
            934078781, 2671586377, 1048877492, 1523056052, 3373403085, 913407943, 2835509350,
            3232974874, 3910156243, 3652032104, 442368320
        ]
    );
}

// reseeding.rs
pub fn test_rand_reseeding() {
    use std::prelude::v1::*;

    use sgx_rand::reseeding::{ReseedWithDefault, ReseedingRng};

    struct Counter {
        i: u32,
    }

    impl Rng for Counter {
        fn next_u32(&mut self) -> u32 {
            self.i += 1;
            // very random
            self.i - 1
        }
    }
    impl Default for Counter {
        /// Constructs a `Counter` with initial value zero.
        fn default() -> Counter {
            Counter { i: 0 }
        }
    }
    impl SeedableRng<u32> for Counter {
        fn reseed(&mut self, seed: u32) {
            self.i = seed;
        }
        fn from_seed(seed: u32) -> Counter {
            Counter { i: seed }
        }
    }
    type MyRng = ReseedingRng<Counter, ReseedWithDefault>;

    let mut rs = ReseedingRng::new(Counter { i: 0 }, 400, ReseedWithDefault);

    let mut i = 0;
    for _ in 0..1000 {
        assert_eq!(rs.next_u32(), i % 100);
        i += 1;
    }

    let mut ra: MyRng = SeedableRng::from_seed((ReseedWithDefault, 2));
    let mut rb: MyRng = SeedableRng::from_seed((ReseedWithDefault, 2));
    assert!(ra
        .gen_ascii_chars()
        .take(100)
        .eq(rb.gen_ascii_chars().take(100)));

    let mut r: MyRng = SeedableRng::from_seed((ReseedWithDefault, 3));
    let string1: String = r.gen_ascii_chars().take(100).collect();

    r.reseed((ReseedWithDefault, 3));

    let string2: String = r.gen_ascii_chars().take(100).collect();
    assert_eq!(string1, string2);

    const FILL_BYTES_V_LEN: usize = 13579;

    let mut v = vec![0; FILL_BYTES_V_LEN];
    let mut rng = StdRng::new().unwrap();
    rng.fill_bytes(&mut v);

    // Sanity test: if we've gotten here, `fill_bytes` has not infinitely
    // recursed.
    assert_eq!(v.len(), FILL_BYTES_V_LEN);

    // To test that `fill_bytes` actually did something, check that the
    // average of `v` is not 0.
    let mut sum = 0.0;
    for &x in &v {
        sum += x as f64;
    }
    assert!(sum / v.len() as f64 != 0.0);
}

// No need for testing others
// Already included in the above tests
