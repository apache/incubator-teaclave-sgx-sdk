use std::prelude::v1::*;
use std::ptr;
use std::time::*;
use std::io::BufReader;

use sgx_types::*;

use rustls;
use base64;
use webpki;
use serde_json;
use serde_json::Value;
use chrono::prelude::*;
use itertools::Itertools;

use pib::*;

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


pub const IAS_REPORT_CA : &[u8] = include_bytes!("../../cert/AttestationReportSigningCACert.pem");

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
    let sig_cert_dec = base64::decode_config(&sig_cert_raw, base64::MIME).unwrap();
    let sig_cert = webpki::EndEntityCert::from(&sig_cert_dec).expect("Bad DER");

    // Load Intel CA

    let mut ias_ca_stripped = IAS_REPORT_CA.to_vec();
    ias_ca_stripped.retain(|&x| x != 0x0d && x != 0x0a);
    let head_len = "-----BEGIN CERTIFICATE-----".len();
    let tail_len = "-----END CERTIFICATE-----".len();
    let full_len = ias_ca_stripped.len();
    let ias_ca_core : &[u8] = &ias_ca_stripped[head_len..full_len - tail_len];
    let ias_cert_dec = base64::decode_config(ias_ca_core, base64::MIME).unwrap();

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
    // 1. Check timestamp is within 24H
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
                    let got_pib = platform_info::from_str(&pib);
                    println!("{:?}", got_pib);
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
            println!("ue RA done!");
        }
    } else {
        println!("Failed to fetch isvEnclaveQuoteBody from attestation report");
        return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
    }

    Ok(())
}
