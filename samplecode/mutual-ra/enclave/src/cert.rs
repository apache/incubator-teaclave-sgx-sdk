use std::prelude::v1::*;
use std::{str, ptr};
use std::time::*;
use std::untrusted::time::SystemTimeEx;
//use std::untrusted::fs::File;
//use std::io::prelude::*;

use sgx_tcrypto::*;
use sgx_types::*;

use super::CERTEXPIRYDAYS;
use std::io::BufReader;
use rustls;
use yasna;
use base64;
use webpki;
use serde_json;
use serde_json::Value;
use num_bigint::BigUint;
use bit_vec::BitVec;
use yasna::models::ObjectIdentifier;
use chrono::prelude::*;
use chrono::Duration;
use chrono::TimeZone;
use chrono::Utc as TzUtc;
use itertools::Itertools;

extern "C" {
    pub fn ocall_get_update_info (ret_val: *mut sgx_status_t,
                                  platformBlob: * const sgx_platform_info_t,
                                  enclaveTrusted: i32,
                                  update_info: * mut sgx_update_info_bit_t) -> sgx_status_t;
}


type SignatureAlgorithms = &'static [&'static webpki::SignatureAlgorithm];
static SUPPORTED_SIG_ALGS: SignatureAlgorithms = &[
    &webpki::ECDSA_P256_SHA256,
    &webpki::ECDSA_P256_SHA384,
    &webpki::ECDSA_P384_SHA256,
    &webpki::ECDSA_P384_SHA384,
    &webpki::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
    &webpki::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
    &webpki::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
    &webpki::RSA_PKCS1_2048_8192_SHA256,
    &webpki::RSA_PKCS1_2048_8192_SHA384,
    &webpki::RSA_PKCS1_2048_8192_SHA512,
    &webpki::RSA_PKCS1_3072_8192_SHA384,
];

pub const IAS_REPORT_CA:&[u8] = include_bytes!("../AttestationReportSigningCACert.pem");

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
                            writer
                                .next()
                                .write_oid(&ObjectIdentifier::from_slice(&[2,5,4,3]));
                            writer.next().write_utf8_string(&ISSUER);
                        });
                    });
                });
                // Validity: Issuing/Expiring Time (unused but required)
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let issue_ts = TzUtc.timestamp(now.as_secs() as i64, 0);
                let expire = now + Duration::days(CERTEXPIRYDAYS).to_std().unwrap();
                let expire_ts = TzUtc.timestamp(expire.as_secs() as i64, 0);
                writer.next().write_sequence(|writer| {
                    writer.next().write_utctime(&yasna::models::UTCTime::from_datetime(&issue_ts));
                    writer.next().write_utctime(&yasna::models::UTCTime::from_datetime(&expire_ts));
                });
                // Subject: CN=MesaTEE (unused but required)
                writer.next().write_sequence(|writer| {
                    writer.next().write_set(|writer| {
                        writer.next().write_sequence(|writer| {
                            writer.next().write_oid(&ObjectIdentifier::from_slice(&[2,5,4,3]));
                            writer.next().write_utf8_string(&SUBJECT);
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

pub fn verify_mra_cert(cert_der: &[u8]) -> Result<(), sgx_status_t> {
    // Before we reach here, Webpki already verifed the cert is properly signed

    // Search for Public Key prime256v1 OID
    let prime256v1_oid = &[0x06, 0x08, 0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x03, 0x01, 0x07];
    let mut offset = cert_der.windows(prime256v1_oid.len()).position(|window| window == prime256v1_oid).unwrap();
    offset += 11; // 10 + TAG (0x03)

    // Obtain Public Key length
    let mut len = cert_der[offset] as usize;
    if len > 0x80 {
        len = (cert_der[offset+1] as usize) * 0x100 + (cert_der[offset+2] as usize);
        offset += 2;
    }

    // Obtain Public Key
    offset += 1;
    let pub_k = cert_der[offset+2..offset+len].to_vec(); // skip "00 04"


    // Search for Netscape Comment OID
    let ns_cmt_oid = &[0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x86, 0xF8, 0x42, 0x01, 0x0D];
    let mut offset = cert_der.windows(ns_cmt_oid.len()).position(|window| window == ns_cmt_oid).unwrap();
    offset += 12; // 11 + TAG (0x04)

    // Obtain Netscape Comment length
    let mut len = cert_der[offset] as usize;
    if len > 0x80 {
        len = (cert_der[offset+1] as usize) * 0x100 + (cert_der[offset+2] as usize);
        offset += 2;
    }

    // Obtain Netscape Comment
    offset += 1;
    let payload = cert_der[offset..offset+len].to_vec();

    // Extract each field
    let mut iter = payload.split(|x| *x == 0x7C);
    let attn_report_raw = iter.next().unwrap();
    let sig_raw = iter.next().unwrap();
    let sig = base64::decode(&sig_raw).unwrap();

    let sig_cert_raw = iter.next().unwrap();
    let sig_cert_dec = base64::decode_config(&sig_cert_raw, base64::STANDARD).unwrap();
    //let sig_cert_input = untrusted::Input::from(&sig_cert_dec);
    let sig_cert = webpki::EndEntityCert::from(&sig_cert_dec).expect("Bad DER");

    // Verify if the signing cert is issued by Intel CA
    let mut ias_ca_stripped = IAS_REPORT_CA.to_vec();
    ias_ca_stripped.retain(|&x| x != 0x0d && x != 0x0a);
    let head_len = "-----BEGIN CERTIFICATE-----".len();
    let tail_len = "-----END CERTIFICATE-----".len();
    let full_len = ias_ca_stripped.len();
    let ias_ca_core : &[u8] = &ias_ca_stripped[head_len..full_len - tail_len];
    let ias_cert_dec = base64::decode_config(ias_ca_core, base64::STANDARD).unwrap();

    let mut ca_reader = BufReader::new(&IAS_REPORT_CA[..]);

    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_pem_file(&mut ca_reader).expect("Failed to add CA");

    let trust_anchors: Vec<webpki::TrustAnchor> = root_store
        .roots
        .iter()
        .map(|cert| cert.to_trust_anchor())
        .collect();

    let mut chain:Vec<&[u8]> = Vec::new();
    chain.push(&ias_cert_dec);

    let now_func = webpki::Time::try_from(SystemTime::now());

    match sig_cert.verify_is_valid_tls_server_cert(
        SUPPORTED_SIG_ALGS,
        &webpki::TLSServerTrustAnchors(&trust_anchors),
        &chain,
        now_func.unwrap()) {
        Ok(_) => println!("Cert is good"),
        Err(e) => println!("Cert verification error {:?}", e),
    }

    // Verify the signature against the signing cert
    match sig_cert.verify_signature(
        &webpki::RSA_PKCS1_2048_8192_SHA256,
        &attn_report_raw,
        &sig) {
        Ok(_) => println!("Signature good"),
        Err(e) => {
            println!("Signature verification error {:?}", e);
            panic!();
        },
    }

    // Verify attestation report
    // 1. Check timestamp is within 24H (90day is recommended by Intel)
    let attn_report: Value = serde_json::from_slice(attn_report_raw).unwrap();
    if let Value::String(time) = &attn_report["timestamp"] {
        let time_fixed = time.clone() + "+0000";
        let ts = DateTime::parse_from_str(&time_fixed, "%Y-%m-%dT%H:%M:%S%.f%z").unwrap().timestamp();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        println!("Time diff = {}", now - ts);
    } else {
        println!("Failed to fetch timestamp from attestation report");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    }

    // 2. Verify quote status (mandatory field)
    if let Value::String(quote_status) = &attn_report["isvEnclaveQuoteStatus"] {
        println!("isvEnclaveQuoteStatus = {}", quote_status);
        match quote_status.as_ref() {
            "OK" => (),
            "GROUP_OUT_OF_DATE" | "GROUP_REVOKED" | "CONFIGURATION_NEEDED" => {
                // Verify platformInfoBlob for further info if status not OK
                if let Value::String(pib) = &attn_report["platformInfoBlob"] {
                    let mut buf = Vec::new();

                    // the TLV Header (4 bytes/8 hexes) should be skipped
                    let n = (pib.len() - 8)/2;
                    for i in 0..n {
                        buf.push(u8::from_str_radix(&pib[(i*2+8)..(i*2+10)], 16).unwrap());
                    }

                    let mut update_info = sgx_update_info_bit_t::default();
                    let mut rt : sgx_status_t = sgx_status_t::SGX_ERROR_UNEXPECTED;
                    let res = unsafe{
                        ocall_get_update_info(&mut rt as *mut sgx_status_t,
                                              buf.as_slice().as_ptr() as * const sgx_platform_info_t,
                                              1,
                                              &mut update_info as * mut sgx_update_info_bit_t)
                    };
                    if res != sgx_status_t::SGX_SUCCESS {
                        println!("res={:?}", res);
                        return Err(res);
                    }

                    if rt != sgx_status_t::SGX_SUCCESS {
                        println!("rt={:?}", rt);
                        // Borrow of packed field is unsafe in future Rust releases
                        unsafe{
                            println!("update_info.pswUpdate: {}", update_info.pswUpdate);
                            println!("update_info.csmeFwUpdate: {}", update_info.csmeFwUpdate);
                            println!("update_info.ucodeUpdate: {}", update_info.ucodeUpdate);
                        }
                        return Err(rt);
                    }
                } else {
                    println!("Failed to fetch platformInfoBlob from attestation report");
                    return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
                }
            }
            _ => return Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    } else {
        println!("Failed to fetch isvEnclaveQuoteStatus from attestation report");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    }

    // 3. Verify quote body
    if let Value::String(quote_raw) = &attn_report["isvEnclaveQuoteBody"] {
        let quote = base64::decode(&quote_raw).unwrap();
        println!("Quote = {:?}", quote);
        // TODO: lack security check here
        let sgx_quote: sgx_quote_t = unsafe{ptr::read(quote.as_ptr() as *const _)};

        // Borrow of packed field is unsafe in future Rust releases
        // ATTENTION
        // DO SECURITY CHECK ON DEMAND
        // DO SECURITY CHECK ON DEMAND
        // DO SECURITY CHECK ON DEMAND
        unsafe{
            println!("sgx quote version = {}", sgx_quote.version);
            println!("sgx quote signature type = {}", sgx_quote.sign_type);
            println!("sgx quote report_data = {:02x}", sgx_quote.report_body.report_data.d.iter().format(""));
            println!("sgx quote mr_enclave = {:02x}", sgx_quote.report_body.mr_enclave.m.iter().format(""));
            println!("sgx quote mr_signer = {:02x}", sgx_quote.report_body.mr_signer.m.iter().format(""));
        }
        println!("Anticipated public key = {:02x}", pub_k.iter().format(""));
        if sgx_quote.report_body.report_data.d.to_vec() == pub_k.to_vec() {
            println!("Mutual RA done!");
        }
    } else {
        println!("Failed to fetch isvEnclaveQuoteBody from attestation report");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    }

    Ok(())
}
