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

#![allow(non_snake_case)]

extern crate base64;
extern crate base64_url;
extern crate jsonwebkey;
extern crate jsonwebtoken;
extern crate libloading;
extern crate serde;
extern crate serde_json;
extern crate sgx_types;
extern crate sgx_urts;
extern crate sha2;
extern crate x509_certificate;

use std::convert::{TryFrom, TryInto};

use base64::{engine::general_purpose, Engine as _};
use jsonwebkey::{JsonWebKey, Key, PublicExponent, RsaPublic};
use jsonwebtoken as jwt;
use serde::{Deserialize, Serialize};
use sgx_types::*;
use sgx_urts::SgxEnclave;
use sha2::{Digest, Sha256};
use x509_certificate::{X509Certificate, X509CertificateError};

static ENCLAVE_FILE: &'static str = "enclave.signed.so";
const ATTESTATION_PROVIDER_URL: &'static str = "https://sharedeus.eus.attest.azure.net";
const SGX_ATTESTATION_URI: &'static str = "/attest/SgxEnclave?api-version=2022-08-01";

extern "C" {
    fn enclave_create_report(
        eid: sgx_enclave_id_t,
        retval: *mut i32,
        p_qe3_target: &sgx_target_info_t,
        p_report_data: &sgx_report_data_t,
        p_report: *mut sgx_report_t,
    ) -> sgx_status_t;
}

fn init_enclave() -> SgxResult<SgxEnclave> {
    let mut launch_token: sgx_launch_token_t = [0; 1024];
    let mut launch_token_updated: i32 = 0;
    // call sgx_create_enclave to initialize an enclave instance
    // Debug Support: set 2nd parameter to 1
    let debug = 0;
    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };
    SgxEnclave::create(
        ENCLAVE_FILE,
        debug,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    )
}

// Re-invent App/utility.cpp
// int generate_quote(uint8_t **quote_buffer, uint32_t& quote_size)
fn generate_quote(runtime_data: &[u8]) -> Option<Vec<u8>> {
    let mut ti: sgx_target_info_t = sgx_target_info_t::default();

    println!("Step1: Call sgx_qe_get_target_info:");
    //println!("sgx_qe_get_target_info = {:p}", sgx_qe_get_target_info as * const _);

    let qe3_ret = unsafe { sgx_qe_get_target_info(&mut ti as *mut _) };

    if qe3_ret != sgx_quote3_error_t::SGX_QL_SUCCESS {
        println!("Error in sgx_qe_get_target_info. {:?}\n", qe3_ret);
        return None;
    }

    //println!("target_info.mr_enclave = {:?}", ti.mr_enclave.m);
    //println!("target_info.config_id = {:02x}", ti.config_id.iter().format(" "));

    println!("succeed!\nStep2: Call create_app_report:");
    let app_report: sgx_report_t = if let Some(r) = create_app_enclave_report(&ti, &runtime_data) {
        println!("succeed! \nStep3: Call sgx_qe_get_quote_size:");
        r
    } else {
        println!("\nCall to create_app_report() failed\n");
        return None;
    };

    println!(
        "app_report.body.mr_enclave = {:x?}",
        app_report.body.mr_enclave.m
    );
    println!(
        "app_report.body.mr_signer = {:x?}",
        app_report.body.mr_signer.m
    );
    // println!(
    //     "app_report.body.misc_select = {:08x}",
    //     app_report.body.misc_select
    // );

    let mut quote_size: u32 = 0;
    let qe3_ret = unsafe { sgx_qe_get_quote_size(&mut quote_size as _) };

    if qe3_ret != sgx_quote3_error_t::SGX_QL_SUCCESS {
        println!("Error in sgx_qe_get_quote_size . {:?}\n", qe3_ret);
        return None;
    }

    println!("succeed!");

    let mut quote_vec: Vec<u8> = vec![0; quote_size as usize];

    println!("\nStep4: Call sgx_qe_get_quote:");

    let qe3_ret =
        unsafe { sgx_qe_get_quote(&app_report as _, quote_size, quote_vec.as_mut_ptr() as _) };

    if qe3_ret != sgx_quote3_error_t::SGX_QL_SUCCESS {
        println!("Error in sgx_qe_get_quote. {:?}\n", qe3_ret);
        return None;
    }
    println!("succeed!");
    Some(quote_vec)
}

fn create_app_enclave_report(
    qe_ti: &sgx_target_info_t,
    runtime_data: &[u8],
) -> Option<sgx_report_t> {
    let enclave = if let Ok(r) = init_enclave() {
        r
    } else {
        return None;
    };

    let mut retval = 0;
    let mut ret_report: sgx_report_t = sgx_report_t::default();
    let mut report_data = sgx_report_data_t::default();
    let mut hasher = Sha256::new();
    hasher.update(runtime_data);
    let res = hasher.finalize();
    report_data.d[..32].copy_from_slice(&res);

    let result = unsafe {
        enclave_create_report(
            enclave.geteid(),
            &mut retval,
            qe_ti,
            &report_data,
            &mut ret_report as *mut sgx_report_t,
        )
    };
    match result {
        sgx_status_t::SGX_SUCCESS => {}
        _ => {
            println!("[-] ECALL Enclave Failed {}!", result.as_str());
            return None;
        }
    }
    enclave.destroy();
    Some(ret_report)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AzureSgxAttestationRequest {
    quote: String,
    runtime_data: SgxRuntimeData,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SgxRuntimeData {
    data: String,
    data_type: String,
}

impl SgxRuntimeData {
    fn new_binary(data: &[u8]) -> Self {
        Self {
            data: base64_url::encode(data),
            data_type: "Binary".to_string(),
        }
    }
}

#[derive(Deserialize)]
struct JwtResponse {
    pub token: String,
}
#[derive(Deserialize)]
struct RawJsonWebKey {
    pub x5c: Vec<String>,
    pub kid: String,
    pub kty: String,
    // pub alg: String,
}
#[derive(Deserialize)]
struct RawJsonWebKeySet {
    pub keys: Vec<RawJsonWebKey>,
}

#[derive(Debug)]
struct JsonWebKeySet {
    pub keys: Vec<JsonWebKey>,
}

impl TryFrom<RawJsonWebKeySet> for JsonWebKeySet {
    type Error = X509CertificateError;
    // This method only works for RS256 algorithm using by Azure Attestation
    fn try_from(raw_set: RawJsonWebKeySet) -> Result<Self, X509CertificateError> {
        let mut keys = Vec::new();
        for key in raw_set.keys {
            if key.kty != "RSA" {
                return Err(X509CertificateError::UnknownKeyAlgorithm(key.kty));
            }
            let raw_cert = general_purpose::STANDARD
                .decode(key.x5c[0].clone())
                .unwrap();
            let x509 = X509Certificate::from_der(&raw_cert)?;
            let pubkey = x509.rsa_public_key_data()?;
            let rsa_pub = RsaPublic {
                e: PublicExponent,
                n: pubkey.modulus.as_slice().into(),
            };
            let rsa_key = Key::RSA {
                public: rsa_pub,
                private: None,
            };
            let mut jwk = JsonWebKey::new(rsa_key);
            jwk.key_id = Some(key.kid);
            keys.push(jwk);
        }
        Ok(JsonWebKeySet { keys })
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct TokenClaims {
    pub x_ms_sgx_is_debuggable: bool,
    pub x_ms_sgx_mrenclave: String,
    pub x_ms_sgx_mrsigner: String,
    // pub x_ms_sgx_product_id: u64,
    pub x_ms_sgx_svn: u64,
    pub x_ms_sgx_ehd: String,
}

/// Validate JsonWebToken with JsonWebKeySet,
/// only works for RS256 algorithm and token from default attestation provider.
fn validate_json_web_token(jwt: String, jwks: JsonWebKeySet) -> jwt::errors::Result<TokenClaims> {
    let header = jwt::decode_header(&jwt)?;
    if header.kid.is_none() {
        return Err(jwt::errors::Error::from(
            jwt::errors::ErrorKind::InvalidToken,
        ));
    }
    // find the corresponding key
    let mut idx: Option<usize> = None;
    for (i, key) in jwks.keys.iter().enumerate() {
        if key.key_id.is_some() {
            if key.key_id == header.kid {
                idx = Some(i);
            }
        }
    }
    if idx.is_none() {
        // cannot find corresponding pubkey
        return Err(jwt::errors::Error::from(
            jwt::errors::ErrorKind::InvalidRsaKey,
        ));
    }
    let pem = jwks.keys[idx.unwrap()].key.try_to_pem().unwrap();
    // println!("\n{pem}");
    let key = jwt::DecodingKey::from_rsa_pem(pem.as_bytes())?;
    // prepare validation
    let algo = jwt::Algorithm::RS256;
    let mut validation = jwt::Validation::new(algo);
    validation.validate_exp = false;
    validation.iss = Some(ATTESTATION_PROVIDER_URL.to_string());
    // decode JWT with the public key
    Ok(jwt::decode::<TokenClaims>(&jwt, &key, &validation)?.claims)
}

fn main() {
    let runtime_data = b"This is some runtime data";
    // generate a quote using runtime data
    let quote = generate_quote(runtime_data).unwrap();
    let attest_request = AzureSgxAttestationRequest {
        quote: base64_url::encode(&quote),
        runtime_data: SgxRuntimeData::new_binary(runtime_data),
    };
    // request for azure attestation
    let client = reqwest::blocking::Client::new();
    let res = client
        .post(format!(
            "{}{}",
            ATTESTATION_PROVIDER_URL, SGX_ATTESTATION_URI
        ))
        .json(&attest_request)
        .send()
        .unwrap();
    let jwt = res.json::<JwtResponse>().unwrap().token;
    // println!("{:?}", jwt);
    // get public key from azure attestation
    let res = client
        .get(format!("{}/certs", ATTESTATION_PROVIDER_URL))
        .send()
        .unwrap();
    let resp_text = res.text().unwrap();
    // println!("{resp_text}");
    let raw_key_set: RawJsonWebKeySet = serde_json::from_str(&resp_text).unwrap();
    let jwk_set: JsonWebKeySet = raw_key_set.try_into().unwrap();
    let claims = validate_json_web_token(jwt, jwk_set).unwrap();
    println!(
        "Verified SGX debuggable status: {}",
        claims.x_ms_sgx_is_debuggable
    );
    println!(
        "Verified SGX enclave measurement: {}",
        claims.x_ms_sgx_mrenclave
    );
    println!(
        "Verified SGX signer measurement: {}",
        claims.x_ms_sgx_mrsigner
    );
    println!("Verified SGX SGX SVN: {}", claims.x_ms_sgx_svn);
    println!(
        "Verified SGX runtime data: {}",
        std::str::from_utf8(&base64_url::decode(&claims.x_ms_sgx_ehd).unwrap()).unwrap()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAW_KEY_SET: &str = r#"{
        "keys": [
          {
            "x5c": [
              "MIIVTTCCFDWgAwIBAgIBATANBgkqhkiG9w0BAQsFADAxMS8wLQYDVQQDDCZodHRwczovL3NoYXJlZGV1cy5ldXMuYXR0ZXN0LmF6dXJlLm5ldDAiGA8yMDE5MDUwMTAwMDAwMFoYDzIwNTAxMjMxMjM1OTU5WjAxMS8wLQYDVQQDDCZodHRwczovL3NoYXJlZGV1cy5ldXMuYXR0ZXN0LmF6dXJlLm5ldDCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAKfQDlZ09kKuVXiUBEImso/kOXUU7qP5rvuesKcITCYOkz1W9er/1uLBxjaTTzpK3G588QtLzOtcrjM86r7+TqGEzSvdLLzDnyr5GCo09kMHMCpuFp12ySL4m8ZqZKgPvOorAeJqsfvrPjsSIojW1q85Lrl3/YPgeTVF5o0izYxarqobEQOLqJer0ZWLVQZshk/kPtTeQcp/TlgxhB1hdP3cXXtQ7vTMuLKxWj9uJhnKHodpuTswgLpglyKGWkHXdYocaP4TbZPBoASeaz3LbJWPLt9UVVy4hmpgYs9M9VoXbZHkjwG8qRMP0n4hdUxw1mxjBqONGQlX9kOsGMrV8xMCAwEAAaOCEmowghJmMAkGA1UdEwQCMAAwHQYDVR0OBBYEFAG31/z/zDAVK37GgK8J5vKnIpaoMB8GA1UdIwQYMBaAFAG31/z/zDAVK37GgK8J5vKnIpaoMIISFwYJKwYBBAGCN2kBBIISCAEAAAACAAAA+BEAAAAAAAADAAIAAAAAAAkADgCTmnIz95xMqZQKDbOVfwYHjy9trlTRyRN1/fcs6ZGm7AAAAAAVFQsH/4AOAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFAAAAAAAAAAcAAAAAAAAAjwyKFKAxjuUoa4INfCr8z1bmJP52nDY56s1rQAP5ze8AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHS6X7oghXHPmeH3FY5lNqBbu2zngH7vj2qX9kp69kuDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQABAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAKxZfcTPoO071+t8tIrHmbSJ9tDA+UmAJxm5ShWrGrvCAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABEEAAAlaZgI3dGCFu0AUbe7gALC6D0j36s7HjXx7im1RxOPjMnh3HzBgE3JyINUH6e3oByYQgLPisiL6y2blDtm3Gj6bAyRKdBHENqBbT2YjIjAv0VYlBMzRaBydb8s0JNl+tRwIZAcP8oNeVf0/2F5iW0rksYs0oEZDBMUg6GNjzcMpMVFQsH/4AOAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAVAAAAAAAAAAcAAAAAAAAAlT8yqiyI1Z3XUaY6O5L36S/+iygkXjnw1r5mSoezAV4AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIxPV3XXllA+lhN/d8aKgpoAVqyN7XAUCwgbCUSQxXv/AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAJAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAB+oRMf8K0VIegER3xAB8xm9lyWA7Ibv6TweDjnoefTCAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAC/Y51tS6HGp5QydaKioF49wG+Z4byydgC4tNMjQL7RLoUVYkJLAZkmDI9v2kCxQtphNYbRQ7xzsp3/D0OX9qGvIAAAAQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHwUA3A0AAC0tLS0tQkVHSU4gQ0VSVElGSUNBVEUtLS0tLQpNSUlFalRDQ0JET2dBd0lCQWdJVWI0WjRscTIwQ1hOZnFVeVRYODkwQTZtWkhXZ3dDZ1lJS29aSXpqMEVBd0l3CmNURWpNQ0VHQTFVRUF3d2FTVzUwWld3Z1UwZFlJRkJEU3lCUWNtOWpaWE56YjNJZ1EwRXhHakFZQmdOVkJBb00KRVVsdWRHVnNJRU52Y25CdmNtRjBhVzl1TVJRd0VnWURWUVFIREF0VFlXNTBZU0JEYkdGeVlURUxNQWtHQTFVRQpDQXdDUTBFeEN6QUpCZ05WQkFZVEFsVlRNQjRYRFRJek1Ea3hPVEV6TkRVMU1Gb1hEVE13TURreE9URXpORFUxCk1Gb3djREVpTUNBR0ExVUVBd3daU1c1MFpXd2dVMGRZSUZCRFN5QkRaWEowYVdacFkyRjBaVEVhTUJnR0ExVUUKQ2d3UlNXNTBaV3dnUTI5eWNHOXlZWFJwYjI0eEZEQVNCZ05WQkFjTUMxTmhiblJoSUVOc1lYSmhNUXN3Q1FZRApWUVFJREFKRFFURUxNQWtHQTFVRUJoTUNWVk13V1RBVEJnY3Foa2pPUFFJQkJnZ3Foa2pPUFFNQkJ3TkNBQVRGCkVtTmd0c1VmcWVHR2FvNmQrZVp6ZTFaQXEyWGlyRWVaZkY1VHFhTGhQREFXTWlOVWxZcUZHUmV1aXhjeUV0L0EKbk9ORjFPanBLWHYzUHFtZWRQa01vNElDcURDQ0FxUXdId1lEVlIwakJCZ3dGb0FVME9pcTJuWFgrUzVKRjVnOApleFJsME5YeVdVMHdiQVlEVlIwZkJHVXdZekJob0YrZ1hZWmJhSFIwY0hNNkx5OWhjR2t1ZEhKMWMzUmxaSE5sCmNuWnBZMlZ6TG1sdWRHVnNMbU52YlM5elozZ3ZZMlZ5ZEdsbWFXTmhkR2x2Ymk5Mk15OXdZMnRqY213L1kyRTkKY0hKdlkyVnpjMjl5Sm1WdVkyOWthVzVuUFdSbGNqQWRCZ05WSFE0RUZnUVV1dWJNV0xtZjhzN1llMTE2TjIvTgplaWZtRFU4d0RnWURWUjBQQVFIL0JBUURBZ2JBTUF3R0ExVWRFd0VCL3dRQ01BQXdnZ0hVQmdrcWhraUcrRTBCCkRRRUVnZ0hGTUlJQndUQWVCZ29xaGtpRytFMEJEUUVCQkJESzdoOHlrZmFQNHFhNmtBZVhyOWVCTUlJQlpBWUsKS29aSWh2aE5BUTBCQWpDQ0FWUXdFQVlMS29aSWh2aE5BUTBCQWdFQ0FSVXdFQVlMS29aSWh2aE5BUTBCQWdJQwpBUlV3RUFZTEtvWklodmhOQVEwQkFnTUNBUUl3RUFZTEtvWklodmhOQVEwQkFnUUNBUVF3RUFZTEtvWklodmhOCkFRMEJBZ1VDQVFFd0VRWUxLb1pJaHZoTkFRMEJBZ1lDQWdDQU1CQUdDeXFHU0liNFRRRU5BUUlIQWdFT01CQUcKQ3lxR1NJYjRUUUVOQVFJSUFnRUFNQkFHQ3lxR1NJYjRUUUVOQVFJSkFnRUFNQkFHQ3lxR1NJYjRUUUVOQVFJSwpBZ0VBTUJBR0N5cUdTSWI0VFFFTkFRSUxBZ0VBTUJBR0N5cUdTSWI0VFFFTkFRSU1BZ0VBTUJBR0N5cUdTSWI0ClRRRU5BUUlOQWdFQU1CQUdDeXFHU0liNFRRRU5BUUlPQWdFQU1CQUdDeXFHU0liNFRRRU5BUUlQQWdFQU1CQUcKQ3lxR1NJYjRUUUVOQVFJUUFnRUFNQkFHQ3lxR1NJYjRUUUVOQVFJUkFnRU5NQjhHQ3lxR1NJYjRUUUVOQVFJUwpCQkFWRlFJRUFZQU9BQUFBQUFBQUFBQUFNQkFHQ2lxR1NJYjRUUUVOQVFNRUFnQUFNQlFHQ2lxR1NJYjRUUUVOCkFRUUVCZ0NRYnRVQUFEQVBCZ29xaGtpRytFMEJEUUVGQ2dFQU1Bb0dDQ3FHU000OUJBTUNBMGdBTUVVQ0lBb00KMFMvOU5IbmpjQnB4bE43d0ZSSDB4N0VGWnNOKzI4aDlQdTkvRkJsWEFpRUE1Sm1XOFhERVBmY3pDUnJqR0VDMQpxbFA5akVyQnJFYWUyZno1TDdqWXl5ND0KLS0tLS1FTkQgQ0VSVElGSUNBVEUtLS0tLQotLS0tLUJFR0lOIENFUlRJRklDQVRFLS0tLS0KTUlJQ21EQ0NBajZnQXdJQkFnSVZBTkRvcXRwMTEva3VTUmVZUEhzVVpkRFY4bGxOTUFvR0NDcUdTTTQ5QkFNQwpNR2d4R2pBWUJnTlZCQU1NRVVsdWRHVnNJRk5IV0NCU2IyOTBJRU5CTVJvd0dBWURWUVFLREJGSmJuUmxiQ0JECmIzSndiM0poZEdsdmJqRVVNQklHQTFVRUJ3d0xVMkZ1ZEdFZ1EyeGhjbUV4Q3pBSkJnTlZCQWdNQWtOQk1Rc3cKQ1FZRFZRUUdFd0pWVXpBZUZ3MHhPREExTWpFeE1EVXdNVEJhRncwek16QTFNakV4TURVd01UQmFNSEV4SXpBaApCZ05WQkFNTUdrbHVkR1ZzSUZOSFdDQlFRMHNnVUhKdlkyVnpjMjl5SUVOQk1Sb3dHQVlEVlFRS0RCRkpiblJsCmJDQkRiM0p3YjNKaGRHbHZiakVVTUJJR0ExVUVCd3dMVTJGdWRHRWdRMnhoY21FeEN6QUpCZ05WQkFnTUFrTkIKTVFzd0NRWURWUVFHRXdKVlV6QlpNQk1HQnlxR1NNNDlBZ0VHQ0NxR1NNNDlBd0VIQTBJQUJMOXErTk1wMklPZwp0ZGwxYmsvdVdaNStUR1FtOGFDaTh6NzhmcytmS0NRM2QrdUR6WG5WVEFUMlpoRENpZnlJdUp3dk4zd05CcDlpCkhCU1NNSk1KckJPamdic3dnYmd3SHdZRFZSMGpCQmd3Rm9BVUltVU0xbHFkTkluemc3U1ZVcjlRR3prbkJxd3cKVWdZRFZSMGZCRXN3U1RCSG9FV2dRNFpCYUhSMGNITTZMeTlqWlhKMGFXWnBZMkYwWlhNdWRISjFjM1JsWkhObApjblpwWTJWekxtbHVkR1ZzTG1OdmJTOUpiblJsYkZOSFdGSnZiM1JEUVM1a1pYSXdIUVlEVlIwT0JCWUVGTkRvCnF0cDExL2t1U1JlWVBIc1VaZERWOGxsTk1BNEdBMVVkRHdFQi93UUVBd0lCQmpBU0JnTlZIUk1CQWY4RUNEQUcKQVFIL0FnRUFNQW9HQ0NxR1NNNDlCQU1DQTBnQU1FVUNJUUNKZ1RidFZxT3laMW0zanFpQVhNNlFZYTZyNXNXUwo0eS9HN3k4dUlKR3hkd0lnUnFQdkJTS3p6UWFnQkxRcTVzNUE3MHBkb2lhUko4ei8wdUR6NE5nVjkxaz0KLS0tLS1FTkQgQ0VSVElGSUNBVEUtLS0tLQotLS0tLUJFR0lOIENFUlRJRklDQVRFLS0tLS0KTUlJQ2p6Q0NBalNnQXdJQkFnSVVJbVVNMWxxZE5JbnpnN1NWVXI5UUd6a25CcXd3Q2dZSUtvWkl6ajBFQXdJdwphREVhTUJnR0ExVUVBd3dSU1c1MFpXd2dVMGRZSUZKdmIzUWdRMEV4R2pBWUJnTlZCQW9NRVVsdWRHVnNJRU52CmNuQnZjbUYwYVc5dU1SUXdFZ1lEVlFRSERBdFRZVzUwWVNCRGJHRnlZVEVMTUFrR0ExVUVDQXdDUTBFeEN6QUoKQmdOVkJBWVRBbFZUTUI0WERURTRNRFV5TVRFd05EVXhNRm9YRFRRNU1USXpNVEl6TlRrMU9Wb3dhREVhTUJnRwpBMVVFQXd3UlNXNTBaV3dnVTBkWUlGSnZiM1FnUTBFeEdqQVlCZ05WQkFvTUVVbHVkR1ZzSUVOdmNuQnZjbUYwCmFXOXVNUlF3RWdZRFZRUUhEQXRUWVc1MFlTQkRiR0Z5WVRFTE1Ba0dBMVVFQ0F3Q1EwRXhDekFKQmdOVkJBWVQKQWxWVE1Ga3dFd1lIS29aSXpqMENBUVlJS29aSXpqMERBUWNEUWdBRUM2bkV3TURJWVpPai9pUFdzQ3phRUtpNwoxT2lPU0xSRmhXR2pibkJWSmZWbmtZNHUzSWprRFlZTDBNeE80bXFzeVlqbEJhbFRWWXhGUDJzSkJLNXpsS09CCnV6Q0J1REFmQmdOVkhTTUVHREFXZ0JRaVpReldXcDAwaWZPRHRKVlN2MUFiT1NjR3JEQlNCZ05WSFI4RVN6QkoKTUVlZ1JhQkRoa0ZvZEhSd2N6b3ZMMk5sY25ScFptbGpZWFJsY3k1MGNuVnpkR1ZrYzJWeWRtbGpaWE11YVc1MApaV3d1WTI5dEwwbHVkR1ZzVTBkWVVtOXZkRU5CTG1SbGNqQWRCZ05WSFE0RUZnUVVJbVVNMWxxZE5JbnpnN1NWClVyOVFHemtuQnF3d0RnWURWUjBQQVFIL0JBUURBZ0VHTUJJR0ExVWRFd0VCL3dRSU1BWUJBZjhDQVFFd0NnWUkKS29aSXpqMEVBd0lEU1FBd1JnSWhBT1cvNVFrUitTOUNpU0RjTm9vd0x1UFJMc1dHZi9ZaTdHU1g5NEJnd1R3ZwpBaUVBNEowbHJIb01zK1hvNW8vc1g2TzlRV3hIUkF2WlVHT2RSUTdjdnFSWGFxST0KLS0tLS1FTkQgQ0VSVElGSUNBVEUtLS0tLQoAMA0GCSqGSIb3DQEBCwUAA4IBAQCPIgeaN4IQA+MH6x4t7uqHLJ52vHVJ0m/DuLJ1DnekUrpRI44WbofC1hKmGPBcqV55sEzoyUH1WWAge3Lg0EBqogKddVmS04rE1He9QMVptiL9Bg1ahfZUjEdKYHriDzPo3KYs43nkXMRMFGWAdAuAdRoVVh6+g66M+iJ16KXxAQT5v6I+OAjwoivzEv6+6MpBt57/tm71iu2CyR5MEsOkEW6deHsKIEnZz0v4fydQfajVu49myXFsd6NFNbyk3Voira/OYuY0T8+eyfZMs5zmGY/waEEgr7U8igAxllV3/FCquZ/b86IRQ4VH7phYQ1oVbLvAem2huFV5LJuqzsfh"
            ],
            "kid": "rFl9xM+g7TvX63y0iseZtIn20MD5SYAnGblKFasau8I=",
            "kty": "RSA"
          },
          {
            "x5c": [
              "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",
              "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",
              "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"
            ],
            "kid": "dSsaF5uUxZO_LRycmQ4KJu3ctMc",
            "kty": "RSA"
          },
          {
            "x5c": [
              "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R0ExVUVBd3daU1c1MFpXd2dVMGRZSUZCRFN5QkRaWEowYVdacFkyRjBaVEVhTUJnR0ExVUUKQ2d3UlNXNTBaV3dnUTI5eWNHOXlZWFJwYjI0eEZEQVNCZ05WQkFjTUMxTmhiblJoSUVOc1lYSmhNUXN3Q1FZRApWUVFJREFKRFFURUxNQWtHQTFVRUJoTUNWVk13V1RBVEJnY3Foa2pPUFFJQkJnZ3Foa2pPUFFNQkJ3TkNBQVRGCkVtTmd0c1VmcWVHR2FvNmQrZVp6ZTFaQXEyWGlyRWVaZkY1VHFhTGhQREFXTWlOVWxZcUZHUmV1aXhjeUV0L0EKbk9ORjFPanBLWHYzUHFtZWRQa01vNElDcURDQ0FxUXdId1lEVlIwakJCZ3dGb0FVME9pcTJuWFgrUzVKRjVnOApleFJsME5YeVdVMHdiQVlEVlIwZkJHVXdZekJob0YrZ1hZWmJhSFIwY0hNNkx5OWhjR2t1ZEhKMWMzUmxaSE5sCmNuWnBZMlZ6TG1sdWRHVnNMbU52YlM5elozZ3ZZMlZ5ZEdsbWFXTmhkR2x2Ymk5Mk15OXdZMnRqY213L1kyRTkKY0hKdlkyVnpjMjl5Sm1WdVkyOWthVzVuUFdSbGNqQWRCZ05WSFE0RUZnUVV1dWJNV0xtZjhzN1llMTE2TjIvTgplaWZtRFU4d0RnWURWUjBQQVFIL0JBUURBZ2JBTUF3R0ExVWRFd0VCL3dRQ01BQXdnZ0hVQmdrcWhraUcrRTBCCkRRRUVnZ0hGTUlJQndUQWVCZ29xaGtpRytFMEJEUUVCQkJESzdoOHlrZmFQNHFhNmtBZVhyOWVCTUlJQlpBWUsKS29aSWh2aE5BUTBCQWpDQ0FWUXdFQVlMS29aSWh2aE5BUTBCQWdFQ0FSVXdFQVlMS29aSWh2aE5BUTBCQWdJQwpBUlV3RUFZTEtvWklodmhOQVEwQkFnTUNBUUl3RUFZTEtvWklodmhOQVEwQkFnUUNBUVF3RUFZTEtvWklodmhOCkFRMEJBZ1VDQVFFd0VRWUxLb1pJaHZoTkFRMEJBZ1lDQWdDQU1CQUdDeXFHU0liNFRRRU5BUUlIQWdFT01CQUcKQ3lxR1NJYjRUUUVOQVFJSUFnRUFNQkFHQ3lxR1NJYjRUUUVOQVFJSkFnRUFNQkFHQ3lxR1NJYjRUUUVOQVFJSwpBZ0VBTUJBR0N5cUdTSWI0VFFFTkFRSUxBZ0VBTUJBR0N5cUdTSWI0VFFFTkFRSU1BZ0VBTUJBR0N5cUdTSWI0ClRRRU5BUUlOQWdFQU1CQUdDeXFHU0liNFRRRU5BUUlPQWdFQU1CQUdDeXFHU0liNFRRRU5BUUlQQWdFQU1CQUcKQ3lxR1NJYjRUUUVOQVFJUUFnRUFNQkFHQ3lxR1NJYjRUUUVOQVFJUkFnRU5NQjhHQ3lxR1NJYjRUUUVOQVFJUwpCQkFWRlFJRUFZQU9BQUFBQUFBQUFBQUFNQkFHQ2lxR1NJYjRUUUVOQVFNRUFnQUFNQlFHQ2lxR1NJYjRUUUVOCkFRUUVCZ0NRYnRVQUFEQVBCZ29xaGtpRytFMEJEUUVGQ2dFQU1Bb0dDQ3FHU000OUJBTUNBMGdBTUVVQ0lBb00KMFMvOU5IbmpjQnB4bE43d0ZSSDB4N0VGWnNOKzI4aDlQdTkvRkJsWEFpRUE1Sm1XOFhERVBmY3pDUnJqR0VDMQpxbFA5akVyQnJFYWUyZno1TDdqWXl5ND0KLS0tLS1FTkQgQ0VSVElGSUNBVEUtLS0tLQotLS0tLUJFR0lOIENFUlRJRklDQVRFLS0tLS0KTUlJQ21EQ0NBajZnQXdJQkFnSVZBTkRvcXRwMTEva3VTUmVZUEhzVVpkRFY4bGxOTUFvR0NDcUdTTTQ5QkFNQwpNR2d4R2pBWUJnTlZCQU1NRVVsdWRHVnNJRk5IV0NCU2IyOTBJRU5CTVJvd0dBWURWUVFLREJGSmJuUmxiQ0JECmIzSndiM0poZEdsdmJqRVVNQklHQTFVRUJ3d0xVMkZ1ZEdFZ1EyeGhjbUV4Q3pBSkJnTlZCQWdNQWtOQk1Rc3cKQ1FZRFZRUUdFd0pWVXpBZUZ3MHhPREExTWpFeE1EVXdNVEJhRncwek16QTFNakV4TURVd01UQmFNSEV4SXpBaApCZ05WQkFNTUdrbHVkR1ZzSUZOSFdDQlFRMHNnVUhKdlkyVnpjMjl5SUVOQk1Sb3dHQVlEVlFRS0RCRkpiblJsCmJDQkRiM0p3YjNKaGRHbHZiakVVTUJJR0ExVUVCd3dMVTJGdWRHRWdRMnhoY21FeEN6QUpCZ05WQkFnTUFrTkIKTVFzd0NRWURWUVFHRXdKVlV6QlpNQk1HQnlxR1NNNDlBZ0VHQ0NxR1NNNDlBd0VIQTBJQUJMOXErTk1wMklPZwp0ZGwxYmsvdVdaNStUR1FtOGFDaTh6NzhmcytmS0NRM2QrdUR6WG5WVEFUMlpoRENpZnlJdUp3dk4zd05CcDlpCkhCU1NNSk1KckJPamdic3dnYmd3SHdZRFZSMGpCQmd3Rm9BVUltVU0xbHFkTkluemc3U1ZVcjlRR3prbkJxd3cKVWdZRFZSMGZCRXN3U1RCSG9FV2dRNFpCYUhSMGNITTZMeTlqWlhKMGFXWnBZMkYwWlhNdWRISjFjM1JsWkhObApjblpwWTJWekxtbHVkR1ZzTG1OdmJTOUpiblJsYkZOSFdGSnZiM1JEUVM1a1pYSXdIUVlEVlIwT0JCWUVGTkRvCnF0cDExL2t1U1JlWVBIc1VaZERWOGxsTk1BNEdBMVVkRHdFQi93UUVBd0lCQmpBU0JnTlZIUk1CQWY4RUNEQUcKQVFIL0FnRUFNQW9HQ0NxR1NNNDlCQU1DQTBnQU1FVUNJUUNKZ1RidFZxT3laMW0zanFpQVhNNlFZYTZyNXNXUwo0eS9HN3k4dUlKR3hkd0lnUnFQdkJTS3p6UWFnQkxRcTVzNUE3MHBkb2lhUko4ei8wdUR6NE5nVjkxaz0KLS0tLS1FTkQgQ0VSVElGSUNBVEUtLS0tLQotLS0tLUJFR0lOIENFUlRJRklDQVRFLS0tLS0KTUlJQ2p6Q0NBalNnQXdJQkFnSVVJbVVNMWxxZE5JbnpnN1NWVXI5UUd6a25CcXd3Q2dZSUtvWkl6ajBFQXdJdwphREVhTUJnR0ExVUVBd3dSU1c1MFpXd2dVMGRZSUZKdmIzUWdRMEV4R2pBWUJnTlZCQW9NRVVsdWRHVnNJRU52CmNuQnZjbUYwYVc5dU1SUXdFZ1lEVlFRSERBdFRZVzUwWVNCRGJHRnlZVEVMTUFrR0ExVUVDQXdDUTBFeEN6QUoKQmdOVkJBWVRBbFZUTUI0WERURTRNRFV5TVRFd05EVXhNRm9YRFRRNU1USXpNVEl6TlRrMU9Wb3dhREVhTUJnRwpBMVVFQXd3UlNXNTBaV3dnVTBkWUlGSnZiM1FnUTBFeEdqQVlCZ05WQkFvTUVVbHVkR1ZzSUVOdmNuQnZjbUYwCmFXOXVNUlF3RWdZRFZRUUhEQXRUWVc1MFlTQkRiR0Z5WVRFTE1Ba0dBMVVFQ0F3Q1EwRXhDekFKQmdOVkJBWVQKQWxWVE1Ga3dFd1lIS29aSXpqMENBUVlJS29aSXpqMERBUWNEUWdBRUM2bkV3TURJWVpPai9pUFdzQ3phRUtpNwoxT2lPU0xSRmhXR2pibkJWSmZWbmtZNHUzSWprRFlZTDBNeE80bXFzeVlqbEJhbFRWWXhGUDJzSkJLNXpsS09CCnV6Q0J1REFmQmdOVkhTTUVHREFXZ0JRaVpReldXcDAwaWZPRHRKVlN2MUFiT1NjR3JEQlNCZ05WSFI4RVN6QkoKTUVlZ1JhQkRoa0ZvZEhSd2N6b3ZMMk5sY25ScFptbGpZWFJsY3k1MGNuVnpkR1ZrYzJWeWRtbGpaWE11YVc1MApaV3d1WTI5dEwwbHVkR1ZzVTBkWVVtOXZkRU5CTG1SbGNqQWRCZ05WSFE0RUZnUVVJbVVNMWxxZE5JbnpnN1NWClVyOVFHemtuQnF3d0RnWURWUjBQQVFIL0JBUURBZ0VHTUJJR0ExVWRFd0VCL3dRSU1BWUJBZjhDQVFFd0NnWUkKS29aSXpqMEVBd0lEU1FBd1JnSWhBT1cvNVFrUitTOUNpU0RjTm9vd0x1UFJMc1dHZi9ZaTdHU1g5NEJnd1R3ZwpBaUVBNEowbHJIb01zK1hvNW8vc1g2TzlRV3hIUkF2WlVHT2RSUTdjdnFSWGFxST0KLS0tLS1FTkQgQ0VSVElGSUNBVEUtLS0tLQoAMA0GCSqGSIb3DQEBCwUAA4GBAJXP4REbBXOUekVsrULSnN1cemM4ZhNNDZngCUrFjRmQ7A1gOt8+QwEH7as/764CWNgTvaporuQzYxr8zTFuQeRfLoyvRuUG1Y46Z+lfJ88H5L3AeAuK6emeaTzYE3klChaNnQOorXFJI5CDgBxfeSH9QNKH/C86aBAlSk+axiLK"
            ],
            "kid": "lH/VQ9TL53cXpeFt3Vf9tiOv+rkWwV+5PbiNzgWcySQ=",
            "kty": "RSA"
          }
        ]
      }"#;

    // The sample JWT from the Azure attestation server.
    // Format: ${header}.${body}.${signature}
    const RAW_TOKEN: &str = "eyJhbGciOiJSUzI1NiIsImprdSI6Imh0dHBzOi8vc2hhcmVkZXVzLmV1cy5hdHRlc3QuYXp1cmUubmV0L2NlcnRzIiwia2lkIjoickZsOXhNK2c3VHZYNjN5MGlzZVp0SW4yME1ENVNZQW5HYmxLRmFzYXU4ST0iLCJ0eXAiOiJKV1QifQ\
    .eyJhYXMtZWhkIjoiVkdocGN5QnBjeUJ6YjIxbElISjFiblJwYldVZ1pHRjBZUSIsImV4cCI6MTY5NTc2NjU0NywiaWF0IjoxNjk1NzM3NzQ3LCJpcy1kZWJ1Z2dhYmxlIjpmYWxzZSwiaXNzIjoiaHR0cHM6Ly9zaGFyZWRldXMuZXVzLmF0dGVzdC5henVyZS5uZXQiLCJqdGkiOiJmM2Q3NDU2ZjIwOGVhNzc5MTkxY2U0ZGVkMDY2YWI3ZmUyY2I3NTVhY2Y1MDYzOThiMzIzOGVmMjY3ZjgzZDlmIiwibWFhLWF0dGVzdGF0aW9uY29sbGF0ZXJhbCI6eyJxZWlkY2VydHNoYXNoIjoiYTY0ZDY0OTE5ODUwN2Q4YjU3ZTMzZjYzYWIyNjY4MzhmNDNmMzI3YmQ0YWFjYzc4NTEwYjY5NzZlZDA0NmUxMCIsInFlaWRjcmxoYXNoIjoiMTMxMTNlZWQ1NTEyZTBmMTcwYjVhY2RkNjkwM2VkNTcxYmU0MGFjOGJkMTVlNzhhYzYwZmI3YWZiOTE2YjFiYiIsInFlaWRoYXNoIjoiNzcwMWY2NDcwMGI3ZjUwNWQ3YjRiN2E5M2U0NWQ1Y2RlOGNmYzg2NWI2MGYxZGQ0OWVjYmVlOTc5MGMzMzcyZSIsInF1b3RlaGFzaCI6Ijg5ZWUxMWE4ODNhMDgwYWFiNmUyNjI2MmMxMDUwMzk4YjY3NWVkYzI0YWMzNGUyMzcwNDg1MWM0NjUzNzBmMTAiLCJ0Y2JpbmZvY2VydHNoYXNoIjoiYTY0ZDY0OTE5ODUwN2Q4YjU3ZTMzZjYzYWIyNjY4MzhmNDNmMzI3YmQ0YWFjYzc4NTEwYjY5NzZlZDA0NmUxMCIsInRjYmluZm9jcmxoYXNoIjoiMTMxMTNlZWQ1NTEyZTBmMTcwYjVhY2RkNjkwM2VkNTcxYmU0MGFjOGJkMTVlNzhhYzYwZmI3YWZiOTE2YjFiYiIsInRjYmluZm9oYXNoIjoiODJkMTA5ZmIzMDhmMjRhOTBlNDM5MzZlYTllMTJiNTViMDUyNTAyMjFmZGEyMjk0Zjc0YWI1ODE3ZTcxYmVhNCJ9LCJtYWEtZWhkIjoiVkdocGN5QnBjeUJ6YjIxbElISjFiblJwYldVZ1pHRjBZUSIsIm5iZiI6MTY5NTczNzc0NywicHJvZHVjdC1pZCI6MSwic2d4LW1yZW5jbGF2ZSI6ImY1NjczNWFhNDI1NjM2MjdhODMyZTBjN2JhOTkxMTM4MmViNjhhZmVkNzU4MzBiM2Y5NzI2NmYzZTY3YmRjOTkiLCJzZ3gtbXJzaWduZXIiOiJhNTk1YzZjNTgwNWRhMGM5YzRjYjkyMDMzNGQzNTRhZWFlZTIyMDdlNDc5ZGZmNjc5ZDVmMzYwMzc1ZjU1N2RkIiwic3ZuIjoxLCJ0ZWUiOiJzZ3giLCJ4LW1zLWF0dGVzdGF0aW9uLXR5cGUiOiJzZ3giLCJ4LW1zLXBvbGljeSI6eyJpcy1kZWJ1Z2dhYmxlIjpmYWxzZSwicHJvZHVjdC1pZCI6MSwic2d4LW1yZW5jbGF2ZSI6ImY1NjczNWFhNDI1NjM2MjdhODMyZTBjN2JhOTkxMTM4MmViNjhhZmVkNzU4MzBiM2Y5NzI2NmYzZTY3YmRjOTkiLCJzZ3gtbXJzaWduZXIiOiJhNTk1YzZjNTgwNWRhMGM5YzRjYjkyMDMzNGQzNTRhZWFlZTIyMDdlNDc5ZGZmNjc5ZDVmMzYwMzc1ZjU1N2RkIiwic3ZuIjoxLCJ0ZWUiOiJzZ3gifSwieC1tcy1wb2xpY3ktaGFzaCI6Ik93RXZwU1ZFV0E1ZWlzQ0VuY0J0OE5TWkZMWURSS29MYW9PTlByWmdvZVkiLCJ4LW1zLXNneC1jb2xsYXRlcmFsIjp7InFlaWRjZXJ0c2hhc2giOiJhNjRkNjQ5MTk4NTA3ZDhiNTdlMzNmNjNhYjI2NjgzOGY0M2YzMjdiZDRhYWNjNzg1MTBiNjk3NmVkMDQ2ZTEwIiwicWVpZGNybGhhc2giOiIxMzExM2VlZDU1MTJlMGYxNzBiNWFjZGQ2OTAzZWQ1NzFiZTQwYWM4YmQxNWU3OGFjNjBmYjdhZmI5MTZiMWJiIiwicWVpZGhhc2giOiI3NzAxZjY0NzAwYjdmNTA1ZDdiNGI3YTkzZTQ1ZDVjZGU4Y2ZjODY1YjYwZjFkZDQ5ZWNiZWU5NzkwYzMzNzJlIiwicXVvdGVoYXNoIjoiODllZTExYTg4M2EwODBhYWI2ZTI2MjYyYzEwNTAzOThiNjc1ZWRjMjRhYzM0ZTIzNzA0ODUxYzQ2NTM3MGYxMCIsInRjYmluZm9jZXJ0c2hhc2giOiJhNjRkNjQ5MTk4NTA3ZDhiNTdlMzNmNjNhYjI2NjgzOGY0M2YzMjdiZDRhYWNjNzg1MTBiNjk3NmVkMDQ2ZTEwIiwidGNiaW5mb2NybGhhc2giOiIxMzExM2VlZDU1MTJlMGYxNzBiNWFjZGQ2OTAzZWQ1NzFiZTQwYWM4YmQxNWU3OGFjNjBmYjdhZmI5MTZiMWJiIiwidGNiaW5mb2hhc2giOiI4MmQxMDlmYjMwOGYyNGE5MGU0MzkzNmVhOWUxMmI1NWIwNTI1MDIyMWZkYTIyOTRmNzRhYjU4MTdlNzFiZWE0In0sIngtbXMtc2d4LWVoZCI6IlZHaHBjeUJwY3lCemIyMWxJSEoxYm5ScGJXVWdaR0YwWVEiLCJ4LW1zLXNneC1pcy1kZWJ1Z2dhYmxlIjpmYWxzZSwieC1tcy1zZ3gtbXJlbmNsYXZlIjoiZjU2NzM1YWE0MjU2MzYyN2E4MzJlMGM3YmE5OTExMzgyZWI2OGFmZWQ3NTgzMGIzZjk3MjY2ZjNlNjdiZGM5OSIsIngtbXMtc2d4LW1yc2lnbmVyIjoiYTU5NWM2YzU4MDVkYTBjOWM0Y2I5MjAzMzRkMzU0YWVhZWUyMjA3ZTQ3OWRmZjY3OWQ1ZjM2MDM3NWY1NTdkZCIsIngtbXMtc2d4LXByb2R1Y3QtaWQiOjEsIngtbXMtc2d4LXJlcG9ydC1kYXRhIjoiOTRiYTQ0ZjM5OWI5YzRhZGM4MzBkNzhjNjdmNDkxNGNiYmMzYTM4MzhmNzk2ZDJlNzY2NjU5NDc1NGMwNjdkOTAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAiLCJ4LW1zLXNneC1zdm4iOjEsIngtbXMtc2d4LXRjYmlkZW50aWZpZXIiOiIxMCIsIngtbXMtdmVyIjoiMS4wIn0\
    .QJyc2Ka98fiy6r_FDbfzjgV3TCTFmODe-32FiGSiAyCz_ZO5Bmw9XnQI2Rzs-Yrq6b4bDV4WlMRmJePRXzI1i2cR3xtWhnJKQjTz_EYp63OfH8SsiWci_BQpTnzoiAbUi5EdrbYz3CXQtThTy_XHyYmJVEY8qLZ0dzSO4QmBxz6q8BfcEp7fhwuKzibetQlJ3zdz-TwIK0l0WbZ1jBG93oXPnQy9KhDAyDX533DvYjDjAE3FPnjV5cMZfjmcLVxTL6DROEIlZtm_yn5zJSWlQBrFRDxoYxoYtQlEeaOn-klKZj4ECJF498mACo5fYW20UhXv5ZZNdEMYVNb4dEVf-w";

    #[test]
    fn raw_key_conversion() {
        let raw_set: RawJsonWebKeySet = serde_json::from_str(&RAW_KEY_SET).unwrap();
        let jwk_set: JsonWebKeySet = raw_set.try_into().unwrap();
        assert_eq!(jwk_set.keys.len(), 3);
    }

    #[test]
    fn token_validation() {
        let raw_set: RawJsonWebKeySet = serde_json::from_str(&RAW_KEY_SET).unwrap();
        let jwks: JsonWebKeySet = raw_set.try_into().unwrap();
        let claims = validate_json_web_token(RAW_TOKEN.to_string(), jwks).unwrap();
        assert!(!claims.x_ms_sgx_is_debuggable);
        assert_eq!(
            claims.x_ms_sgx_mrenclave,
            "f56735aa42563627a832e0c7ba9911382eb68afed75830b3f97266f3e67bdc99"
        );
        assert_eq!(
            claims.x_ms_sgx_mrsigner,
            "a595c6c5805da0c9c4cb920334d354aeaee2207e479dff679d5f360375f557dd"
        );
        assert_eq!(
            base64_url::decode(&claims.x_ms_sgx_ehd).unwrap(),
            b"This is some runtime data",
        )
    }
}
