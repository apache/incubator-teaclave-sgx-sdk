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

use crate::sys::cache::NodeRef;
use crate::sys::error::FsResult;
use crate::sys::host::HostFs;
use crate::sys::keys::{DeriveKey, KeyType};
use crate::sys::metadata::EncryptFlags;
use sgx_crypto::aes::gcm::{Aad, AesGcm, Nonce};
use sgx_types::error::SgxStatus;
use sgx_types::types::{Key128bit, Mac128bit};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::mem;
use std::sync::Arc;

#[macro_export]
macro_rules! impl_asref_slice {
    ($($t:ty;)*) => {$(
        impl ::core::convert::AsRef<[u8]> for $t {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                unsafe { &*(self as *const _ as *const [u8; ::core::mem::size_of::<$t>()]) }
            }
        }
    )*}
}

#[macro_export]
macro_rules! impl_asmut_slice {
    ($($t:ty;)*) => {$(
        impl ::core::convert::AsMut<[u8]> for $t {
            #[inline]
            fn as_mut(&mut self) -> &mut [u8] {
                unsafe { &mut *(self as *mut _ as *mut [u8; ::core::mem::size_of::<$t>()]) }
            }
        }
    )*}
}

// the key to encrypt the data or mht, and the gmac
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct GcmData {
    pub key: Key128bit,
    pub mac: Mac128bit,
}

impl GcmData {
    fn new(key: Key128bit, mac: Mac128bit) -> GcmData {
        GcmData { key, mac }
    }
}

pub const ROOT_MHT_PHY_NUM: u64 = 1;
pub const META_DATA_PHY_NUM: u64 = 0;

pub const NODE_SIZE: usize = 4096;
// for NODE_SIZE == 4096, we have 96 attached data nodes and 32 mht child nodes
// 3/4 of the node size is dedicated to data nodes
pub const ATTACHED_DATA_NODES_COUNT: u64 = ((NODE_SIZE / mem::size_of::<GcmData>()) * 3 / 4) as u64;
// 1/4 of the node size is dedicated to child mht nodes
pub const CHILD_MHT_NODES_COUNT: u64 = ((NODE_SIZE / mem::size_of::<GcmData>()) / 4) as u64;

#[derive(Clone, Debug)]
#[repr(C, packed)]
pub struct MhtNode {
    pub data: [GcmData; ATTACHED_DATA_NODES_COUNT as usize],
    pub mht: [GcmData; CHILD_MHT_NODES_COUNT as usize],
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct DataNode {
    pub data: [u8; NODE_SIZE],
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct EncryptedData {
    data: [u8; NODE_SIZE],
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct EncryptedNode {
    pub physical_number: u64,
    pub node_data: EncryptedData,
}

impl_struct_default! {
    MhtNode;
    DataNode;
    EncryptedData;
    EncryptedNode;
}

impl_asref_slice! {
    GcmData;
    MhtNode;
    DataNode;
    EncryptedData;
    EncryptedNode;
}

impl_asmut_slice! {
    GcmData;
    MhtNode;
    DataNode;
    EncryptedData;
    EncryptedNode;
}

impl_struct_ContiguousMemory! {
    GcmData;
    MhtNode;
    DataNode;
    EncryptedData;
    EncryptedNode;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum NodeType {
    Mht,
    Data,
}

#[derive(Clone, Debug)]
pub enum Node {
    Mht(MhtNode),
    Data(DataNode),
}

impl Node {
    fn new(node_type: NodeType) -> Node {
        match node_type {
            NodeType::Mht => Node::Mht(MhtNode::default()),
            NodeType::Data => Node::Data(DataNode::default()),
        }
    }

    fn get_gcm_data(&self, node_type: NodeType, index: usize) -> Option<&GcmData> {
        match self {
            Node::Mht(ref m) => match node_type {
                NodeType::Mht => Some(&m.mht[index]),
                NodeType::Data => Some(&m.data[index]),
            },
            Node::Data(_) => None,
        }
    }

    fn set_gcm_data(&mut self, node_type: NodeType, index: usize, gcm_data: GcmData) {
        match self {
            Node::Mht(ref mut m) => match node_type {
                NodeType::Mht => m.mht[index] = gcm_data,
                NodeType::Data => m.data[index] = gcm_data,
            },
            Node::Data(_) => (),
        }
    }
}

impl AsRef<[u8]> for Node {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match self {
            Node::Mht(mht) => mht.as_ref(),
            Node::Data(data) => data.as_ref(),
        }
    }
}

impl AsMut<[u8]> for Node {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        match self {
            Node::Mht(mht) => mht.as_mut(),
            Node::Data(data) => data.as_mut(),
        }
    }
}

impl_struct_ContiguousMemory! {
    Node;
}

#[derive(Clone, Debug)]
pub struct FileNode {
    pub node_type: NodeType,
    pub logic_number: u64,
    pub need_writing: bool,
    pub new_node: bool,
    pub encrypt_flags: EncryptFlags,
    pub ciphertext: EncryptedNode,
    pub plaintext: Node,
    pub parent: Option<FileNodeRef>,
}

impl FileNode {
    pub fn new(
        node_type: NodeType,
        logic_number: u64,
        physical_number: u64,
        encrypt_flags: EncryptFlags,
    ) -> FileNode {
        FileNode {
            node_type,
            logic_number,
            need_writing: false,
            new_node: true,
            encrypt_flags,
            ciphertext: EncryptedNode {
                physical_number,
                node_data: EncryptedData::default(),
            },
            plaintext: Node::new(node_type),
            parent: None,
        }
    }

    #[inline]
    pub fn new_ref(
        node_type: NodeType,
        logic_number: u64,
        physical_number: u64,
        encrypt_flags: EncryptFlags,
    ) -> FileNodeRef {
        Arc::new(RefCell::new(FileNode::new(
            node_type,
            logic_number,
            physical_number,
            encrypt_flags,
        )))
    }

    #[inline]
    pub fn build_ref(file_node: Self) -> FileNodeRef {
        Arc::new(RefCell::new(file_node))
    }

    #[inline]
    pub fn new_root(encrypt_flags: EncryptFlags) -> FileNode {
        Self::new(NodeType::Mht, 0, ROOT_MHT_PHY_NUM, encrypt_flags)
    }

    #[inline]
    pub fn new_root_ref(encrypt_flags: EncryptFlags) -> FileNodeRef {
        Self::new_ref(NodeType::Mht, 0, ROOT_MHT_PHY_NUM, encrypt_flags)
    }

    pub fn encrypt(&mut self, key: &Key128bit) -> FsResult<Mac128bit> {
        let parent = if !self.is_root_mht() {
            let parent = self.parent.as_ref().ok_or(SgxStatus::Unexpected)?;
            Some(parent)
        } else {
            None
        };

        let mac = if !self.encrypt_flags.is_integrity_only() {
            let mut aes = AesGcm::new(key, Nonce::zeroed(), Aad::empty())?;
            aes.encrypt(self.plaintext.as_ref(), self.ciphertext.node_data.as_mut())?
        } else {
            let mut aes = AesGcm::new(key, Nonce::zeroed(), Aad::from(&self.plaintext))?;
            let mac = aes.encrypt(&[], &mut [])?;
            self.ciphertext
                .node_data
                .as_mut()
                .copy_from_slice(self.plaintext.as_ref());
            mac
        };

        if let Some(parent) = parent {
            let index = match self.node_type {
                NodeType::Mht => (self.logic_number - 1) % CHILD_MHT_NODES_COUNT,
                NodeType::Data => self.logic_number % ATTACHED_DATA_NODES_COUNT,
            };
            parent.borrow_mut().plaintext.set_gcm_data(
                self.node_type,
                index as usize,
                GcmData::new(*key, mac),
            );
        }

        Ok(mac)
    }

    pub fn decrypt(&mut self, key: &Key128bit, mac: &Mac128bit) -> FsResult {
        if !self.encrypt_flags.is_integrity_only() {
            let mut aes = AesGcm::new(key, Nonce::zeroed(), Aad::empty())?;
            aes.decrypt(
                self.ciphertext.node_data.as_ref(),
                self.plaintext.as_mut(),
                mac,
            )?
        } else {
            let mut aes = AesGcm::new(key, Nonce::zeroed(), Aad::from(&self.ciphertext.node_data))?;
            aes.decrypt(&[], &mut [], mac)?;
            self.plaintext
                .as_mut()
                .copy_from_slice(self.ciphertext.node_data.as_ref());
        };

        Ok(())
    }

    pub fn derive_key(&mut self, derive: &mut dyn DeriveKey) -> FsResult<Key128bit> {
        if !self.encrypt_flags.is_integrity_only() {
            let (key, _) = derive.derive_key(KeyType::Random, self.ciphertext.physical_number)?;
            Ok(key)
        } else {
            Ok(Key128bit::default())
        }
    }

    #[inline]
    pub fn read_from_disk(&mut self, file: &mut dyn HostFs) -> FsResult {
        let physical_number = self.ciphertext.physical_number;
        assert!(physical_number != 0);

        file.read(physical_number, &mut self.ciphertext.node_data)
    }

    #[inline]
    pub fn write_to_disk(&mut self, file: &mut dyn HostFs) -> FsResult {
        let physical_number = self.ciphertext.physical_number;
        assert!(physical_number != 0);

        file.write(physical_number, &self.ciphertext.node_data)?;
        self.need_writing = false;
        self.new_node = false;
        Ok(())
    }

    #[inline]
    pub fn write_recovery_file(&self, file: &mut dyn HostFs) -> FsResult {
        let physical_number = self.ciphertext.physical_number;
        assert!(physical_number != 0);

        file.write(physical_number, &self.ciphertext)
    }

    #[inline]
    pub fn is_mht(&self) -> bool {
        self.node_type == NodeType::Mht
    }

    #[inline]
    pub fn is_data(&self) -> bool {
        self.node_type == NodeType::Data
    }

    #[inline]
    pub fn is_root_mht(&self) -> bool {
        self.logic_number == 0 && self.ciphertext.physical_number == ROOT_MHT_PHY_NUM
    }

    #[inline]
    pub fn gcm_index(&self) -> Option<usize> {
        if self.is_root_mht() {
            return None;
        }

        assert!(self.ciphertext.physical_number > ROOT_MHT_PHY_NUM);
        Some(match self.node_type {
            NodeType::Mht => (self.logic_number - 1) % CHILD_MHT_NODES_COUNT,
            NodeType::Data => self.logic_number % ATTACHED_DATA_NODES_COUNT,
        } as usize)
    }

    #[inline]
    pub fn get_gcm_data(&self) -> Option<GcmData> {
        self.parent
            .as_ref()?
            .borrow()
            .plaintext
            .get_gcm_data(self.node_type, self.gcm_index()?)
            .cloned()
    }

    #[allow(dead_code)]
    #[inline]
    pub fn set_gcm_data(&mut self, gcm_data: GcmData) {
        if let Some((index, parent)) = self.gcm_index().zip(self.parent.as_ref()) {
            parent
                .borrow_mut()
                .plaintext
                .set_gcm_data(self.node_type, index, gcm_data)
        }
    }
}

impl PartialOrd for FileNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileNode {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_number = self.ciphertext.physical_number;
        let other_number = other.ciphertext.physical_number;
        self_number.cmp(&other_number)
    }
}

impl PartialEq for FileNode {
    fn eq(&self, other: &Self) -> bool {
        self.ciphertext.physical_number == other.ciphertext.physical_number
    }
}

impl Eq for FileNode {}

impl Drop for FileNode {
    fn drop(&mut self) {
        self.plaintext.as_mut().fill(0)
    }
}

pub type FileNodeRef = NodeRef<FileNode>;
