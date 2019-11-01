use merklebtree::merklebtree::{MerkleBTree, Nodes};
use merklebtree::node::Node;
use merklebtree::traits::CalculateHash;
use ring::digest;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct search_result {
    pub key: String,
    pub version: i32,
    pub existed: bool,
}

#[derive(Clone, Debug)]
pub struct key_version {
    pub key: String,
    pub version: i32,
}

impl PartialEq for key_version {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl Eq for key_version {}

impl Ord for key_version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.key).cmp(&(other.key))
    }
}

impl PartialOrd for key_version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl CalculateHash for key_version {
    fn calculate(&self) -> String {
        let mut s1 = String::new();
        s1.push_str(self.key.as_str());
        s1.push_str("-");
        s1.push_str(self.version.to_string().as_str());

        let hash = digest::digest(&digest::SHA256, s1.as_ref());
        let hex = hex::encode(hash);
        hex
    }
}

pub struct Merklebtree {
    pub mbtree: MerkleBTree,
    pub nodes: Nodes<key_version>,
    pub root_node: Node<key_version>,
}

pub fn new_mbtree() -> Merklebtree {
    let mut nodes = Nodes {
        nodes_map: Default::default(),
        size: 0,
        root_id: 0,
        content_size: 0,
        next_id: 0,
        m: 0,
    };

    let mut tree = MerkleBTree::new_empty(5, &mut nodes);

    nodes.m = tree.m;

    Merklebtree {
        mbtree: tree,
        nodes,
        root_node: Node::new_empty(0),
    }
}

impl Merklebtree {
    pub fn search(&mut self, key: String) -> (search_result, Nodes<key_version>) {
        let (value, found, subnodes) = self.mbtree.get_clone(
            key_version {
                key: key.clone(),
                version: 0,
            },
            &mut self.nodes,
        );
        if found {
            return (
                search_result {
                    existed: true,
                    key: key,
                    version: value.version,
                },
                subnodes,
            );
        } else {
            return (
                search_result {
                    existed: false,
                    key: key,
                    version: -1,
                },
                subnodes,
            );
        }
    }

    pub fn delete(&mut self, key: String) -> Nodes<key_version> {
        let subnodes = self.mbtree.remove_clone(
            key_version {
                key: key,
                version: 0,
            },
            &mut self.nodes,
        );
        subnodes
    }

    pub fn build_with_key_value(&mut self, kv: key_version) -> Nodes<key_version> {
        println!("kv:{:?}", kv);
        let subnodes = self.mbtree.put_clone(
            key_version {
                key: kv.key,
                version: kv.version,
            },
            &mut self.nodes,
        );
        subnodes
    }
}
