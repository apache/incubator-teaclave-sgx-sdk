use super::page_table;

#[test]
fn test_read_write() {
    let mut pt = page_table::PageTable::new();
    assert_eq!(pt.write_u64(0x00100000, 42), false);
    assert_eq!(pt.write_u64(0x00200000, 192), false);

    assert_eq!(pt.virtual_alloc(0x00100000), true);

    assert_eq!(pt.write_u64(0x00100000, 42), true);
    assert_eq!(pt.write_u64(0x00200000, 192), false);

    assert_eq!(pt.write_u64(0x00100008, 7889315787603), true);
    assert_eq!(pt.read_u64(0x00100000), Some(42));
    assert_eq!(pt.read_u64(0x00100008), Some(7889315787603));

    assert_eq!(pt.read_u64(0x001ffff8), Some(0));
    assert_eq!(pt.read_u64(0x001ffff9), None);

    assert_eq!(pt.virtual_alloc(0x00200000), true);

    assert_eq!(pt.write_u64(0x00200000, 192), true);
    assert_eq!(pt.read_u64(0x00100000), Some(42));
    assert_eq!(pt.read_u64(0x00200000), Some(192));
}

#[test]
fn test_types() {
    let mut pt = page_table::PageTable::new();

    assert_eq!(pt.virtual_alloc(0x0), true);

    assert_eq!(pt.write_u8(0x0, 42), true);
    assert_eq!(pt.read_u8(0x0), Some(42));

    assert_eq!(pt.write_i8(0x0, -1), true);
    assert_eq!(pt.read_i8(0x0), Some(-1));

    assert_eq!(pt.write_u16(0x0, 12345), true);
    assert_eq!(pt.read_u16(0x0), Some(12345));

    assert_eq!(pt.write_i16(0x0, -12345), true);
    assert_eq!(pt.read_i16(0x0), Some(-12345));

    assert_eq!(pt.write_u32(0x0, 99200), true);
    assert_eq!(pt.read_u32(0x0), Some(99200));

    assert_eq!(pt.write_i32(0x0, -99200), true);
    assert_eq!(pt.read_i32(0x0), Some(-99200));

    assert_eq!(pt.write_u64(0x0, 7889315787603), true);
    assert_eq!(pt.read_u64(0x0), Some(7889315787603));

    assert_eq!(pt.write_i64(0x0, -7889315787603), true);
    assert_eq!(pt.read_i64(0x0), Some(-7889315787603));

    assert_eq!(pt.write_f64(0x0, 0.267207909), true);
    assert!((pt.read_f64(0x0).unwrap() - 0.267207909).abs() < 1e-12);
}

#[test]
fn test_concurrent() {
    use std::thread;
    let pt = page_table::PageTable::new();

    let handles = (0..128).map(|id| {
        let mut pt = pt.clone();
        thread::spawn(move || {
            let base: u64 = 0x00100000 * (id as u64);
            assert_eq!(pt.virtual_alloc(base), true);
            for i in 0..1000 {
                assert_eq!(pt.write_u64(base + (i as u64) * 8, i), true);
            }
            for i in 0..1000 {
                assert_eq!(pt.read_u64(base + (i as u64) * 8), Some(i));
            }
        })
    });

    for handle in handles {
        handle.join().unwrap();
    }
}
