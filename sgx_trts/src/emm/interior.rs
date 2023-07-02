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

use buddy_system_allocator::LockedHeap;
use intrusive_collections::intrusive_adapter;
use intrusive_collections::linked_list::CursorMut;
use intrusive_collections::UnsafeRef;
use intrusive_collections::{LinkedList, LinkedListLink};

use crate::edmm::{PageInfo, PageType, ProtFlags};
use crate::emm::alloc::StaticAlloc;
use crate::veh::{ExceptionHandler, ExceptionInfo};
use alloc::alloc::Global;
use alloc::boxed::Box;
use core::alloc::Layout;
use core::ffi::c_void;
use core::mem::transmute;
use core::mem::MaybeUninit;
use core::ptr::NonNull;
use spin::{Mutex, Once};

use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::types::ProtectPerm;

use crate::emm::alloc::ResAlloc;
use crate::emm::ema::EMA;
use crate::emm::user::{self, is_within_rts_range, is_within_user_range, USER_RANGE};
use crate::enclave::is_within_enclave;

use super::ema::ResEmaAda;
use super::flags::AllocFlags;

const STATIC_MEM_SIZE: usize = 65536;

/// first level: static memory
static STATIC: LockedHeap<32> = LockedHeap::empty();

static mut STATIC_MEM: [u8; STATIC_MEM_SIZE] = [0; STATIC_MEM_SIZE];

pub fn init() {
    unsafe {
        STATIC
            .lock()
            .init(STATIC_MEM.as_ptr() as usize, STATIC_MEM_SIZE);
    }
}

/// second level: reserve memory
///
static RES_ALLOCATOR: Once = Once::new();

pub fn init_res() {
    // res_allocator需要在meta_allocator之后初始化
    RES_ALLOCATOR.call_once(|| {
        Mutex::new(Reserve::new(1024));
    });
}

// mm_reserve
struct Chunk {
    base: usize,
    size: usize,
    used: usize,
    link: LinkedListLink, // intrusive linkedlist
}

intrusive_adapter!(ChunkAda = Global, UnsafeRef<Chunk>: Chunk { link: LinkedListLink });
// let linkedlist = LinkedList::new(ResChunk_Adapter::new());

// mm_reserve
struct Block {
    size: usize,
    link: LinkedListLink, // intrusive linkedlist
}

intrusive_adapter!(BlockAda = Global, UnsafeRef<Block>: Block { link: LinkedListLink });

pub struct Reserve {
    // 这些list是block list，每个block用于存放如 ema meta / bitmap meta / bitmap data
    exact_blocks: [LinkedList<BlockAda>; 256],
    large_blocks: LinkedList<BlockAda>,

    // chunks 这个结构体是存放于reserve EMA分配的reserve内存
    chunks: LinkedList<ChunkAda>,
    // compared to intel emm using user range to store ema meta,
    // we use rts range to store ema node
    emas: LinkedList<ResEmaAda>,

    // statistics
    allocated: usize,
    total: usize,
}

impl Reserve {
    /// Create an empty heap
    pub fn new(size: usize) -> Self {
        // unsafe {
        //     self.add_reserve(size);
        // }
        let exact_blocks: [LinkedList<BlockAda>; 256] = {
            let mut exact_blocks: [MaybeUninit<LinkedList<BlockAda>>; 256] =
                MaybeUninit::uninit_array();
            for block in &mut exact_blocks {
                block.write(LinkedList::new(BlockAda::new()));
            }
            unsafe { transmute(exact_blocks) }
        };

        Self {
            exact_blocks,
            large_blocks: LinkedList::new(BlockAda::new()),
            chunks: LinkedList::new(ChunkAda::new()),
            emas: LinkedList::new(ResEmaAda::new()),
            allocated: 0,
            total: 0,
        }
    }
    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        // // 先check是否内存是否不够了，如果不够了就掉用add_reserve
        // // 从空闲区域分配一块内存，这块内存头部有个block header，记录使用的bytes
        // // 随后，把这块block链入对应链表
        // static threshold = 512*1024; // 0.5MB
        // if self.allocated + layout.size() + threshold > self.total {
        //     self.add_reserve(2*threshold);
        // }

        // // search available region
        // if layout.size() < 256 {
        //     let exact_block_list = exact_blocks[layout.size()-1];
        //     if !exact_block_list.is_empty() {
        //         let block: Box<ResBlock> = exact_block_list.pop_front().unwrap();
        //         let ptr = unsafe {
        //             block.as_mut_ptr() - mem::size_of::<ResBlock>();
        //         }
        //         let addr = std::ptr::NonNull::<u8>::new(ptr as *mut u8).unwrap();
        //         return addr;
        //     }
        // } else {
        //     // similar operation in large blocks
        // }

        // // no available region in free blocks
        // let chunk = self.chunks.iter().find(
        //     |&chunk| (chunk.size - chunk.used) > layout.size()
        // );
        // if let chunk = Some(chunk) {
        //     let ptr = chunk.base + chunk.used;
        //     chunk.used += layout.size();
        //     let addr = std::ptr::NonNull::<u8>::new(ptr as *mut u8).unwrap();
        //     return addr;
        // } else {
        //     // self.add_reserve
        //     // self.alloc()
        // }
        todo!()
    }
    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        // // 先通过ptr前面的block知道这个ptr的长度是多少
        // if size < 256 {
        //     // 将当前的ptr塞回队列
        // } else {
        //     // similar operation in large blocks
        // }
        todo!()
    }
    pub unsafe fn add_reserve(&mut self, size: usize) {
        // // 分配一个EMA
        // let reserve_ema: EMA = EMA::new(size);
        // reserve_ema.alloc(size);
        // self.emas.push(reserve_ema);
        // // 将mm_res写入reserve_ema分配的EMA的首部
        // let chunk: ResChunk = ResChunk::new();
        // unsafe {
        //     let res_mem_ptr = reserve_ema.alloc().unwrap().as_mut_ptr();
        // 	std::ptr::write(res_mem_ptr as *mut ResChunk, chunk);
        //     let res_node = Box::from_raw(res_mem_ptr as *mut MM_Res );
        //     // let new_mm_res = std::ptr::read(metadata_ptr as *const ResChunk);
        //     self.mm_reserve_list.push(res_node);
        // }
        todo!()
    }

    // Not considering concurrency and lock
    // TODO: carefull examination !!
    fn alloc_inner(
        &mut self,
        addr: Option<usize>,
        size: usize,
        flags: AllocFlags,
    ) -> SgxResult<usize> {
        let info = if flags.contains(AllocFlags::RESERVED) {
            PageInfo {
                prot: ProtFlags::NONE,
                typ: PageType::None,
            }
        } else {
            PageInfo {
                prot: ProtFlags::R | ProtFlags::W,
                typ: PageType::Reg,
            }
        };

        let align_flag = 12;
        let align_mask: usize = (1 << align_flag) - 1;
        if (addr.unwrap_or(0) & align_mask) != 0 {
            return Err(SgxStatus::InvalidParameter);
        }

        if addr.is_some() {
            let addr = addr.unwrap();
            let is_fixed_alloc = flags.contains(AllocFlags::FIXED);
            let range = self.search_ema_range(addr, addr + size, false);

            match range {
                // exist in emas list
                Ok(_) => {
                    // TODO: ema realloc from reserve
                    if is_fixed_alloc {
                        // FIXME: return EEXIST
                        return Err(SgxStatus::InvalidParameter);
                    }
                }
                // not exist in emas list
                Err(_) => {
                    let next_ema = self.find_free_region_at(addr, size);
                    if next_ema.is_ok() && is_fixed_alloc {
                        return Err(SgxStatus::InvalidParameter);
                    }
                }
            };
        };

        let (free_addr, mut next_ema) = self.find_free_region(size, 1 << align_flag)?;

        let mut new_ema = Box::<EMA<StaticAlloc>, StaticAlloc>::new_in(
            EMA::<StaticAlloc>::new(free_addr, size, flags, info, None, None, StaticAlloc)?,
            StaticAlloc,
        );
        new_ema.alloc()?;
        next_ema.insert_before(new_ema);
        return Ok(free_addr);
    }

    // Not considering concurrency and lock
    // TODO: carefull examination !!
    fn commit_inner(&mut self, addr: usize, size: usize) -> SgxResult {
        let (mut cursor, ema_num) = self.search_ema_range(addr, addr + size, true)?;
        let start_ema_ptr = cursor.get().unwrap() as *const EMA<StaticAlloc>;

        // check ema can commit
        let mut count = ema_num;
        while count != 0 {
            cursor.get().unwrap().commit_check()?;
            cursor.move_next();
            count -= 1;
        }

        let mut cursor = unsafe { self.emas.cursor_mut_from_ptr(start_ema_ptr) };

        count = ema_num;
        while count != 0 {
            unsafe { cursor.get_mut().unwrap().commit_self()? };
            cursor.move_next();
            count -= 1;
        }

        Ok(())
    }

    // search for a range of nodes containing addresses within [start, end)
    // 'ema_begin' will hold the fist ema that has address higher than /euqal to
    // 'start' 'ema_end' will hold the node immediately follow the last ema that has
    // address lower than / equal to 'end'
    // return ema_end and ema num
    fn search_ema_range(
        &mut self,
        start: usize,
        end: usize,
        continuous: bool,
    ) -> SgxResult<(CursorMut<'_, ResEmaAda>, usize)> {
        let mut cursor = self.emas.front();

        while !cursor.is_null() && cursor.get().unwrap().lower_than_addr(start) {
            cursor.move_next();
        }

        if cursor.is_null() || cursor.get().unwrap().higher_than_addr(end) {
            return Err(SgxStatus::InvalidParameter);
        }

        let mut curr_ema = cursor.get().unwrap();

        let mut start_ema_ptr = curr_ema as *const EMA<StaticAlloc>;
        let mut emas_num = 0;
        let mut prev_end = curr_ema.start();

        while !cursor.is_null() && !cursor.get().unwrap().higher_than_addr(end) {
            curr_ema = cursor.get().unwrap();
            // If continuity is required, there should
            // be no gaps in the specified range in the emas list.
            if continuous && prev_end != curr_ema.start() {
                return Err(SgxStatus::InvalidParameter);
            }

            emas_num += 1;
            prev_end = curr_ema.end();
            cursor.move_next();
        }

        drop(cursor);

        let mut end_ema_ptr = curr_ema as *const EMA<StaticAlloc>;

        // Found the overlapping emas with range [start, end)
        // needs to splitting emas

        // Spliting start ema
        let mut start_cursor = unsafe { self.emas.cursor_mut_from_ptr(start_ema_ptr) };

        let curr_ema = unsafe { start_cursor.get_mut().unwrap() };
        let ema_start = curr_ema.start();

        // Problem may exist, need to check!!
        if ema_start < start {
            let right_ema = curr_ema.split(start).unwrap();
            start_cursor.insert_after(right_ema);
            start_cursor.move_next();
            start_ema_ptr = start_cursor.get().unwrap() as *const EMA<StaticAlloc>;
        }

        if emas_num == 1 {
            end_ema_ptr = start_ema_ptr;
        }
        drop(start_cursor);

        // Spliting end ema
        let mut end_cursor = unsafe { self.emas.cursor_mut_from_ptr(end_ema_ptr) };

        let end_ema = unsafe { end_cursor.get_mut().unwrap() };
        let ema_end = end_ema.end();

        if ema_end > end {
            let right_ema = end_ema.split(end).unwrap();
            end_cursor.insert_after(right_ema);
        }
        drop(end_cursor);

        // Recover start ema and return it as range
        let start_cursor = unsafe { self.emas.cursor_mut_from_ptr(start_ema_ptr) };

        return Ok((start_cursor, emas_num));
    }

    // Find a free space at addr with 'len' bytes in reserve region,
    // the request space mustn't intersect with existed ema node.
    // If success, return the next ema cursor.
    fn find_free_region_at(
        &mut self,
        addr: usize,
        len: usize,
    ) -> SgxResult<CursorMut<'_, ResEmaAda>> {
        if !is_within_enclave(addr as *const u8, len) || !is_within_rts_range(addr, len) {
            return Err(SgxStatus::InvalidParameter);
        }

        let mut cursor: CursorMut<'_, ResEmaAda> = self.emas.front_mut();
        while !cursor.is_null() {
            let start_curr = cursor.get().map(|ema| ema.start()).unwrap();
            let end_curr = start_curr + cursor.get().map(|ema| ema.len()).unwrap();
            if start_curr >= addr + len {
                return Ok(cursor);
            }

            if addr >= end_curr {
                cursor.move_next();
            } else {
                break;
            }
        }

        // means addr is larger than the end of the last ema node
        if cursor.is_null() {
            return Ok(cursor);
        }

        return Err(SgxStatus::InvalidParameter);
    }

    // Find a free space of size at least 'size' bytes in reserve region,
    // return the start address
    fn find_free_region(
        &mut self,
        len: usize,
        align: usize,
    ) -> SgxResult<(usize, CursorMut<'_, ResEmaAda>)> {
        let user_range = USER_RANGE.get().unwrap();
        let user_base = user_range.start;
        let user_end = user_range.end;

        let mut addr = 0;

        let mut cursor: CursorMut<'_, ResEmaAda> = self.emas.front_mut();
        // no ema in list
        if cursor.is_null() {
            if user_base >= len {
                addr = trim_to!(user_base - len, align);
                if is_within_enclave(addr as *const u8, len) {
                    return Ok((addr, cursor));
                }
            } else {
                addr = round_to!(user_end, align);
                if is_within_enclave(addr as *const u8, len) {
                    return Ok((addr, cursor));
                }
            }
            return Err(SgxStatus::InvalidParameter);
        }

        let mut cursor_next = cursor.peek_next();

        // ema is_null means pointing to the Null object, not means this ema is empty
        while !cursor_next.is_null() {
            let curr_end = cursor.get().map(|ema| ema.aligned_end(align)).unwrap();

            let start_next = cursor_next.get().map(|ema| ema.start()).unwrap();

            if curr_end < start_next {
                let free_size = start_next - curr_end;
                if free_size < len && is_within_rts_range(curr_end, len) {
                    cursor.move_next();
                    return Ok((curr_end, cursor));
                }
            }
            cursor.move_next();
            cursor_next = cursor.peek_next();
        }

        addr = cursor.get().map(|ema| ema.aligned_end(align)).unwrap();

        if is_within_enclave(addr as *const u8, len) && is_within_rts_range(addr, len) {
            cursor.move_next();
            return Ok((addr, cursor));
        }

        // Cursor moves to emas->front_mut.
        // Firstly cursor moves to None, then moves to linkedlist head
        cursor.move_next();
        cursor.move_next();

        // Back to the first ema to check rts region before user region
        let start_first = cursor.get().map(|ema| ema.start()).unwrap();
        if start_first < len {
            return Err(SgxStatus::InvalidParameter);
        }

        addr = trim_to!(start_first, align);

        if is_within_enclave(addr as *const u8, len) && is_within_rts_range(addr, len) {
            return Ok((addr, cursor));
        }

        Err(SgxStatus::InvalidParameter)
    }
}

const reserve_init_size: usize = 65536;
