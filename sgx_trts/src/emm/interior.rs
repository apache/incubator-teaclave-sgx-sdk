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
use intrusive_collections::singly_linked_list::CursorMut;
use intrusive_collections::singly_linked_list::{Link, SinglyLinkedList};
use intrusive_collections::UnsafeRef;
use intrusive_collections::{LinkedList, LinkedListLink};

use crate::sync::SpinMutex;
use core::mem::size_of;
use core::mem::transmute;
use core::mem::MaybeUninit;
use spin::{Mutex, Once};

use super::flags::AllocFlags;
use super::range::{RangeType, RM};
use sgx_types::error::{SgxResult, SgxStatus};

// fixed static memory size
const STATIC_MEM_SIZE: usize = 65536;
// initial interior reserve memory size
const INIT_MEM_SIZE: usize = 65536;
// the size of guard for interior memory
const GUARD_SIZE: usize = 0x8000;
// this is enough for bit map of an 8T EMA
const MAX_EMALLOC_SIZE: usize = 0x10000000;

const ALLOC_MASK: usize = 1;
const SIZE_MASK: usize = !(EXACT_MATCH_INCREMENT - 1);

// the increment size for interior memory
static mut INCR_SIZE: usize = 65536;

/// first level: static memory
pub static STATIC: LockedHeap<32> = LockedHeap::empty();

#[derive(Clone)]
#[repr(u8)]
pub enum Alloc {
    Static,
    Reserve,
}

static mut STATIC_MEM: [u8; STATIC_MEM_SIZE] = [0; STATIC_MEM_SIZE];

pub fn init_static_alloc() {
    unsafe {
        STATIC
            .lock()
            .init(STATIC_MEM.as_ptr() as usize, STATIC_MEM_SIZE);
    }
}

/// second level: reserve memory
/// Some problem here!
// static RES_ALLOCATOR: Once<Mutex<Reserve>> = Once::new();
pub static RES_ALLOCATOR: Once<Mutex<Reserve>> = Once::new();
pub static GLOBAL_LOCK: Once<SpinMutex<()>> = Once::new();

pub fn init_global_lock() {
    GLOBAL_LOCK.call_once(|| SpinMutex::new(()));
}

pub fn init_reserve_alloc() -> SgxResult {
    let reserve = Mutex::new(Reserve::new(1024)?);
    // res_allocator need to be intialized after static_allocator
    RES_ALLOCATOR.call_once(|| reserve);
    Ok(())
}

// mm_reserve
struct Chunk {
    base: usize,
    size: usize,
    used: usize,
    link: Link, // singly intrusive linkedlist
}

impl Chunk {
    fn new(base: usize, size: usize) -> Self {
        Self {
            base,
            size,
            used: 0,
            link: Link::new(),
        }
    }
}

intrusive_adapter!(ChunkAda = UnsafeRef<Chunk>: Chunk { link: LinkedListLink });

const NUM_EXACT_LIST: usize = 0x100;
const HEADER_SIZE: usize = size_of::<usize>();
const EXACT_MATCH_INCREMENT: usize = 0x8;
const MIN_BLOCK_SIZE: usize = 0x10;
const MAX_EXACT_SIZE: usize = MIN_BLOCK_SIZE + EXACT_MATCH_INCREMENT * (NUM_EXACT_LIST - 1);

enum Block {
    Free(BlockFree),
    Used(BlockUsed),
}

// free block for allocationg exact size
#[repr(C)]
struct BlockFree {
    size: usize,
    link: LinkedListLink, // doubly intrusive linkedlist
}

// used block for tracking allocated size and base address
#[repr(C)]
struct BlockUsed {
    size: usize,
    payload: usize,
}

impl BlockFree {
    fn new(size: usize) -> Self {
        Self {
            size,
            link: LinkedListLink::new(),
        }
    }

    fn block_size(&self) -> usize {
        self.size & SIZE_MASK
    }
}

impl BlockUsed {
    fn new(size: usize) -> Self {
        Self { size, payload: 0 }
    }
}

intrusive_adapter!(BlockFreeAda = UnsafeRef<BlockFree>: BlockFree { link: LinkedListLink });

pub struct Reserve {
    // Each block manage an area of free memory
    exact_blocks: [LinkedList<BlockFreeAda>; 256],
    large_blocks: LinkedList<BlockFreeAda>,

    // Each chunk manage an area of memory allocated by one EMA
    chunks: SinglyLinkedList<ChunkAda>,

    // statistics
    allocated: usize,
    total: usize,
}

impl Reserve {
    pub fn new(size: usize) -> SgxResult<Self> {
        let exact_blocks: [LinkedList<BlockFreeAda>; 256] = {
            let mut exact_blocks: [MaybeUninit<LinkedList<BlockFreeAda>>; 256] =
                MaybeUninit::uninit_array();
            for block in &mut exact_blocks {
                block.write(LinkedList::new(BlockFreeAda::new()));
            }
            unsafe { transmute(exact_blocks) }
        };

        let mut reserve = Self {
            exact_blocks,
            large_blocks: LinkedList::new(BlockFreeAda::new()),
            chunks: SinglyLinkedList::new(ChunkAda::new()),
            allocated: 0,
            total: 0,
        };
        unsafe {
            reserve.add_reserve(size)?;
        }
        Ok(reserve)
    }

    fn get_free_block(&mut self, bsize: usize) -> Option<UnsafeRef<BlockFree>> {
        if bsize <= MAX_EXACT_SIZE {
            return self.get_exact_block(bsize);
        }

        // loop and find the most available large block
        let list = &mut self.large_blocks;
        let mut cursor = list.front_mut();
        let mut suit_block: Option<*const BlockFree> = None;
        let mut suit_block_size = 0;
        while !cursor.is_null() {
            let curr_block = cursor.get().unwrap();
            if curr_block.size >= bsize {
                if suit_block.is_none() {
                    suit_block = Some(curr_block as *const BlockFree);
                    suit_block_size = curr_block.size;
                } else if suit_block_size > curr_block.size {
                    suit_block = Some(curr_block as *const BlockFree);
                    suit_block_size = curr_block.size;
                }
            }
            cursor.move_next();
        }

        if suit_block.is_none() {
            return None;
        }

        let mut cursor = unsafe { list.cursor_mut_from_ptr(suit_block.unwrap()) };
        let suit_block = cursor.remove();

        // TODO: split suit block
        return suit_block;
    }

    fn get_exact_block(&mut self, bsize: usize) -> Option<UnsafeRef<BlockFree>> {
        let idx = self.get_list_idx(bsize);
        let list = &mut self.exact_blocks[idx];
        list.pop_front()
    }

    fn put_free_block(&mut self, block: UnsafeRef<BlockFree>) {
        let block_size = block.block_size();
        if block_size <= MAX_EXACT_SIZE {
            // put block into list with exact block size
            let idx = self.get_list_idx(block_size);
            let list = &mut self.exact_blocks[idx];
            list.push_back(block);
        } else {
            // put block into list with large block size
            let list = &mut self.large_blocks;
            list.push_back(block);
        }
    }

    // merge next free block into this block
    fn reconfigure_block(&self, _block: UnsafeRef<BlockFree>) -> UnsafeRef<BlockFree> {
        todo!()
    }

    /// Obtain the list index for exact block size
    fn get_list_idx(&self, size: usize) -> usize {
        assert!(size % EXACT_MATCH_INCREMENT == 0);
        if size < MIN_BLOCK_SIZE {
            return 0;
        }
        let idx = (size - MIN_BLOCK_SIZE) / EXACT_MATCH_INCREMENT;
        assert!(idx < NUM_EXACT_LIST);
        return idx;
    }

    // Attention! If we need to clear the memory
    fn block_to_payload(&self, block: UnsafeRef<BlockFree>) -> usize {
        let block_size = block.size;
        let block_used = BlockUsed::new(block_size);
        let ptr = UnsafeRef::into_raw(block) as *mut BlockUsed;
        let payload_addr = unsafe {
            ptr.write(block_used);
            ptr.offset(HEADER_SIZE as isize) as usize
        };
        return payload_addr;
    }

    fn payload_to_block(&self, payload_addr: usize) -> UnsafeRef<BlockFree> {
        // payload shift to block_use
        let payload_ptr = payload_addr as *const u8;
        let block_used_ptr =
            unsafe { payload_ptr.offset(-(HEADER_SIZE as isize)) as *mut BlockUsed };

        let block_size = unsafe { block_used_ptr.read().size };

        let block_free = BlockFree::new(block_size);
        let ptr = block_used_ptr as *mut BlockFree;
        unsafe {
            ptr.write(block_free);
            UnsafeRef::from_raw(ptr)
        }
    }

    pub fn emalloc(&mut self, size: usize) -> SgxResult<usize> {
        let mut bsize = round_to!(size + HEADER_SIZE, EXACT_MATCH_INCREMENT);
        bsize = bsize.max(MIN_BLOCK_SIZE);

        // find free block in lists
        let mut block = self.get_free_block(bsize);

        match block {
            Some(mut block) => {
                let block_size = bsize | ALLOC_MASK;
                block.size = block_size;
                return Ok(self.block_to_payload(block));
            }
            None => (),
        };

        block = self.alloc_from_reserve(bsize);
        if block.is_none() {
            let chunk_size = size_of::<Chunk>();
            let new_reserve_size = round_to!(bsize + chunk_size, INIT_MEM_SIZE);
            unsafe { self.add_reserve(new_reserve_size)? };
            block = self.alloc_from_reserve(bsize);
            // should never happen
            if block.is_none() {
                return Err(SgxStatus::InvalidParameter);
            }
        }

        let mut block = block.unwrap();

        block.size = bsize | ALLOC_MASK;
        return Ok(self.block_to_payload(block));
    }

    fn alloc_from_reserve(&mut self, bsize: usize) -> Option<UnsafeRef<BlockFree>> {
        let mut addr: usize = 0;
        let mut cursor = self.chunks.front_mut();
        while !cursor.is_null() {
            let chunk = unsafe { cursor.get_mut().unwrap() };
            if chunk.size - chunk.used >= bsize {
                addr = chunk.base + chunk.used;
                chunk.used += bsize;
                break;
            }
            cursor.move_next();
        }

        if addr == 0 {
            return None;
        } else {
            let block = BlockFree::new(bsize);
            let ptr = addr as *mut BlockFree;
            let block = unsafe {
                ptr.write(block);
                UnsafeRef::from_raw(ptr)
            };
            return Some(block);
        }
    }

    pub fn efree(&mut self, payload_addr: usize) {
        let block = self.payload_to_block(payload_addr);
        let block_addr = block.as_ref() as *const BlockFree as usize;
        let block_size = block.block_size();
        let block_end = block_addr + block_size;
        let res = self.find_chunk_with_block(block_addr, block_size);
        if res.is_err() {
            panic!();
        }
        // reconfigure block
        let mut cursor = res.unwrap();
        let chunk = unsafe { cursor.get_mut().unwrap() };

        if block_end - chunk.base == chunk.used {
            chunk.used -= block.size;
        }

        // TODO: merge large block into reserve

        self.put_free_block(block);
    }

    /// Adding the size of interior memory
    /// rsize: memory increment
    pub unsafe fn add_reserve(&mut self, rsize: usize) -> SgxResult {
        // Here we alloc at least INIT_MEM_SIZE size,
        // but commit rsize memory, the remaining memory is COMMIT_ON_DEMAND
        let increment = INCR_SIZE.max(rsize);
        let mut range_manage = RM.get().unwrap().lock();
        let base = range_manage.alloc(
            None,
            increment + 2 * GUARD_SIZE,
            AllocFlags::RESERVED.bits() as usize,
            None,
            None,
            RangeType::User,
            Alloc::Static,
        )?;

        let base = range_manage.alloc(
            Some(base + GUARD_SIZE),
            increment,
            (AllocFlags::COMMIT_ON_DEMAND | AllocFlags::FIXED).bits() as usize,
            None,
            None,
            RangeType::User,
            Alloc::Static,
        )?;

        range_manage.commit(base, rsize, RangeType::User)?;
        drop(range_manage);

        unsafe {
            self.write_chunk(base, increment);
        }

        INCR_SIZE = INCR_SIZE * 2;
        if INCR_SIZE > MAX_EMALLOC_SIZE {
            INCR_SIZE = MAX_EMALLOC_SIZE
        };

        Ok(())
    }

    unsafe fn write_chunk(&mut self, base: usize, size: usize) {
        let chunk: Chunk = Chunk::new(base, size);
        unsafe {
            core::ptr::write(base as *mut Chunk, chunk);
            let chunk_ref = UnsafeRef::from_raw(base as *const Chunk);
            self.chunks.push_front(chunk_ref);
        }
    }

    // find the chunk including the specified block: fn find_used_in_reserve
    fn find_chunk_with_block(
        &mut self,
        block_addr: usize,
        block_size: usize,
    ) -> SgxResult<CursorMut<'_, ChunkAda>> {
        if block_size == 0 {
            return Err(SgxStatus::InvalidParameter);
        }
        let mut cursor = self.chunks.front_mut();
        while !cursor.is_null() {
            let chunk = cursor.get().unwrap();
            if block_addr >= chunk.base && block_addr + block_size <= chunk.base + chunk.used {
                return Ok(cursor);
            }
            cursor.move_next();
        }

        return Err(SgxStatus::InvalidParameter);
    }
}
