use std::prelude::v1::*;
use std::ops::Deref;
use std::sync::{Arc, SgxMutex as Mutex};
use std::cell::UnsafeCell;
use byteorder::{ReadBytesExt, NativeEndian};

// N_PAGES * PAGE_SIZE = 2 ^ 32
const N_PAGES: usize = 1024;
const PAGE_SIZE: usize = 1048576;

struct UnsafeData {
    inner: UnsafeCell<Box<[u8]>>
}

impl Deref for UnsafeData {
    type Target = UnsafeCell<Box<[u8]>>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

unsafe impl Sync for UnsafeData {}

impl UnsafeData {
    fn new(inner: UnsafeCell<Box<[u8]>>) -> UnsafeData {
        UnsafeData {
            inner: inner
        }
    }
}

// Virtual pointers are 64-bit.
// The upper 32 bits are used to specify the 'address space'
// and the lower 32 bits are the actual virtual address that
// will be handled by the actual address resolver.
//
// When a PageTable is cloned, they must share the same
// underlying data.
//
// PageTable must be Send.
#[derive(Clone)]
pub struct PageTable {
    pt_impl: Arc<PageTableImpl>,
    page_cache: Vec<Option<Arc<UnsafeData>>>
}

unsafe impl Sync for PageTable {}

struct PageTableImpl {
    pages: Mutex<Vec<Option<Arc<UnsafeData>>>>
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct AddrInfo {
    pub address_space: u32,
    pub page_id: u32,
    pub index: u32
}

impl From<u64> for AddrInfo {
    fn from(other: u64) -> AddrInfo {
        let addr_space = (other >> 32) as u32;
        let vaddr = (other & 0xffffffff) as u32;

        let page_id = (vaddr >> 20) as u32;
        let index = (vaddr & 0xfffff) as u32; // [0..1048576)

        AddrInfo {
            address_space: addr_space,
            page_id: page_id,
            index: index
        }
    }
}

impl PageTableImpl {
    fn new() -> PageTableImpl {
        PageTableImpl {
            pages: Mutex::new(vec![None; N_PAGES])
        }
    }

    fn create_page(&self, id: usize) -> Option<Arc<UnsafeData>> {
        let mut pages = self.pages.lock().unwrap();
        if id >= pages.len() {
            return None;
        }

        pages[id] = Some(Arc::new(UnsafeData::new(
            UnsafeCell::new(vec![0; PAGE_SIZE].into_boxed_slice())
        )));
        pages[id].clone()
    }
}

impl PageTable {
    pub fn new() -> PageTable {
        PageTable {
            pt_impl: Arc::new(PageTableImpl::new()),
            page_cache: vec![None; N_PAGES]
        }
    }

    fn locate<'a>(&'a mut self, vaddr: AddrInfo, len: usize) -> Option<&'a mut [u8]> {
        let addr_space = vaddr.address_space;
        let page_id = vaddr.page_id as usize;
        let index = vaddr.index as usize;

        if addr_space != 0 {
            return None;
        }

        // [index..index + len)
        if page_id >= N_PAGES || index + len > PAGE_SIZE {
            return None;
        }

        if let Some(ref pc) = self.page_cache[page_id] {
            let inner = unsafe { &mut * pc.get() };
            return Some(&mut inner[index..index + len])
        }

        {
            let global_pages = self.pt_impl.pages.lock().unwrap();
            if let Some(ref pc) = global_pages[page_id] {
                self.page_cache[page_id] = Some(pc.clone());

                let pc = self.page_cache[page_id].as_ref().unwrap();
                let inner = unsafe { &mut * pc.get() };

                return Some(&mut inner[index..index + len])
            }
        }

        None
    }

    pub fn virtual_alloc<T: Into<AddrInfo>>(&mut self, vaddr: T) -> bool {
        let vaddr: AddrInfo = vaddr.into();

        if let Some(_) = self.pt_impl.create_page(vaddr.page_id as usize) {
            true
        } else {
            false
        }
    }

    pub fn get<'a, T: Into<AddrInfo>>(&'a mut self, vaddr: T, len: usize) -> Option<&'a [u8]> {
        self.locate(vaddr.into(), len).map(|blk| blk as &'a [u8])
    }

    pub fn set<T: Into<AddrInfo>>(&mut self, vaddr: T, data: &[u8]) -> bool {
        match self.locate(vaddr.into(), data.len()) {
            Some(target) => {
                target.copy_from_slice(data);
                true
            },
            None => false
        }
    }

    pub fn read_u8<T: Into<AddrInfo>>(&mut self, vaddr: T) -> Option<u8> {
        self.get(vaddr, 1).map(|view| view[0])
    }

    pub fn write_u8<T: Into<AddrInfo>>(&mut self, vaddr: T, value: u8) -> bool {
        self.set(vaddr, &[value])
    }

    pub fn read_i8<T: Into<AddrInfo>>(&mut self, vaddr: T) -> Option<i8> {
        self.get(vaddr, 1).map(|mut view| view.read_i8().unwrap())
    }

    pub fn write_i8<T: Into<AddrInfo>>(&mut self, vaddr: T, value: i8) -> bool {
        self.set(vaddr, &[unsafe {
            ::std::mem::transmute::<i8, u8>(value)
        }])
    }

    pub fn read_u16<T: Into<AddrInfo>>(&mut self, vaddr: T) -> Option<u16> {
        self.get(vaddr, 2).map(|mut view| view.read_u16::<NativeEndian>().unwrap())
    }

    pub fn write_u16<T: Into<AddrInfo>>(&mut self, vaddr: T, value: u16) -> bool {
        self.set(vaddr, &unsafe {
            ::std::mem::transmute::<u16, [u8; 2]>(value)
        })
    }

    pub fn read_i16<T: Into<AddrInfo>>(&mut self, vaddr: T) -> Option<i16> {
        self.get(vaddr, 2).map(|mut view| view.read_i16::<NativeEndian>().unwrap())
    }

    pub fn write_i16<T: Into<AddrInfo>>(&mut self, vaddr: T, value: i16) -> bool {
        self.set(vaddr, &unsafe {
            ::std::mem::transmute::<i16, [u8; 2]>(value)
        })
    }

    pub fn read_u32<T: Into<AddrInfo>>(&mut self, vaddr: T) -> Option<u32> {
        self.get(vaddr, 4).map(|mut view| view.read_u32::<NativeEndian>().unwrap())
    }

    pub fn write_u32<T: Into<AddrInfo>>(&mut self, vaddr: T, value: u32) -> bool {
        self.set(vaddr, &unsafe {
            ::std::mem::transmute::<u32, [u8; 4]>(value)
        })
    }

    pub fn read_i32<T: Into<AddrInfo>>(&mut self, vaddr: T) -> Option<i32> {
        self.get(vaddr, 4).map(|mut view| view.read_i32::<NativeEndian>().unwrap())
    }

    pub fn write_i32<T: Into<AddrInfo>>(&mut self, vaddr: T, value: i32) -> bool {
        self.set(vaddr, &unsafe {
            ::std::mem::transmute::<i32, [u8; 4]>(value)
        })
    }

    pub fn read_u64<T: Into<AddrInfo>>(&mut self, vaddr: T) -> Option<u64> {
        self.get(vaddr, 8).map(|mut view| view.read_u64::<NativeEndian>().unwrap())
    }

    pub fn write_u64<T: Into<AddrInfo>>(&mut self, vaddr: T, value: u64) -> bool {
        self.set(vaddr, &unsafe {
            ::std::mem::transmute::<u64, [u8; 8]>(value)
        })
    }

    pub fn read_i64<T: Into<AddrInfo>>(&mut self, vaddr: T) -> Option<i64> {
        self.get(vaddr, 8).map(|mut view| view.read_i64::<NativeEndian>().unwrap())
    }

    pub fn write_i64<T: Into<AddrInfo>>(&mut self, vaddr: T, value: i64) -> bool {
        self.set(vaddr, &unsafe {
            ::std::mem::transmute::<i64, [u8; 8]>(value)
        })
    }

    pub fn read_f64<T: Into<AddrInfo>>(&mut self, vaddr: T) -> Option<f64> {
        self.get(vaddr, 8).and_then(|mut view| {
            match view.read_f64::<NativeEndian>() {
                Ok(v) => Some(v),
                Err(_) => None
            }
        })
    }

    pub fn write_f64<T: Into<AddrInfo>>(&mut self, vaddr: T, value: f64) -> bool {
        self.set(vaddr, &unsafe {
            ::std::mem::transmute::<f64, [u8; 8]>(value)
        })
    }
}
