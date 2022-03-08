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

use crate::sys::error::FsResult;
use crate::sys::file::{FileInner, FileStatus};
use crate::sys::metadata::MD_USER_DATA_SIZE;
use crate::sys::node::{FileNode, FileNodeRef, NodeType};
use crate::sys::node::{ATTACHED_DATA_NODES_COUNT, CHILD_MHT_NODES_COUNT, NODE_SIZE};
use sgx_types::error::SgxStatus;

impl FileInner {
    pub fn get_data_node(&mut self) -> FsResult<FileNodeRef> {
        ensure!(
            self.offset >= MD_USER_DATA_SIZE,
            esgx!(SgxStatus::Unexpected)
        );

        let data_node = if ((self.offset - MD_USER_DATA_SIZE) % NODE_SIZE == 0)
            && (self.offset == self.metadata.encrypted_plain.size)
        {
            self.append_data_node()
        } else {
            self.read_data_node()
        };

        // bump all the parents mht to reside before the data node in the cache
        if let Ok(ref data_node) = data_node {
            self.bump_mht_node(data_node);
        }

        // even if we didn't get the required data_node, we might have read other nodes in the process
        self.shrink_cache()?;
        data_node
    }

    fn get_mht_node(&mut self) -> FsResult<FileNodeRef> {
        ensure!(
            self.offset >= MD_USER_DATA_SIZE,
            esgx!(SgxStatus::Unexpected)
        );

        let (logic_number, _) = self.get_mht_node_numbers();
        if logic_number == 0 {
            return Ok(self.root_mht.clone());
        }

        if ((self.offset - MD_USER_DATA_SIZE) % (ATTACHED_DATA_NODES_COUNT as usize * NODE_SIZE)
            == 0)
            && self.offset == self.metadata.encrypted_plain.size
        {
            self.append_mht_node(logic_number)
        } else {
            self.read_mht_node(logic_number)
        }
    }

    fn append_mht_node(&mut self, logic_number: u64) -> FsResult<FileNodeRef> {
        let parent_mht_node = self.read_mht_node((logic_number - 1) / CHILD_MHT_NODES_COUNT)?;

        // the '1' is for the meta data node
        // ATTACHED_DATA_NODES_COUNT + 1 (the '1' is for the mht node preceding every 96 data nodes)
        let physical_number = 1 + logic_number * (ATTACHED_DATA_NODES_COUNT + 1);
        let mut mht_node = FileNode::new(
            NodeType::Mht,
            logic_number,
            physical_number,
            self.metadata.encrypt_flags(),
        );
        mht_node.parent = Some(parent_mht_node);

        let mht_node = FileNode::build_ref(mht_node);
        ensure!(
            self.cache.push(physical_number, mht_node.clone()),
            esgx!(SgxStatus::Unexpected)
        );
        Ok(mht_node)
    }

    fn read_mht_node(&mut self, logic_number: u64) -> FsResult<FileNodeRef> {
        if logic_number == 0 {
            return Ok(self.root_mht.clone());
        }
        // the '1' is for the meta data node
        // ATTACHED_DATA_NODES_COUNT + 1 (the '1' is for the mht node preceding every 96 data nodes)
        let physical_number = 1 + logic_number * (ATTACHED_DATA_NODES_COUNT + 1);

        if let Some(mht_node) = self.cache.find(physical_number) {
            return Ok(mht_node);
        }

        let parent_mht_node = self.read_mht_node((logic_number - 1) / CHILD_MHT_NODES_COUNT)?;
        let mut mht_node = FileNode::new(
            NodeType::Mht,
            logic_number,
            physical_number,
            self.metadata.encrypt_flags(),
        );
        mht_node.parent = Some(parent_mht_node);

        mht_node.read_from_disk(&mut self.host_file)?;

        let gcm_data = mht_node.get_gcm_data().ok_or(SgxStatus::Unexpected)?;
        mht_node.decrypt(&gcm_data.key, &gcm_data.mac)?;

        let mht_node = FileNode::build_ref(mht_node);
        ensure!(
            self.cache.push(physical_number, mht_node.clone()),
            esgx!(SgxStatus::Unexpected)
        );
        Ok(mht_node)
    }

    fn append_data_node(&mut self) -> FsResult<FileNodeRef> {
        let mht_node = self.get_mht_node()?;
        let (logic_number, physical_number) = self.get_data_node_numbers();

        let mut data_node = FileNode::new(
            NodeType::Data,
            logic_number,
            physical_number,
            self.metadata.encrypt_flags(),
        );
        data_node.parent = Some(mht_node);

        let data_node = FileNode::build_ref(data_node);
        ensure!(
            self.cache.push(physical_number, data_node.clone()),
            esgx!(SgxStatus::Unexpected)
        );
        Ok(data_node)
    }

    fn read_data_node(&mut self) -> FsResult<FileNodeRef> {
        let (logic_number, physical_number) = self.get_data_node_numbers();

        if let Some(data_node) = self.cache.find(physical_number) {
            return Ok(data_node);
        }

        let mht_node = self.get_mht_node()?;
        let mut data_node = FileNode::new(
            NodeType::Data,
            logic_number,
            physical_number,
            self.metadata.encrypt_flags(),
        );
        data_node.parent = Some(mht_node);

        data_node.read_from_disk(&mut self.host_file)?;

        let gcm_data = data_node.get_gcm_data().ok_or(SgxStatus::Unexpected)?;
        data_node.decrypt(&gcm_data.key, &gcm_data.mac)?;

        let data_node = FileNode::build_ref(data_node);
        ensure!(
            self.cache.push(physical_number, data_node.clone()),
            esgx!(SgxStatus::Unexpected)
        );
        Ok(data_node)
    }

    fn bump_mht_node(&mut self, node: &FileNodeRef) {
        let mut parent = node.borrow().parent.clone();
        while let Some(mht) = parent {
            let mht = mht.borrow();
            if !mht.is_root_mht() {
                self.cache.move_to_head(mht.ciphertext.physical_number);
                parent = mht.parent.clone();
            } else {
                break;
            }
        }
    }

    fn shrink_cache(&mut self) -> FsResult {
        while self.cache.len() > super::MAX_PAGES_IN_CACHE {
            let node = self.cache.back().ok_or(SgxStatus::Unexpected)?;
            if !node.borrow().need_writing {
                let _node = self.cache.pop_back();
            } else {
                self.internal_flush(false).map_err(|error| {
                    if self.status.is_ok() {
                        self.set_file_status(FileStatus::FlushError);
                    }
                    error
                })?;
            }
        }
        Ok(())
    }

    fn get_node_numbers(&self) -> (u64, u64, u64, u64) {
        if self.offset < MD_USER_DATA_SIZE {
            return (0, 0, 0, 0);
        }

        // node 0 - meta data node
        // node 1 - mht
        // nodes 2-97 - data (ATTACHED_DATA_NODES_COUNT == 96)
        // node 98 - mht
        // node 99-195 - data
        // etc.
        let data_logic_number = ((self.offset - MD_USER_DATA_SIZE) / NODE_SIZE) as u64;
        let mht_logic_number = data_logic_number / ATTACHED_DATA_NODES_COUNT;

        // + 1 - meta data node
        // + 1 - mht root
        // + mht_logic_number - number of mht nodes in the middle (the root mht mht_node_number is 0)
        let data_physical_number = data_logic_number + 1 + 1 + mht_logic_number;

        let mht_physical_number =
            data_physical_number - data_logic_number % ATTACHED_DATA_NODES_COUNT - 1;

        (
            mht_logic_number,
            data_logic_number,
            mht_physical_number,
            data_physical_number,
        )
    }

    #[inline]
    fn get_data_node_numbers(&self) -> (u64, u64) {
        let (_, logic, _, physical) = self.get_node_numbers();
        (logic, physical)
    }

    #[inline]
    fn get_mht_node_numbers(&self) -> (u64, u64) {
        let (logic, _, physical, _) = self.get_node_numbers();
        (logic, physical)
    }
}
