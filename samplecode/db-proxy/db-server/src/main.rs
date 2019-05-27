extern crate libc;
extern crate rocksdb;

mod util;

use rocksdb::DB;
use util::DBPath;

fn main() {
    let path = DBPath::new("_rust_rocksdb_snapshottest");
    println!("DB init");
    {
        let db = DB::open_default(&path).unwrap();

        assert!(db.put(b"k1", b"v1111").is_ok());

        let snap = db.snapshot();
        assert!(snap.get(b"k1").unwrap().unwrap().to_utf8().unwrap() == "v1111");

        assert!(db.put(b"k2", b"v2222").is_ok());

        assert!(db.get(b"k2").unwrap().is_some());
        assert!(snap.get(b"k2").unwrap().is_none());
    }
}
