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
// under the License.

extern crate sgx_types;
extern crate sgx_trts;
extern crate sgx_libc;

use sgx_types::error::SgxStatus;
use sgx_types::types::*;
use sgx_tse::EnclaveReport;
use std::backtrace::{self, PrintFormat};
use std::io;
use std::fs;
use std::str;
use std::io::{Read, Write};
use std::sync::Arc;
use std::net::{TcpStream, SocketAddr};
use sgx_crypto::sha::Sha256;
use sgx_crypto::ecc::*;
use itertools::Itertools;

mod cert;
mod hex;

pub const DEV_HOSTNAME:&'static str = "api.trustedservices.intel.com";
pub const SIGRL_SUFFIX:&'static str = "/sgx/dev/attestation/v3/sigrl/";
pub const REPORT_SUFFIX:&'static str = "/sgx/dev/attestation/v3/report";
pub const CERTEXPIRYDAYS: i64 = 90i64;

extern "C" {
    pub fn ocall_sgx_init_quote ( ret_val : *mut SgxStatus,
                  ret_ti  : *mut TargetInfo,
                  ret_gid : *mut EpidGroupId) -> SgxStatus;
    pub fn ocall_get_ias_socket ( ret_val : *mut SgxStatus,
                  ret_fd  : *mut i32) -> SgxStatus;
    pub fn ocall_get_quote (ret_val            : *mut SgxStatus,
                p_sigrl            : *const u8,
                sigrl_len          : u32,
                p_report           : *const Report,
                quote_type         : QuoteSignType,
                p_spid             : *const Spid,
                p_nonce            : *const QuoteNonce,
                p_qe_report        : *mut Report,
                p_quote            : *mut u8,
                maxlen             : u32,
                p_quote_len        : *mut u32) -> SgxStatus;
}


fn parse_response_attn_report(resp : &[u8]) -> (String, String, String){
    println!("parse_response_attn_report");
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut respp   = httparse::Response::new(&mut headers);
    let result = respp.parse(resp);
    println!("parse result {:?}", result);

    let msg : &'static str;

    match respp.code {
        Some(200) => msg = "OK Operation Successful",
        Some(401) => msg = "Unauthorized Failed to authenticate or authorize request.",
        Some(404) => msg = "Not Found GID does not refer to a valid EPID group ID.",
        Some(500) => msg = "Internal error occurred",
        Some(503) => msg = "Service is currently not able to process the request (due to
            a temporary overloading or maintenance). This is a
            temporary state – the same request can be repeated after
            some time. ",
        _ => {println!("DBG:{}", respp.code.unwrap()); msg = "Unknown error occured"},
    }

    println!("{}", msg);
    let mut len_num : u32 = 0;

    let mut sig = String::new();
    let mut cert = String::new();
    let mut attn_report = String::new();

    for i in 0..respp.headers.len() {
        let h = respp.headers[i];
        //println!("{} : {}", h.name, str::from_utf8(h.value).unwrap());
        match h.name{
            "Content-Length" => {
                let len_str = String::from_utf8(h.value.to_vec()).unwrap();
                len_num = len_str.parse::<u32>().unwrap();
                println!("content length = {}", len_num);
            }
            "X-IASReport-Signature" => sig = str::from_utf8(h.value).unwrap().to_string(),
            "X-IASReport-Signing-Certificate" => cert = str::from_utf8(h.value).unwrap().to_string(),
            _ => (),
        }
    }

    // Remove %0A from cert, and only obtain the signing cert
    cert = cert.replace("%0A", "");
    cert = cert::percent_decode(cert);
    let v: Vec<&str> = cert.split("-----").collect();
    let sig_cert = v[2].to_string();

    if len_num != 0 {
        let header_len = result.unwrap().unwrap();
        let resp_body = &resp[header_len..];
        attn_report = str::from_utf8(resp_body).unwrap().to_string();
        println!("Attestation report: {}", attn_report);
    }

    // len_num == 0
    (attn_report, sig, sig_cert)
}


fn parse_response_sigrl(resp : &[u8]) -> Vec<u8> {
    println!("parse_response_sigrl");
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut respp   = httparse::Response::new(&mut headers);
    let result = respp.parse(resp);
    println!("parse result {:?}", result);
    println!("parse response{:?}", respp);

    let msg : &'static str;

    match respp.code {
        Some(200) => msg = "OK Operation Successful",
        Some(401) => msg = "Unauthorized Failed to authenticate or authorize request.",
        Some(404) => msg = "Not Found GID does not refer to a valid EPID group ID.",
        Some(500) => msg = "Internal error occurred",
        Some(503) => msg = "Service is currently not able to process the request (due to
            a temporary overloading or maintenance). This is a
            temporary state – the same request can be repeated after
            some time. ",
        _ => msg = "Unknown error occured",
    }

    println!("{}", msg);
    let mut len_num : u32 = 0;

    for i in 0..respp.headers.len() {
        let h = respp.headers[i];
        if h.name == "content-length" {
            let len_str = String::from_utf8(h.value.to_vec()).unwrap();
            len_num = len_str.parse::<u32>().unwrap();
            println!("content length = {}", len_num);
        }
    }

    if len_num != 0 {
        let header_len = result.unwrap().unwrap();
        let resp_body = &resp[header_len..];
        println!("Base64-encoded SigRL: {:?}", resp_body);

        return base64::decode(str::from_utf8(resp_body).unwrap()).unwrap();
    }

    // len_num == 0
    Vec::new()
}

pub fn make_ias_client_config() -> rustls::ClientConfig {
    let mut config = rustls::ClientConfig::new();

    config.root_store.add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

    config
}


pub fn get_sigrl_from_intel(fd : c_int, gid : u32) -> Vec<u8> {
    println!("get_sigrl_from_intel fd = {:?}", fd);
    let config = make_ias_client_config();
    //let sigrl_arg = SigRLArg { group_id : gid };
    //let sigrl_req = sigrl_arg.to_httpreq();
    let ias_key = get_ias_api_key();

    let req = format!("GET {}{:08x} HTTP/1.1\r\nHOST: {}\r\nOcp-Apim-Subscription-Key: {}\r\nConnection: Close\r\n\r\n",
                        SIGRL_SUFFIX,
                        gid,
                        DEV_HOSTNAME,
                        ias_key);
    println!("{}", req);

    let dns_name = webpki::DNSNameRef::try_from_ascii_str(DEV_HOSTNAME).unwrap();
    let mut sess = rustls::ClientSession::new(&Arc::new(config), dns_name);

    let port = 443;
    let hostname = "api.trustedservices.intel.com";
    let addr = lookup_ipv4(hostname, port);
    let mut sock = TcpStream::connect(&addr).expect("[-] Connect tls server failed!");

    let mut tls = rustls::Stream::new(&mut sess, &mut sock);

    let _result = tls.write(req.as_bytes());
    let mut plaintext = Vec::new();

    println!("write complete");

    match tls.read_to_end(&mut plaintext) {
        Ok(_) => (),
        Err(e) => {
            println!("get_sigrl_from_intel tls.read_to_end: {:?}", e);
            panic!("haha");
        }
    }
    println!("read_to_end complete");
    let resp_string = String::from_utf8(plaintext.clone()).unwrap();

    println!("{}", resp_string);

    parse_response_sigrl(&plaintext)
}

// TODO: support pse
pub fn get_report_from_intel(fd : c_int, quote : Vec<u8>) -> (String, String, String) {
    println!("get_report_from_intel fd = {:?}", fd);
    let config = make_ias_client_config();
    let encoded_quote = base64::encode(&quote[..]);
    let encoded_json = format!("{{\"isvEnclaveQuote\":\"{}\"}}\r\n", encoded_quote);

    let ias_key = get_ias_api_key();

    let req = format!("POST {} HTTP/1.1\r\nHOST: {}\r\nOcp-Apim-Subscription-Key:{}\r\nContent-Length:{}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{}",
                           REPORT_SUFFIX,
                           DEV_HOSTNAME,
                           ias_key,
                           encoded_json.len(),
                           encoded_json);
    println!("{}", req);
    let dns_name = webpki::DNSNameRef::try_from_ascii_str(DEV_HOSTNAME).unwrap();
    let mut sess = rustls::ClientSession::new(&Arc::new(config), dns_name);


    let port = 443;
    let hostname = "api.trustedservices.intel.com";
    let addr = lookup_ipv4(hostname, port);
    let mut sock = TcpStream::connect(&addr).expect("[-] Connect tls server failed!");

    let mut tls = rustls::Stream::new(&mut sess, &mut sock);

    let _result = tls.write(req.as_bytes());
    let mut plaintext = Vec::new();

    println!("write complete");

    tls.read_to_end(&mut plaintext).unwrap();
    println!("read_to_end complete");
    let resp_string = String::from_utf8(plaintext.clone()).unwrap();

    println!("resp_string = {}", resp_string);

    let (attn_report, sig, cert) = parse_response_attn_report(&plaintext);

    (attn_report, sig, cert)
}

fn as_u32_le(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) <<  0) +
    ((array[1] as u32) <<  8) +
    ((array[2] as u32) << 16) +
    ((array[3] as u32) << 24)
}

#[allow(const_err)]
pub fn create_attestation_report(pub_k: &EcPublicKey, sign_type: QuoteSignType) -> Result<(String, String, String), SgxStatus> {
    // Workflow:
    // (1) ocall to get the target_info structure (ti) and epid group id (eg)
    // (1.5) get sigrl
    // (2) call sgx_create_report with ti+data, produce an Report
    // (3) ocall to sgx_get_quote to generate (*mut sgx-quote_t, uint32_t)

    // (1) get ti + eg
    let mut ti : TargetInfo = TargetInfo::default();
    let mut eg : EpidGroupId = EpidGroupId::default();
    let mut rt : SgxStatus = SgxStatus::Unexpected;

    let res = unsafe {
        ocall_sgx_init_quote(&mut rt as *mut SgxStatus,
                             &mut ti as *mut TargetInfo,
                             &mut eg as *mut EpidGroupId)
    };

    println!("eg = {:?}", eg);

    if res != SgxStatus::Success {
        return Err(res);
    }

    if rt != SgxStatus::Success {
        return Err(rt);
    }

    let eg_num = as_u32_le(&eg);

    // (1.5) get sigrl
    let mut ias_sock : i32 = 0;

    let res = unsafe {
        ocall_get_ias_socket(&mut rt as *mut SgxStatus,
                             &mut ias_sock as *mut i32)
    };

    if res != SgxStatus::Success {
        return Err(res);
    }

    if rt != SgxStatus::Success {
        return Err(rt);
    }

    //println!("Got ias_sock = {}", ias_sock);

    // Now sigrl_vec is the revocation list, a vec<u8>
    let sigrl_vec : Vec<u8> = get_sigrl_from_intel(ias_sock, eg_num);

    // (2) Generate the report
    // Fill ecc256 public key into report_data
    let mut report_data: ReportData = ReportData::default();
    let mut pub_k_gx = pub_k.public_key().gx.clone();
    pub_k_gx.reverse();
    let mut pub_k_gy = pub_k.public_key().gy.clone();
    pub_k_gy.reverse();
    report_data.d[..32].clone_from_slice(&pub_k_gx);
    report_data.d[32..].clone_from_slice(&pub_k_gy);

    //let rep = match rsgx_create_report(&ti, &report_data) {
    //    Ok(r) =>{
    //        println!("Report creation => success {:?}", r.body.mr_signer.m);
    //        Some(r)
    //    },
    //    Err(e) =>{
    //        println!("Report creation => failed {:?}", e);
    //        None
    //    },
    //};
    let rep = Report::for_target(&ti, &report_data);

    let mut quote_nonce = QuoteNonce { rand : [0;16] };
    let mut os_rng = sgx_trts::rand::Rng::new();
    os_rng.fill_bytes(&mut quote_nonce.rand);
    println!("rand finished");
    let mut qe_report = Report::default();
    const RET_QUOTE_BUF_LEN : u32 = 2048;
    let mut return_quote_buf : [u8; RET_QUOTE_BUF_LEN as usize] = [0;RET_QUOTE_BUF_LEN as usize];
    let mut quote_len : u32 = 0;

    // (3) Generate the quote
    // Args:
    //       1. sigrl: ptr + len
    //       2. report: ptr 432bytes
    //       3. linkable: u32, unlinkable=0, linkable=1
    //       4. spid: Spid ptr 16bytes
    //       5. QuoteNonce ptr 16bytes
    //       6. p_sig_rl + sigrl size ( same to sigrl)
    //       7. [out]p_qe_report need further check
    //       8. [out]p_quote
    //       9. quote_size
    let (p_sigrl, sigrl_len) =
        if sigrl_vec.len() == 0 {
            (std::ptr::null(), 0)
        } else {
            (sigrl_vec.as_ptr(), sigrl_vec.len() as u32)
        };
    let p_report = (&rep.unwrap()) as * const Report;
    let quote_type = sign_type;

    let spid : Spid = load_spid("spid.txt");

    let p_spid = &spid as *const Spid;
    let p_nonce = &quote_nonce as * const QuoteNonce;
    let p_qe_report = &mut qe_report as *mut Report;
    let p_quote = return_quote_buf.as_mut_ptr();
    let maxlen = RET_QUOTE_BUF_LEN;
    let p_quote_len = &mut quote_len as *mut u32;

    let result = unsafe {
        ocall_get_quote(&mut rt as *mut SgxStatus,
                p_sigrl,
                sigrl_len,
                p_report,
                quote_type,
                p_spid,
                p_nonce,
                p_qe_report,
                p_quote,
                maxlen,
                p_quote_len)
    };

    if result != SgxStatus::Success {
        return Err(result);
    }

    if rt != SgxStatus::Success {
        println!("ocall_get_quote returned {}", rt);
        return Err(rt);
    }

    // Added 09-28-2018
    // Perform a check on qe_report to verify if the qe_report is valid
    match qe_report.verify() {
        Ok(()) => println!("rsgx_verify_report passed!"),
        Err(x) => {
            println!("rsgx_verify_report failed with {:?}", x);
            return Err(x);
        },
    }

    // Check if the qe_report is produced on the same platform
    if ti.mr_enclave.m != qe_report.body.mr_enclave.m ||
       ti.attributes.flags != qe_report.body.attributes.flags ||
       ti.attributes.xfrm  != qe_report.body.attributes.xfrm {
        println!("qe_report does not match current target_info!");
        return Err(SgxStatus::Unexpected);
    }

    println!("qe_report check passed");

    // Debug
    // for i in 0..quote_len {
    //     print!("{:02X}", unsafe {*p_quote.offset(i as isize)});
    // }
    // println!("");

    // Check qe_report to defend against replay attack
    // The purpose of p_qe_report is for the ISV enclave to confirm the QUOTE
    // it received is not modified by the untrusted SW stack, and not a replay.
    // The implementation in QE is to generate a REPORT targeting the ISV
    // enclave (target info from p_report) , with the lower 32Bytes in
    // report.data = SHA256(p_nonce||p_quote). The ISV enclave can verify the
    // p_qe_report and report.data to confirm the QUOTE has not be modified and
    // is not a replay. It is optional.

    let mut rhs_vec : Vec<u8> = quote_nonce.rand.to_vec();
    rhs_vec.extend(&return_quote_buf[..quote_len as usize]);
    //let rhs_hash = rsgx_sha256_slice(&rhs_vec[..]).unwrap();
    let mut shactx = Sha256::new().unwrap();
    shactx.update(&rhs_vec[..]).unwrap();
    let rhs_hash = shactx.finalize().unwrap();
    let lhs_hash = &qe_report.body.report_data.d[..32];

    println!("rhs hash = {:02X}", rhs_hash.iter().format(""));
    println!("report hs= {:02X}", lhs_hash.iter().format(""));

    if rhs_hash.hash != lhs_hash {
        println!("Quote is tampered!");
        return Err(SgxStatus::Unexpected);
    }

    let quote_vec : Vec<u8> = return_quote_buf[..quote_len as usize].to_vec();
    let res = unsafe {
        ocall_get_ias_socket(&mut rt as *mut SgxStatus,
                             &mut ias_sock as *mut i32)
    };

    if res != SgxStatus::Success {
        return Err(res);
    }

    if rt != SgxStatus::Success {
        return Err(rt);
    }

    let (attn_report, sig, cert) = get_report_from_intel(ias_sock, quote_vec);
    Ok((attn_report, sig, cert))
}

fn load_spid(filename: &str) -> Spid {
    let mut spidfile = fs::File::open(filename).expect("cannot open spid file");
    let mut contents = String::new();
    spidfile.read_to_string(&mut contents).expect("cannot read the spid file");

    hex::decode_spid(&contents)
}

fn get_ias_api_key() -> String {
    let mut keyfile = fs::File::open("key.txt").expect("cannot open ias key file");
    let mut key = String::new();
    keyfile.read_to_string(&mut key).expect("cannot read the ias key file");

    key.trim_end().to_owned()
}

struct ClientAuth {
    outdated_ok: bool,
}

impl ClientAuth {
    fn new(outdated_ok: bool) -> ClientAuth {
        ClientAuth{ outdated_ok : outdated_ok }
    }
}

impl rustls::ClientCertVerifier for ClientAuth {
    fn client_auth_root_subjects(&self, _sni: Option<&webpki::DNSName>) -> Option<rustls::DistinguishedNames> {
        Some(rustls::DistinguishedNames::new())
    }

    fn verify_client_cert(&self, _certs: &[rustls::Certificate], _sni: Option<&webpki::DNSName>)
    -> Result<rustls::ClientCertVerified, rustls::TLSError> {
        println!("client cert: {:?}", _certs);
            // This call will automatically verify cert is properly signed
            match cert::verify_mra_cert(&_certs[0].0) {
                Ok(()) => {
                    return Ok(rustls::ClientCertVerified::assertion());
                }
                Err(SgxStatus::UpdateNeeded) => {
                    if self.outdated_ok {
                        println!("outdated_ok is set, overriding outdated error");
                        return Ok(rustls::ClientCertVerified::assertion());
                    } else {
                        return Err(rustls::TLSError::WebPKIError(webpki::Error::ExtensionValueInvalid));
                    }
                }
                Err(_) => {
                    return Err(rustls::TLSError::WebPKIError(webpki::Error::ExtensionValueInvalid));
                }
            }
    }
}

struct ServerAuth {
    outdated_ok: bool
}

impl ServerAuth {
    fn new(outdated_ok: bool) -> ServerAuth {
        ServerAuth{ outdated_ok : outdated_ok }
    }
}

impl rustls::ServerCertVerifier for ServerAuth {
    fn verify_server_cert(&self,
              _roots: &rustls::RootCertStore,
              _certs: &[rustls::Certificate],
              _hostname: webpki::DNSNameRef,
              _ocsp: &[u8]) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
    println!("server cert: {:?}", _certs);
        // This call will automatically verify cert is properly signed
        match cert::verify_mra_cert(&_certs[0].0) {
            Ok(()) => {
                return Ok(rustls::ServerCertVerified::assertion());
            }
            Err(SgxStatus::UpdateNeeded) => {
                if self.outdated_ok {
                    println!("outdated_ok is set, overriding outdated error");
                    return Ok(rustls::ServerCertVerified::assertion());
                } else {
                    return Err(rustls::TLSError::WebPKIError(webpki::Error::ExtensionValueInvalid));
                }
            }
            Err(_) => {
                return Err(rustls::TLSError::WebPKIError(webpki::Error::ExtensionValueInvalid));
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn run_server(socket_fd : c_int, sign_type: QuoteSignType) -> SgxStatus {
    let _ = backtrace::enable_backtrace(PrintFormat::Short);

    // Generate Keypair
    let ecc_handle = EcKeyPair::create().unwrap();
    //let (prv_k, pub_k) = ecc_handle.create_key_pair().unwrap();
    let prv_k = ecc_handle.private_key();
    let pub_k = ecc_handle.public_key();

    let (attn_report, sig, cert) = match create_attestation_report(&pub_k, sign_type) {
        Ok(r) => r,
        Err(e) => {
            println!("Error in create_attestation_report: {:?}", e);
            return e;
        }
    };

    let payload = attn_report + "|" + &sig + "|" + &cert;
    let (key_der, cert_der) = match cert::gen_ecc_cert(payload, &prv_k, &pub_k, &ecc_handle) {
        Ok(r) => r,
        Err(e) => {
            println!("Error in gen_ecc_cert: {:?}", e);
            return e;
        }
    };
    //let _result = ecc_handle.close();


    let mut cfg = rustls::ServerConfig::new(Arc::new(ClientAuth::new(true)));
    let mut certs = Vec::new();
    certs.push(rustls::Certificate(cert_der));
    let privkey = rustls::PrivateKey(key_der);

    cfg.set_single_cert_with_ocsp_and_sct(certs, privkey, vec![], vec![]).unwrap();

    let mut sess = rustls::ServerSession::new(&Arc::new(cfg));

    let port = 443;
    let hostname = "api.trustedservices.intel.com";
    let addr = lookup_ipv4(hostname, port);
    let mut conn = TcpStream::connect(&addr).expect("[-] Connect tls server failed!");

    let mut tls = rustls::Stream::new(&mut sess, &mut conn);
    let mut plaintext = [0u8;1024]; //Vec::new();
    match tls.read(&mut plaintext) {
        Ok(_) => println!("Client said: {}", str::from_utf8(&plaintext).unwrap()),
        Err(e) => {
            println!("Error in read_to_end: {:?}", e);
            panic!("");
        }
    };

    tls.write("hello back".as_bytes()).unwrap();

    SgxStatus::Success
}


#[no_mangle]
pub extern "C" fn run_client(socket_fd : c_int, sign_type: QuoteSignType) -> SgxStatus {
    let _ = backtrace::enable_backtrace(PrintFormat::Short);

    // Generate Keypair
    let ecc_handle = EcKeyPair::create().unwrap();
    //ecc_handle.open().unwrap();
    let prv_k = ecc_handle.private_key();
    let pub_k = ecc_handle.public_key();

    let (attn_report, sig, cert) = match create_attestation_report(&pub_k, sign_type) {
        Ok(r) => r,
        Err(e) => {
            println!("Error in create_attestation_report: {:?}", e);
            return e;
        }
    };

    let payload = attn_report + "|" + &sig + "|" + &cert;

    let (key_der, cert_der) = match cert::gen_ecc_cert(payload, &prv_k, &pub_k, &ecc_handle) {
        Ok(r) => r,
        Err(e) => {
            println!("Error in gen_ecc_cert: {:?}", e);
            return e;
        }
    };
    //ecc_handle.close().unwrap();


    let mut cfg = rustls::ClientConfig::new();
    let mut certs = Vec::new();
    certs.push(rustls::Certificate(cert_der));
    let privkey = rustls::PrivateKey(key_der);

    cfg.set_single_client_cert(certs, privkey).unwrap();
    cfg.dangerous().set_certificate_verifier(Arc::new(ServerAuth::new(true)));
    cfg.versions.clear();
    cfg.versions.push(rustls::ProtocolVersion::TLSv1_2);

    let dns_name = webpki::DNSNameRef::try_from_ascii_str("localhost").unwrap();
    let mut sess = rustls::ClientSession::new(&Arc::new(cfg), dns_name);
    let port = 443;
    let hostname = "api.trustedservices.intel.com";
    let addr = lookup_ipv4(hostname, port);
    let mut conn = TcpStream::connect(&addr).expect("[-] Connect tls server failed!");

    let mut tls = rustls::Stream::new(&mut sess, &mut conn);

    tls.write("hello".as_bytes()).unwrap();

    let mut plaintext = Vec::new();
    match tls.read_to_end(&mut plaintext) {
        Ok(_) => {
            println!("Server replied: {}", str::from_utf8(&plaintext).unwrap());
        }
        Err(ref err) if err.kind() == io::ErrorKind::ConnectionAborted => {
            println!("EOF (tls)");
        }
        Err(e) => println!("Error in read_to_end: {:?}", e),
    }

    SgxStatus::Success
}

pub fn lookup_ipv4(host: &str, port: u16) -> SocketAddr {
    use std::net::ToSocketAddrs;

    let addrs = (host, port).to_socket_addrs().unwrap();
    for addr in addrs {
        if let SocketAddr::V4(_) = addr {
            return addr;
        }
    }

    unreachable!("Cannot lookup address");
}
