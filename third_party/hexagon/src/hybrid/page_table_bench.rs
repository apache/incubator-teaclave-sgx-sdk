use test::Bencher;
use super::page_table::PageTable;

#[bench]
fn bench_paged_mem_read(b: &mut Bencher) {
    let mut pt = PageTable::new();
    assert_eq!(pt.virtual_alloc(0x00100000), true);
    assert_eq!(pt.virtual_alloc(0x00200000), true);
    b.iter(|| {
        assert_eq!(pt.read_u64(0x00100000), Some(0));
        assert_eq!(pt.read_u64(0x00200000), Some(0));
        assert_eq!(pt.read_u64(0x00300000), None);
    });
}

#[bench]
fn bench_paged_mem_write(b: &mut Bencher) {
    let mut pt = PageTable::new();
    assert_eq!(pt.virtual_alloc(0x00100000), true);
    assert_eq!(pt.virtual_alloc(0x00200000), true);
    b.iter(|| {
        assert_eq!(pt.write_u64(0x00100000, 1), true);
        assert_eq!(pt.write_u64(0x00200000, 1), true);
        assert_eq!(pt.write_u64(0x00300000, 1), false);
    });
}
