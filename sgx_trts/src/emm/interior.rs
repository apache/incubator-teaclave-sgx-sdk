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
use intrusive_collections::{LinkedList, LinkedListLink};

use alloc::boxed::Box;
use core::alloc::Layout;
use core::ffi::c_void;
use core::mem::transmute;
use core::mem::MaybeUninit;
use core::ptr::NonNull;
use spin::{Mutex, Once};

use sgx_types::error::{SgxResult, SgxStatus};
use sgx_types::types::ProtectPerm;

use crate::emm::ema::EMA;
use crate::emm::user::{USER_RANGE, self, is_within_user_range};
use crate::enclave::is_within_enclave;

use super::ema::ResEmaAda;

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
    pub base: usize,
    pub size: usize,
    pub used: usize,
    link: LinkedListLink, // intrusive linkedlist
}

intrusive_adapter!(ChunkAda = Box<Chunk>: Chunk { link: LinkedListLink });
// let linkedlist = LinkedList::new(ResChunk_Adapter::new());

// mm_reserve
struct Block {
    size: usize,
    link: LinkedListLink, // intrusive linkedlist
}
// 或许在某些情况里也不需要link。

intrusive_adapter!(BlockAda = Box<Block>: Block { link: LinkedListLink });

pub struct Reserve {
    // 这些list是block list，每个block用于存放如 ema meta / bitmap meta / bitmap data
    exact_blocks: [LinkedList<BlockAda>; 256],
    large_blocks: LinkedList<BlockAda>,

    // chunks 这个结构体是存放于reserve EMA分配的reserve内存
    chunks: LinkedList<ChunkAda>,
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

    // Find a free space of size at least 'size' bytes in reserve region,
    // return the start address
    fn find_free_region(&mut self, len: usize, align: usize) -> SgxResult<usize> {
        let user_range = USER_RANGE.get().unwrap();
        let user_base = user_range.start;
        let user_end = user_range.end;

        // no ema in list
        if self.emas.is_empty() {
            let mut addr = 0;

            if user_base >= len {
                addr = trim_to!(user_base - len, align);
                if is_within_enclave(addr as *const u8, len) {
                    return Ok(addr);
                }
            } else {
                addr = round_to!(user_end, align);
                if is_within_enclave(addr as *const u8, len) {
                    return Ok(addr);
                }
            }
            return Err(SgxStatus::InvalidParameter);
        }


        let mut cursor = self.emas.cursor_mut();
        while !cursor.is_null() {
            let curr_end = cursor.get()
                .map(|ema| ema.aligned_end(align)).unwrap();

            cursor.move_next();
            if cursor.is_null() {
                break;
            }
            
            let next_start = cursor.get()
            .map(|ema| ema.start()).unwrap();
            
            if curr_end < next_start {
                let free_size = next_start - curr_end;
                // 这里或许得用is_within_rts
                if free_size < len && is_within_enclave(curr_end as *const u8, len){
                    return Ok(curr_end);
                }
            }
            cursor.move_next();
        }


        todo!()
    }
}

const reserve_init_size: usize = 65536;
