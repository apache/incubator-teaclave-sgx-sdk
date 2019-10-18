use crate::client::*;
use crate::verifytree::mbtree::*;
use hex;
use ring::hmac::Key;
use merklebtree::node::Node;
use merklebtree::traits::CalculateHash;
use parking_lot::RwLock;
use ring::{digest, hmac, rand};
use rocksdb::{DBVector, DB};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub struct server {
    db_handler: Arc<RwLock<DB>>,
    sgx_counter: i32,
    hmac_key: Key,
    present_mbtree: Merklebtree,
    deleted_mbtree: Merklebtree,
}

//hmacPayload is used to compute hmac
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct hmac_payload {
    key: String,
    value: String,
    counter: i32,
}

//storePayload is used to store value in the db
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct store_payload {
    value: String,
    tag: String,
    ctr: i32,
}

#[derive(Clone, Debug)]
pub struct sgx_private_data {
    hmac_key: Key,
    sgx_counter: i32,
    persisted_present_hash: String,
    persisted_present_hmac: String,
    persisted_deleted_hash: String,
    persisted_deleted_hmac: String,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct mbtree_payload {
    root_hash: String,
    sgx_counter: i32,
}

impl server {
    fn put(&mut self, key: String, value: String) {
        let db = self.db_handler.clone();
        db.write().put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    fn get(&mut self, key: String) -> String {
        let db = self.db_handler.clone();
        let db_read = db.read();
        let r = db.read().get(key.as_str());
        match r {
            Err(t) => return String::new(),
            Ok(t) => match t {
                None => return String::new(),
                Some(t) => return t.to_utf8().unwrap().to_string(),
            },
        }
    }

    fn delete(&mut self, key: String) {
        let db = self.db_handler.clone();
        db.write().delete(key.as_bytes()).unwrap();
    }
}

pub fn new_server(key_value: Vec<u8>) -> server {
    let path = "rocksdb";
    let db = DB::open_default(path).unwrap();

    let db_handler = Arc::new(RwLock::new(db));

    let s_key = hmac::Key::new(hmac::HMAC_SHA256, key_value.as_ref());

    server {
        db_handler,
        sgx_counter: 0,
        hmac_key: s_key,
        present_mbtree: new_mbtree(),
        deleted_mbtree: new_mbtree(),
    }
}

impl server {
    pub fn handle_req(&mut self, req: request) -> response {
        //TODO:safecheck for the insecurity mbtree should be added
        let mut op_status = true;
        let mut get_result = String::new();
        if req.req_type == String::from("put") {
            self.veritasdb_put(req.clone());
        } else if req.req_type == String::from("get") {
            get_result = self.veritasdb_get(req.clone());
        } else if req.req_type == String::from("delete") {
            self.veritasdb_delete(req.clone());
        } else if req.req_type == String::from("insert") {
            self.veritasdb_insert(req.clone());
        }

        response {
            rsp_status: op_status,
            req_type: req.req_type,
            data: get_result,
            error_info: "".to_string(),
        }
    }

    pub fn veritasdb_get(&mut self, req: request) -> String {
        let data = self.get(req.key.clone());
        if data == "".to_string() {
            return String::new();
        }

        let sp: store_payload = serde_json::from_str(data.as_str()).unwrap();
        let hmac_data = hmac_payload {
            key: req.key.clone(),
            value: sp.value.clone(),
            counter: sp.ctr,
        };
        let hmac_string = serde_json::to_string(&hmac_data).unwrap();
        let verify_result = self.verify_hmac(hmac_string, sp.tag.clone());

        let sr = self.present_mbtree.search(req.key.clone());
        if verify_result && sp.ctr == sr.version {
            return sp.value;
        } else {
            panic!("verify failed");
        }
        String::new();
    }

    pub fn veritasdb_put(&mut self, req: request) {
        let get_result = self.present_mbtree.search(req.key.clone());
        if get_result.existed {
            let hmac_data = hmac_payload {
                key: req.key.clone(),
                value: req.value.clone(),
                counter: get_result.version + 1,
            };
            let hmac_string = serde_json::to_string(&hmac_data).unwrap();
            let tag_string = self.compute_hmac(hmac_string);

            //try to put it into kvdb
            let store_data = store_payload {
                value: req.value.clone().to_string(),
                tag: tag_string,
                ctr: get_result.version + 1,
            };
            let store_string = serde_json::to_string(&store_data).unwrap();
            self.put(req.key.clone(), store_string.clone());

            //update present if there is no error
            self.present_mbtree.build_with_key_value(key_version {
                key: req.key,
                version: get_result.version + 1,
            });
        } else {
            println!("key doesn't exist in present when called put");
            return;
        }
    }

    pub fn veritasdb_insert(&mut self, req: request) {
        let sr = self.present_mbtree.search(req.key.clone());
        if sr.existed {
            println!("key existed in present when called insert");
            return;
        } else {
            let mut ctr = 0;
            let delete_sr = self.deleted_mbtree.search(req.key.clone());
            if delete_sr.existed {
                ctr = delete_sr.version + 1;
            } else {
                ctr = 0;
            }
            let hmac_data = hmac_payload {
                key: req.key.clone(),
                value: req.value.clone(),
                counter: ctr,
            };
            let hmac_string = serde_json::to_string(&hmac_data).unwrap();
            let tag_string = self.compute_hmac(hmac_string);

            let store_data = store_payload {
                value: req.value,
                ctr: ctr,
                tag: tag_string,
            };
            let store_string = serde_json::to_string(&store_data).unwrap();
            self.put(req.key.clone(), store_string);
            self.present_mbtree.build_with_key_value(key_version {
                key: req.key.clone(),
                version: ctr,
            });
            self.deleted_mbtree.delete(req.key.clone());
        }
    }

    pub fn veritasdb_delete(&mut self, req: request) {
        let get_result = self.get(req.key.clone());
        if get_result == "".to_string() {
        } else {
            self.delete(req.key.clone());
            let sr = self.present_mbtree.search(req.key.clone());
            self.deleted_mbtree.build_with_key_value(key_version {
                key: req.key.clone(),
                version: sr.version,
            });
            self.present_mbtree.delete(req.key);
        }
    }

    pub fn compute_hmac(&mut self, msg: String) -> String {
        let tag = hmac::sign(&self.hmac_key, msg.as_str().as_bytes());
        let tag_string = hex::encode(tag.as_ref());
        tag_string
    }

    pub fn verify_hmac(&mut self, msg: String, tag_string: String) -> bool {
        let result = hmac::verify(
            &self.hmac_key,
            msg.as_bytes(),
            hex::decode(tag_string).unwrap().as_ref(),
        );
        match result {
            Ok(t) => return true,
            Err(e) => return false,
        }
    }
}
