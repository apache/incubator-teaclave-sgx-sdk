// Copyright 2015 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
// ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
// OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

//! webpki: Web PKI X.509 Certificate Validation.
//!
//! <code>git clone https://github.com/briansmith/webpki</code>
//!
//! See `EndEntityCert`'s documentation for a description of the certificate
//! processing steps necessary for a TLS connection.

#![doc(html_root_url="https://briansmith.org/rustdoc/")]

#![no_std]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#![allow(
    missing_debug_implementations,
)]

// `#[derive(...)]` uses `#[allow(unused_qualifications)]` internally.
#![deny(
    unused_qualifications,
)]

#![forbid(
    anonymous_parameters,
    box_pointers,
    fat_ptr_transmutes,
    missing_copy_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
//  unstable_features,
    unused_extern_crates,
    unused_import_braces,
    unused_results,
    variant_size_differences,
    warnings,
)]

#[cfg(all(any(test, feature = "trust_anchor_util"), not(target_env = "sgx")))]
#[macro_use(format)]
extern crate sgx_tstd as std;

#[cfg(all(any(test, feature = "trust_anchor_util"), target_env = "sgx"))]
#[macro_use(format)]
extern crate std;

extern crate ring;

#[cfg(test)]
extern crate base64;

extern crate untrusted;

#[macro_use]
mod der;

mod cert;
mod name;
mod signed_data;
mod calendar;
mod time;

#[cfg(feature = "trust_anchor_util")]
pub mod trust_anchor_util;

mod verify_cert;

pub use name::DNSNameRef;

#[cfg(feature = "std")]
pub use name::DNSName;


pub use signed_data::{
    SignatureAlgorithm,
    ECDSA_P256_SHA256,
    ECDSA_P256_SHA384,
    ECDSA_P384_SHA256,
    ECDSA_P384_SHA384,
    RSA_PKCS1_2048_8192_SHA1,
    RSA_PKCS1_2048_8192_SHA256,
    RSA_PKCS1_2048_8192_SHA384,
    RSA_PKCS1_2048_8192_SHA512,
    RSA_PKCS1_3072_8192_SHA384,
    RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
    RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
    RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
};

pub use time::Time;

/// An end-entity certificate.
///
/// Server certificate processing in a TLS connection consists of several
/// steps. All of these steps are necessary:
///
/// * `EndEntityCert.verify_is_valid_tls_server_cert`: Verify that the server's
///   certificate is currently valid *for use by a TLS server*.
/// * `EndEntityCert.verify_is_valid_for_dns_name`: Verify that the server's
///   certificate is valid for the host that is being connected to.
/// * `EndEntityCert.verify_signature`: Verify that the signature of server's
///   `ServerKeyExchange` message is valid for the server's certificate.
///
/// Client certificate processing in a TLS connection consists of analogous
/// steps. All of these steps are necessary:
///
/// * `EndEntityCert.verify_is_valid_tls_client_cert`: Verify that the client's
///   certificate is currently valid *for use by a TLS client*.
/// * `EndEntityCert.verify_is_valid_for_dns_name` or
///   `EndEntityCert.verify_is_valid_for_at_least_one_dns_name`: Verify that the
///   client's certificate is valid for the identity or identities used to
///   identify the client. (Currently client authentication only works when the
///   client is identified by one or more DNS hostnames.)
/// * `EndEntityCert.verify_signature`: Verify that the client's signature in
///   its `CertificateVerify` message is valid using the public key from the
///   client's certificate.
///
/// Although it would be less error-prone to combine all these steps into a
/// single function call, some significant optimizations are possible if the
/// three steps are processed separately (in parallel). It does not matter much
/// which order the steps are done in, but **all of these steps must completed
/// before application data is sent and before received application data is
/// processed**. `EndEntityCert::from` is an inexpensive operation and is
/// deterministic, so if these tasks are done in multiple threads, it is
/// probably best to just call `EndEntityCert::from` multiple times (before each
/// operation) for the same DER-encoded ASN.1 certificate bytes.
pub struct EndEntityCert<'a> {
    inner: cert::Cert<'a>,
}

impl <'a> EndEntityCert<'a> {
    /// Parse the ASN.1 DER-encoded X.509 encoding of the certificate
    /// `cert_der`.
    pub fn from(cert_der: untrusted::Input<'a>)
                -> Result<EndEntityCert<'a>, Error> {
        Ok(EndEntityCert {
            inner: cert::parse_cert(cert_der, cert::EndEntityOrCA::EndEntity)?
        })
    }

    /// Verifies that the end-entity certificate is valid for use by a TLS
    /// server.
    ///
    /// `supported_sig_algs` is the list of signature algorithms that are
    /// trusted for use in certificate signatures; the end-entity certificate's
    /// public key is not validated against this list. `trust_anchors` is the
    /// list of root CAs to trust. `intermediate_certs` is the sequence of
    /// intermediate certificates that the server sent in the TLS handshake.
    /// `time` is the time for which the validation is effective (usually the
    /// current time).
    pub fn verify_is_valid_tls_server_cert(
            &self, supported_sig_algs: &[&SignatureAlgorithm],
            &TLSServerTrustAnchors(trust_anchors): &TLSServerTrustAnchors,
            intermediate_certs: &[untrusted::Input], time: Time)
            -> Result<(), Error> {
        verify_cert::build_chain(verify_cert::EKU_SERVER_AUTH,
                                 supported_sig_algs, trust_anchors,
                                 intermediate_certs, &self.inner, time, 0)
    }

    /// Verifies that the end-entity certificate is valid for use by a TLS
    /// client.
    ///
    /// If the certificate is not valid for any of the given names then this
    /// fails with `Error::CertNotValidForName`.
    ///
    /// `supported_sig_algs` is the list of signature algorithms that are
    /// trusted for use in certificate signatures; the end-entity certificate's
    /// public key is not validated against this list. `trust_anchors` is the
    /// list of root CAs to trust. `intermediate_certs` is the sequence of
    /// intermediate certificates that the client sent in the TLS handshake.
    /// `cert` is the purported end-entity certificate of the client. `time` is
    /// the time for which the validation is effective (usually the current
    /// time).
    pub fn verify_is_valid_tls_client_cert(
            &self, supported_sig_algs: &[&SignatureAlgorithm],
            &TLSClientTrustAnchors(trust_anchors): &TLSClientTrustAnchors,
            intermediate_certs: &[untrusted::Input], time: Time)
            -> Result<(), Error> {
        verify_cert::build_chain(verify_cert::EKU_CLIENT_AUTH,
                                 supported_sig_algs, trust_anchors,
                                 intermediate_certs, &self.inner, time, 0)
    }

    /// Verifies that the certificate is valid for the given DNS host name.
    pub fn verify_is_valid_for_dns_name(&self, dns_name: DNSNameRef)
                                        -> Result<(), Error> {
        name::verify_cert_dns_name(&self, dns_name)
    }

    /// Verifies that the certificate is valid for at least one of the given DNS
    /// host names.
    ///
    /// If the certificate is not valid for any of the given names then this
    /// fails with `Error::CertNotValidForName`. Otherwise the DNS names for
    /// which the certificate is valid are returned.
    ///
    /// Requires the `std` default feature; i.e. this isn't available in
    /// `#![no_std]` configurations.
    #[cfg(feature = "std")]
    pub fn verify_is_valid_for_at_least_one_dns_name<'names, Names>(
            &self, dns_names: Names)
            -> Result<std::vec::Vec<DNSNameRef<'names>>, Error>
            where Names: Iterator<Item=DNSNameRef<'names>> {
        let result: std::vec::Vec<DNSNameRef<'names>> = dns_names
            .filter(|n| self.verify_is_valid_for_dns_name(*n).is_ok())
            .collect();
        if result.is_empty() {
            return Err(Error::CertNotValidForName);
        }
        Ok(result)
    }

    /// Verifies the signature `signature` of message `msg` using the
    /// certificate's public key.
    ///
    /// `signature_alg` is the algorithm to use to
    /// verify the signature; the certificate's public key is verified to be
    /// compatible with this algorithm.
    ///
    /// For TLS 1.2, `signature` corresponds to TLS's
    /// `DigitallySigned.signature` and `signature_alg` corresponds to TLS's
    /// `DigitallySigned.algorithm` of TLS type `SignatureAndHashAlgorithm`. In
    /// TLS 1.2 a single `SignatureAndHashAlgorithm` may map to multiple
    /// `SignatureAlgorithm`s. For example, a TLS 1.2
    /// `ignatureAndHashAlgorithm` of (ECDSA, SHA-256) may map to any or all
    /// of {`ECDSA_P256_SHA256`, `ECDSA_P384_SHA256`}, depending on how the TLS
    /// implementation is configured.
    ///
    /// For current TLS 1.3 drafts, `signature_alg` corresponds to TLS's
    /// `algorithm` fields of type `SignatureScheme`. There is (currently) a
    /// one-to-one correspondence between TLS 1.3's `SignatureScheme` and
    /// `SignatureAlgorithm`.
    pub fn verify_signature(&self, signature_alg: &SignatureAlgorithm,
                            msg: untrusted::Input,
                            signature: untrusted::Input) -> Result<(), Error> {
        signed_data::verify_signature(signature_alg, self.inner.spki, msg,
                                      signature)
    }
}


/// An error that occurs during certificate validation or name validation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    /// The encoding of some ASN.1 DER-encoded item is invalid.
    BadDER,

    /// The encoding of an ASN.1 DER-encoded time is invalid.
    BadDERTime,

    /// A CA certificate is veing used as an end-entity certificate.
    CAUsedAsEndEntity,

    /// The certificate is expired; i.e. the time it is being validated for is
    /// later than the certificate's notAfter time.
    CertExpired,

    /// The certificate is not valid for the name it is being validated for.
    CertNotValidForName,

    /// The certificate is not valid yet; i.e. the time it is being validated
    /// for is earlier than the certificate's notBefore time.
    CertNotValidYet,

    /// An end-entity certificate is being used as a CA certificate.
    EndEntityUsedAsCA,

    /// An X.509 extension is invalid.
    ExtensionValueInvalid,

    /// The certificate validity period (notBefore, notAfter) is invalid; e.g.
    /// the notAfter time is earlier than the notBefore time.
    InvalidCertValidity,

    /// The signature is invalid for the given public key.
    InvalidSignatureForPublicKey,

    /// The certificate violates one or more name constraints.
    NameConstraintViolation,

    /// The certificate violates one or more path length constraints.
    PathLenConstraintViolated,

    /// The algorithm in the TBSCertificate "signature" field of a certificate
    /// does not match the algorithm in the signature of the certificate.
    SignatureAlgorithmMismatch,

    /// The certificate is not valid for the Extended Key Usage for which it is
    /// being validated.
    RequiredEKUNotFound,

    /// A valid issuer for the certificate could not be found.
    UnknownIssuer,

    /// The certificate is not a v3 X.509 certificate.
    UnsupportedCertVersion,

    /// The certificate contains an unsupported critical extension.
    UnsupportedCriticalExtension,

    /// The signature's algorithm does not match the algorithm of the public
    /// key it is being validated for. This may be because the public key
    /// algorithm's OID isn't recognized (e.g. DSA), or the public key
    /// algorithm's parameters don't match the supported parameters for that
    /// algorithm (e.g. ECC keys for unsupported curves), or the public key
    /// algorithm and the signature algorithm simply don't match (e.g.
    /// verifying an RSA signature with an ECC public key).
    UnsupportedSignatureAlgorithmForPublicKey,

    /// The signature algorithm for a signature is not in the set of supported
    /// signature algorithms given.
    UnsupportedSignatureAlgorithm,
}

/// A trust anchor (a.k.a. root CA).
///
/// Traditionally, certificate verification libraries have represented trust
/// anchors as full X.509 root certificates. However, those certificates
/// contain a lot more data than is needed for verifying certificates. The
/// `TrustAnchor` representation allows an application to store just the
/// essential elements of trust anchors. The `webpki::trust_anchor_util` module
/// provides functions for converting X.509 certificates to to the minimized
/// `TrustAnchor` representation, either at runtime or in a build script.
#[derive(Debug)]
pub struct TrustAnchor<'a> {
    /// The value of the `subject` field of the trust anchor.
    pub subject: &'a [u8],

    /// The value of the `subjectPublicKeyInfo` field of the trust anchor.
    pub spki: &'a [u8],

    /// The value of a DER-encoded NameConstraints, containing name
    /// constraints to apply to the trust anchor, if any.
    pub name_constraints: Option<&'a [u8]>
}

/// Trust anchors which may be used for authenticating servers.
#[derive(Debug)]
pub struct TLSServerTrustAnchors<'a>(pub &'a [TrustAnchor<'a>]);

/// Trust anchors which may be used for authenticating clients.
#[derive(Debug)]
pub struct TLSClientTrustAnchors<'a>(pub &'a [TrustAnchor<'a>]);
