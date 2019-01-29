use std::prelude::v1::*;
use std::str;
use std::time::*;
use std::untrusted::time::SystemTimeEx;

use sgx_tcrypto::*;
use sgx_types::*;

use num_bigint::BigUint;
use bit_vec::BitVec;
use yasna::models::ObjectIdentifier;
use yasna::writer::PC;
use yasna::tags::{TAG_UTCTIME, TAG_UTF8STRING};
use chrono::prelude::*;
use chrono::offset::Utc;
use chrono::Duration;

const ISSUER : &str = "MesaTEE";
const SUBJECT : &str = "MesaTEE";

pub fn gen_ecc_cert(payload: String,
                    prv_k: &sgx_ec256_private_t,
                    pub_k: &sgx_ec256_public_t,
                    ecc_handle: &SgxEccHandle) -> Result<(Vec<u8>, Vec<u8>), sgx_status_t> {
    // Generate public key bytes since both DER will use it
    let mut pub_key_bytes: Vec<u8> = vec![4];
    let mut pk_gx = pub_k.gx.clone();
    pk_gx.reverse();
    let mut pk_gy = pub_k.gy.clone();
    pk_gy.reverse();
    pub_key_bytes.extend_from_slice(&pk_gx);
    pub_key_bytes.extend_from_slice(&pk_gy);


    // Generate Certificate DER
    let cert_der = yasna::construct_der(|writer| {
        writer.write_sequence(|writer| {
            writer.next().write_sequence(|writer| {
                // Certificate Version
                writer.next().write_tagged(yasna::Tag::context(0), |writer| {
                    writer.write_i8(2);
                });
                // Certificate Serial Number (unused but required)
                writer.next().write_u8(1);
                // Signature Algorithm: ecdsa-with-SHA256
                writer.next().write_sequence(|writer| {
                    writer.next().write_oid(&ObjectIdentifier::from_slice(&[1,2,840,10045,4,3,2]));
                });
                // Issuer: CN=MesaTEE (unused but required)
                writer.next().write_sequence(|writer| {
                    writer.next().write_set(|writer| {
                        writer.next().write_sequence(|writer| {
                            writer.next().write_oid(&ObjectIdentifier::from_slice(&[2,5,4,3]));
                            writer.next().write_identifier(TAG_UTF8STRING, PC::Primitive);
                            writer.next().write_length(ISSUER.len());
                            writer.buf.extend_from_slice(ISSUER.as_bytes());
                        });
                    });
                });
                // Validity: Issuing/Expiring Time (unused but required)
                let now = SystemTime::now();//.duration_since(UNIX_EPOCH).unwrap().as_secs();
                let chrono_now: DateTime<Utc> = now.into();
                let issue_ts = chrono_now.format("%y%m%d%H%M%SZ").to_string();
                let expire_ts = (chrono_now + Duration::days(1)).format("%y%m%d%H%M%SZ").to_string();
                writer.next().write_sequence(|writer| {
                    writer.next().write_identifier(TAG_UTCTIME, PC::Primitive);
                    writer.next().write_length(13);
                    writer.buf.extend_from_slice(&issue_ts.as_bytes());
                    writer.next().write_identifier(TAG_UTCTIME, PC::Primitive);
                    writer.next().write_length(13);
                    writer.buf.extend_from_slice(&expire_ts.as_bytes());
                });
                // Subject: CN=MesaTEE (unused but required)
                writer.next().write_sequence(|writer| {
                    writer.next().write_set(|writer| {
                        writer.next().write_sequence(|writer| {
                            writer.next().write_oid(&ObjectIdentifier::from_slice(&[2,5,4,3]));
                            writer.next().write_identifier(TAG_UTF8STRING, PC::Primitive);
                            writer.next().write_length(SUBJECT.len());
                            writer.buf.extend_from_slice(SUBJECT.as_bytes());
                        });
                    });
                });
                writer.next().write_sequence(|writer| {
                    // Public Key Algorithm
                    writer.next().write_sequence(|writer| {
                        // id-ecPublicKey
                        writer.next().write_oid(&ObjectIdentifier::from_slice(&[1,2,840,10045,2,1]));
                        // prime256v1
                        writer.next().write_oid(&ObjectIdentifier::from_slice(&[1,2,840,10045,3,1,7]));
                    });
                    // Public Key
                    writer.next().write_bitvec(&BitVec::from_bytes(&pub_key_bytes));
                });
                // Certificate V3 Extension
                writer.next().write_tagged(yasna::Tag::context(3), |writer| {
                    writer.write_sequence(|writer| {
                        writer.next().write_sequence(|writer| {
                            writer.next().write_oid(&ObjectIdentifier::from_slice(&[2,16,840,1,113730,1,13]));
                            writer.next().write_bytes(&payload.into_bytes());
                        });
                    });
                });
            });
            // Signature Algorithm: ecdsa-with-SHA256
            writer.next().write_sequence(|writer| {
                writer.next().write_oid(&ObjectIdentifier::from_slice(&[1,2,840,10045,4,3,2]));
            });
            // Signature
            let sig = {
                let tbs = &writer.buf[4..];
                ecc_handle.ecdsa_sign_slice(tbs, &prv_k).unwrap()
            };
            let sig_der = yasna::construct_der(|writer| {
                writer.write_sequence(|writer| {
                    let mut sig_x = sig.x.clone();
                    sig_x.reverse();
                    let mut sig_y = sig.y.clone();
                    sig_y.reverse();
                    writer.next().write_biguint(&BigUint::from_slice(&sig_x));
                    writer.next().write_biguint(&BigUint::from_slice(&sig_y));
                });
            });
            writer.next().write_bitvec(&BitVec::from_bytes(&sig_der));
        });
    });

    // Generate Private Key DER
    let key_der = yasna::construct_der(|writer| {
        writer.write_sequence(|writer| {
            writer.next().write_u8(0);
            writer.next().write_sequence(|writer| {
                writer.next().write_oid(&ObjectIdentifier::from_slice(&[1,2,840,10045,2,1]));
                writer.next().write_oid(&ObjectIdentifier::from_slice(&[1,2,840,10045,3,1,7]));
            });
            let inner_key_der = yasna::construct_der(|writer| {
                writer.write_sequence(|writer| {
                    writer.next().write_u8(1);
                    let mut prv_k_r = prv_k.r.clone();
                    prv_k_r.reverse();
                    writer.next().write_bytes(&prv_k_r);
                    writer.next().write_tagged(yasna::Tag::context(1), |writer| {
                        writer.write_bitvec(&BitVec::from_bytes(&pub_key_bytes));
                    });
                });
            });
            writer.next().write_bytes(&inner_key_der);
        });
    });

    Ok((key_der, cert_der))
}

pub fn percent_decode(orig: String) -> String {
    let v:Vec<&str> = orig.split("%").collect();
    let mut ret = String::new();
    ret.push_str(v[0]);
    if v.len() > 1 {
        for s in v[1..].iter() {
            ret.push(u8::from_str_radix(&s[0..2], 16).unwrap() as char);
            ret.push_str(&s[2..]);
        }
    }
    ret
}
