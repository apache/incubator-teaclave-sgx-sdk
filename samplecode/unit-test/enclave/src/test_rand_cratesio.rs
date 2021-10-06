use rand::{ChaChaRng, RngCore, SeedableRng};

pub fn test_rand_cratesio() {
    let seed = [0u8; 32];
    let mut rng = ChaChaRng::from_seed(seed);
    let mut results = [0u8; 32];
    rng.fill_bytes(&mut results);
    let expected = [
        118, 184, 224, 173, 160, 241, 61, 144, 64, 93, 106, 229, 83, 134, 189, 40, 189, 210, 25,
        184, 160, 141, 237, 26, 168, 54, 239, 204, 139, 119, 13, 199,
    ];
    assert_eq!(results, expected);
}
