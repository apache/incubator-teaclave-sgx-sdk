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

use crate::{Decodable, Decoder, Encodable, Encoder};
use sgx_types::types::EnclaveIdentity;
use sgx_types::types::{
    AlignEc256PrivateKey, AlignEc256SharedKey, AlignKey128bit, AlignKey256bit, AlignMac128bit,
    AlignMac256bit, Key128bit, Key256bit, Mac128bit, Mac256bit,
};
use sgx_types::types::{
    Attributes, AttributesFlags, ConfigId, CpuSvn, KeyId, KeyName, KeyPolicy, KeyRequest,
    Measurement, MiscAttribute, MiscSelect, Report, Report2, Report2Body, Report2Mac, ReportBody,
    ReportData, TargetInfo, TeeAttributes, TeeCpuSvn, TeeInfo, TeeMeasurement, TeeReportData,
    TeeReportType, TeeTcbInfo, TeeTcbSvn,
};
use sgx_types::types::{BaseName, PsSecPropDesc, QuoteNonce, Spid};
use sgx_types::types::{Ec256PrivateKey, Ec256PublicKey, Ec256SharedKey, Ec256Signature};
use sgx_types::types::{
    Rsa2048Key, Rsa2048Param, Rsa2048PrivKey, Rsa2048PubKey, Rsa2048Signature, Rsa3072Key,
    Rsa3072Param, Rsa3072PrivKey, Rsa3072PubKey, Rsa3072Signature, RsaKeyType, RsaResult,
};
use sgx_types::types::{Sha1Hash, Sha256Hash, Sha384Hash, Sm3Hash};

impl Encodable for Sha1Hash {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Sha1Hash { hash: ref _h } = *self;
        e.emit_struct("Sha1Hash", 1usize, |e| -> _ {
            e.emit_struct_field("hash", 0usize, |e| -> _ { Encodable::encode(&*_h, e) })
        })
    }
}

impl Decodable for Sha1Hash {
    fn decode<D: Decoder>(d: &mut D) -> Result<Sha1Hash, D::Error> {
        d.read_struct("Sha1Hash", 1usize, |d| -> _ {
            Ok(Sha1Hash {
                hash: d.read_struct_field("hash", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Sha256Hash {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Sha256Hash { hash: ref _h } = *self;
        e.emit_struct("Sha256Hash", 1usize, |e| -> _ {
            e.emit_struct_field("hash", 0usize, |e| -> _ { Encodable::encode(&*_h, e) })
        })
    }
}

impl Decodable for Sha256Hash {
    fn decode<D: Decoder>(d: &mut D) -> Result<Sha256Hash, D::Error> {
        d.read_struct("Sha256Hash", 1usize, |d| -> _ {
            Ok(Sha256Hash {
                hash: d.read_struct_field("hash", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Sha384Hash {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Sha384Hash { hash: ref _h } = *self;
        e.emit_struct("Sha384Hash", 1usize, |e| -> _ {
            e.emit_struct_field("hash", 0usize, |e| -> _ { Encodable::encode(&*_h, e) })
        })
    }
}

impl Decodable for Sha384Hash {
    fn decode<D: Decoder>(d: &mut D) -> Result<Sha384Hash, D::Error> {
        d.read_struct("Sha384Hash", 1usize, |d| -> _ {
            Ok(Sha384Hash {
                hash: d.read_struct_field("hash", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Sm3Hash {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Sm3Hash { hash: ref _h } = *self;
        e.emit_struct("Sm3Hash", 1usize, |e| -> _ {
            e.emit_struct_field("hash", 0usize, |e| -> _ { Encodable::encode(&*_h, e) })
        })
    }
}

impl Decodable for Sm3Hash {
    fn decode<D: Decoder>(d: &mut D) -> Result<Sm3Hash, D::Error> {
        d.read_struct("Sm3Hash", 1usize, |d| -> _ {
            Ok(Sm3Hash {
                hash: d.read_struct_field("hash", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Ec256SharedKey {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Ec256SharedKey { s: ref _s } = *self;
        e.emit_struct("Ec256SharedKey", 1usize, |e| -> _ {
            e.emit_struct_field("s", 0usize, |e| -> _ { Encodable::encode(&*_s, e) })
        })
    }
}

impl Decodable for Ec256SharedKey {
    fn decode<D: Decoder>(d: &mut D) -> Result<Ec256SharedKey, D::Error> {
        d.read_struct("Ec256SharedKey", 1usize, |d| -> _ {
            Ok(Ec256SharedKey {
                s: d.read_struct_field("s", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Ec256PrivateKey {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Ec256PrivateKey { r: ref _r } = *self;
        e.emit_struct("Ec256PrivateKey", 1usize, |e| -> _ {
            e.emit_struct_field("r", 0usize, |e| -> _ { Encodable::encode(&*_r, e) })
        })
    }
}

impl Decodable for Ec256PrivateKey {
    fn decode<D: Decoder>(d: &mut D) -> Result<Ec256PrivateKey, D::Error> {
        d.read_struct("Ec256PrivateKey", 1usize, |d| -> _ {
            Ok(Ec256PrivateKey {
                r: d.read_struct_field("r", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Ec256PublicKey {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Ec256PublicKey {
            gx: ref _gx,
            gy: ref _gy,
        } = *self;
        e.emit_struct("Ec256PublicKey", 2usize, |e| -> _ {
            e.emit_struct_field("gx", 0usize, |e| -> _ { Encodable::encode(&*_gx, e) })?;
            e.emit_struct_field("gy", 1usize, |e| -> _ { Encodable::encode(&*_gy, e) })
        })
    }
}

impl Decodable for Ec256PublicKey {
    fn decode<D: Decoder>(d: &mut D) -> Result<Ec256PublicKey, D::Error> {
        d.read_struct("Ec256PublicKey", 2usize, |d| -> _ {
            Ok(Ec256PublicKey {
                gx: d.read_struct_field("gx", 0usize, Decodable::decode)?,
                gy: d.read_struct_field("gy", 1usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Ec256Signature {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Ec256Signature {
            x: ref _x,
            y: ref _y,
        } = *self;
        e.emit_struct("Ec256Signature", 2usize, |e| -> _ {
            e.emit_struct_field("x", 0usize, |e| -> _ { Encodable::encode(&*_x, e) })?;
            e.emit_struct_field("y", 1usize, |e| -> _ { Encodable::encode(&*_y, e) })
        })
    }
}

impl Decodable for Ec256Signature {
    fn decode<D: Decoder>(d: &mut D) -> Result<Ec256Signature, D::Error> {
        d.read_struct("Ec256Signature", 2usize, |d| -> _ {
            Ok(Ec256Signature {
                x: d.read_struct_field("x", 0usize, Decodable::decode)?,
                y: d.read_struct_field("y", 1usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for RsaKeyType {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        match *self {
            RsaKeyType::PrivateKey => e.emit_enum("RsaKeyType", |e| -> _ {
                e.emit_enum_variant("PrivateKey", 0usize, 0usize, |_e| -> _ { Ok(()) })
            }),
            RsaKeyType::PublicKey => e.emit_enum("RsaKeyType", |e| -> _ {
                e.emit_enum_variant("PublicKey", 1usize, 0usize, |_e| -> _ { Ok(()) })
            }),
        }
    }
}
impl Decodable for RsaKeyType {
    fn decode<D: Decoder>(d: &mut D) -> Result<RsaKeyType, D::Error> {
        d.read_enum("RsaKeyType", |d| -> _ {
            d.read_enum_variant(&["PrivateKey", "PublicKey"], |_d, i| -> _ {
                Ok(match i {
                    0usize => RsaKeyType::PrivateKey,
                    1usize => RsaKeyType::PublicKey,
                    _ => panic!("internal error: entered unreachable code"),
                })
            })
        })
    }
}

impl Encodable for RsaResult {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        match *self {
            RsaResult::Valid => e.emit_enum("RsaResult", |e| -> _ {
                e.emit_enum_variant("Valid", 0usize, 0usize, |_e| -> _ { Ok(()) })
            }),
            RsaResult::InvalidSignature => e.emit_enum("RsaResult", |e| -> _ {
                e.emit_enum_variant("InvalidSignature", 1usize, 0usize, |_e| -> _ { Ok(()) })
            }),
        }
    }
}
impl Decodable for RsaResult {
    fn decode<D: Decoder>(d: &mut D) -> Result<RsaResult, D::Error> {
        d.read_enum("RsaResult", |d| -> _ {
            d.read_enum_variant(&["Valid", "InvalidSignature"], |_d, i| -> _ {
                Ok(match i {
                    0usize => RsaResult::Valid,
                    1usize => RsaResult::InvalidSignature,
                    _ => panic!("internal error: entered unreachable code"),
                })
            })
        })
    }
}

impl Encodable for Rsa3072Param {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Rsa3072Param {
            n: ref _n,
            d: ref _d,
            e: ref _e,
            p: ref _p,
            q: ref _q,
            dmp1: ref _dmp1,
            dmq1: ref _dmq1,
            iqmp: ref _iqmp,
        } = *self;
        e.emit_struct("Rsa3072Param", 8usize, |e| -> _ {
            e.emit_struct_field("n", 0usize, |e| -> _ { Encodable::encode(&*_n, e) })?;
            e.emit_struct_field("d", 1usize, |e| -> _ { Encodable::encode(&*_d, e) })?;
            e.emit_struct_field("e", 2usize, |e| -> _ { Encodable::encode(&*_e, e) })?;
            e.emit_struct_field("p", 3usize, |e| -> _ { Encodable::encode(&*_p, e) })?;
            e.emit_struct_field("q", 4usize, |e| -> _ { Encodable::encode(&*_q, e) })?;
            e.emit_struct_field("dmp1", 5usize, |e| -> _ { Encodable::encode(&*_dmp1, e) })?;
            e.emit_struct_field("dmq1", 6usize, |e| -> _ { Encodable::encode(&*_dmq1, e) })?;
            e.emit_struct_field("iqmp", 7usize, |e| -> _ { Encodable::encode(&*_iqmp, e) })
        })
    }
}

impl Decodable for Rsa3072Param {
    fn decode<D: Decoder>(d: &mut D) -> Result<Rsa3072Param, D::Error> {
        d.read_struct("Rsa3072Param", 8usize, |d| -> _ {
            Ok(Rsa3072Param {
                n: d.read_struct_field("n", 0usize, Decodable::decode)?,
                d: d.read_struct_field("d", 1usize, Decodable::decode)?,
                e: d.read_struct_field("e", 2usize, Decodable::decode)?,
                p: d.read_struct_field("p", 3usize, Decodable::decode)?,
                q: d.read_struct_field("q", 4usize, Decodable::decode)?,
                dmp1: d.read_struct_field("dmp1", 5usize, Decodable::decode)?,
                dmq1: d.read_struct_field("dmq1", 6usize, Decodable::decode)?,
                iqmp: d.read_struct_field("iqmp", 7usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Rsa3072PubKey {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Rsa3072PubKey {
            modulus: ref _modulus,
            exponent: ref _exponent,
        } = *self;
        e.emit_struct("Rsa3072PubKey", 2usize, |e| -> _ {
            e.emit_struct_field("modulus", 0usize, |e| -> _ {
                Encodable::encode(&*_modulus, e)
            })?;
            e.emit_struct_field("exponent", 1usize, |e| -> _ {
                Encodable::encode(&*_exponent, e)
            })
        })
    }
}
impl Decodable for Rsa3072PubKey {
    fn decode<D: Decoder>(d: &mut D) -> Result<Rsa3072PubKey, D::Error> {
        d.read_struct("Rsa3072PubKey", 2usize, |d| -> _ {
            Ok(Rsa3072PubKey {
                modulus: d.read_struct_field("modulus", 0usize, Decodable::decode)?,
                exponent: d.read_struct_field("exponent", 1usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Rsa3072PrivKey {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Rsa3072PrivKey {
            modulus: ref _modulus,
            exponent: ref _exponent,
        } = *self;
        e.emit_struct("Rsa3072PrivKey", 2usize, |e| -> _ {
            e.emit_struct_field("modulus", 0usize, |e| -> _ {
                Encodable::encode(&*_modulus, e)
            })?;
            e.emit_struct_field("exponent", 1usize, |e| -> _ {
                Encodable::encode(&*_exponent, e)
            })
        })
    }
}
impl Decodable for Rsa3072PrivKey {
    fn decode<D: Decoder>(d: &mut D) -> Result<Rsa3072PrivKey, D::Error> {
        d.read_struct("Rsa3072PrivKey", 2usize, |d| -> _ {
            Ok(Rsa3072PrivKey {
                modulus: d.read_struct_field("modulus", 0usize, Decodable::decode)?,
                exponent: d.read_struct_field("exponent", 1usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Rsa3072Key {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Rsa3072Key {
            modulus: ref _modulus,
            d: ref _d,
            e: ref _e,
        } = *self;
        e.emit_struct("Rsa3072Key", 3usize, |e| -> _ {
            e.emit_struct_field("modulus", 0usize, |e| -> _ {
                Encodable::encode(&*_modulus, e)
            })?;
            e.emit_struct_field("d", 1usize, |e| -> _ { Encodable::encode(&*_d, e) })?;
            e.emit_struct_field("e", 2usize, |e| -> _ { Encodable::encode(&*_e, e) })
        })
    }
}
impl Decodable for Rsa3072Key {
    fn decode<D: Decoder>(d: &mut D) -> Result<Rsa3072Key, D::Error> {
        d.read_struct("Rsa3072Key", 3usize, |d| -> _ {
            Ok(Rsa3072Key {
                modulus: d.read_struct_field("modulus", 0usize, Decodable::decode)?,
                d: d.read_struct_field("d", 1usize, Decodable::decode)?,
                e: d.read_struct_field("e", 2usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Rsa3072Signature {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Rsa3072Signature {
            signature: ref _signature,
        } = *self;
        e.emit_struct("Rsa3072Signature", 1usize, |e| -> _ {
            e.emit_struct_field("signature", 0usize, |e| -> _ {
                Encodable::encode(&*_signature, e)
            })
        })
    }
}

impl Decodable for Rsa3072Signature {
    fn decode<D: Decoder>(d: &mut D) -> Result<Rsa3072Signature, D::Error> {
        d.read_struct("Rsa3072Signature", 1usize, |d| -> _ {
            Ok(Rsa3072Signature {
                signature: d.read_struct_field("signature", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Rsa2048Param {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Rsa2048Param {
            n: ref _n,
            d: ref _d,
            e: ref _e,
            p: ref _p,
            q: ref _q,
            dmp1: ref _dmp1,
            dmq1: ref _dmq1,
            iqmp: ref _iqmp,
        } = *self;
        e.emit_struct("Rsa2048Param", 8usize, |e| -> _ {
            e.emit_struct_field("n", 0usize, |e| -> _ { Encodable::encode(&*_n, e) })?;
            e.emit_struct_field("d", 1usize, |e| -> _ { Encodable::encode(&*_d, e) })?;
            e.emit_struct_field("e", 2usize, |e| -> _ { Encodable::encode(&*_e, e) })?;
            e.emit_struct_field("p", 3usize, |e| -> _ { Encodable::encode(&*_p, e) })?;
            e.emit_struct_field("q", 4usize, |e| -> _ { Encodable::encode(&*_q, e) })?;
            e.emit_struct_field("dmp1", 5usize, |e| -> _ { Encodable::encode(&*_dmp1, e) })?;
            e.emit_struct_field("dmq1", 6usize, |e| -> _ { Encodable::encode(&*_dmq1, e) })?;
            e.emit_struct_field("iqmp", 7usize, |e| -> _ { Encodable::encode(&*_iqmp, e) })
        })
    }
}

impl Decodable for Rsa2048Param {
    fn decode<D: Decoder>(d: &mut D) -> Result<Rsa2048Param, D::Error> {
        d.read_struct("Rsa2048Param", 8usize, |d| -> _ {
            Ok(Rsa2048Param {
                n: d.read_struct_field("n", 0usize, Decodable::decode)?,
                d: d.read_struct_field("d", 1usize, Decodable::decode)?,
                e: d.read_struct_field("e", 2usize, Decodable::decode)?,
                p: d.read_struct_field("p", 3usize, Decodable::decode)?,
                q: d.read_struct_field("q", 4usize, Decodable::decode)?,
                dmp1: d.read_struct_field("dmp1", 5usize, Decodable::decode)?,
                dmq1: d.read_struct_field("dmq1", 6usize, Decodable::decode)?,
                iqmp: d.read_struct_field("iqmp", 7usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Rsa2048PubKey {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Rsa2048PubKey {
            modulus: ref _modulus,
            exponent: ref _exponent,
        } = *self;
        e.emit_struct("Rsa2048PubKey", 2usize, |e| -> _ {
            e.emit_struct_field("modulus", 0usize, |e| -> _ {
                Encodable::encode(&*_modulus, e)
            })?;
            e.emit_struct_field("exponent", 1usize, |e| -> _ {
                Encodable::encode(&*_exponent, e)
            })
        })
    }
}
impl Decodable for Rsa2048PubKey {
    fn decode<D: Decoder>(d: &mut D) -> Result<Rsa2048PubKey, D::Error> {
        d.read_struct("Rsa2048PubKey", 2usize, |d| -> _ {
            Ok(Rsa2048PubKey {
                modulus: d.read_struct_field("modulus", 0usize, Decodable::decode)?,
                exponent: d.read_struct_field("exponent", 1usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Rsa2048PrivKey {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Rsa2048PrivKey {
            modulus: ref _modulus,
            exponent: ref _exponent,
        } = *self;
        e.emit_struct("Rsa2048PrivKey", 2usize, |e| -> _ {
            e.emit_struct_field("modulus", 0usize, |e| -> _ {
                Encodable::encode(&*_modulus, e)
            })?;
            e.emit_struct_field("exponent", 1usize, |e| -> _ {
                Encodable::encode(&*_exponent, e)
            })
        })
    }
}
impl Decodable for Rsa2048PrivKey {
    fn decode<D: Decoder>(d: &mut D) -> Result<Rsa2048PrivKey, D::Error> {
        d.read_struct("Rsa2048PrivKey", 2usize, |d| -> _ {
            Ok(Rsa2048PrivKey {
                modulus: d.read_struct_field("modulus", 0usize, Decodable::decode)?,
                exponent: d.read_struct_field("exponent", 1usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Rsa2048Key {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Rsa2048Key {
            modulus: ref _modulus,
            d: ref _d,
            e: ref _e,
        } = *self;
        e.emit_struct("Rsa2048Key", 3usize, |e| -> _ {
            e.emit_struct_field("modulus", 0usize, |e| -> _ {
                Encodable::encode(&*_modulus, e)
            })?;
            e.emit_struct_field("d", 1usize, |e| -> _ { Encodable::encode(&*_d, e) })?;
            e.emit_struct_field("e", 2usize, |e| -> _ { Encodable::encode(&*_e, e) })
        })
    }
}
impl Decodable for Rsa2048Key {
    fn decode<D: Decoder>(d: &mut D) -> Result<Rsa2048Key, D::Error> {
        d.read_struct("Rsa2048Key", 3usize, |d| -> _ {
            Ok(Rsa2048Key {
                modulus: d.read_struct_field("modulus", 0usize, Decodable::decode)?,
                d: d.read_struct_field("d", 1usize, Decodable::decode)?,
                e: d.read_struct_field("e", 2usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Rsa2048Signature {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Rsa2048Signature {
            signature: ref _signature,
        } = *self;
        e.emit_struct("Rsa2048Signature", 1usize, |e| -> _ {
            e.emit_struct_field("signature", 0usize, |e| -> _ {
                Encodable::encode(&*_signature, e)
            })
        })
    }
}

impl Decodable for Rsa2048Signature {
    fn decode<D: Decoder>(d: &mut D) -> Result<Rsa2048Signature, D::Error> {
        d.read_struct("Rsa2048Signature", 1usize, |d| -> _ {
            Ok(Rsa2048Signature {
                signature: d.read_struct_field("signature", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for AlignKey128bit {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let key = &self.key;
        e.emit_struct("AlignKey128bit", 1usize, |e| -> _ {
            e.emit_struct_field("key", 0usize, |e| -> _ { Encodable::encode(key, e) })
        })
    }
}

impl Decodable for AlignKey128bit {
    fn decode<D: Decoder>(d: &mut D) -> Result<AlignKey128bit, D::Error> {
        d.read_struct("AlignKey128bit", 1usize, |d| -> _ {
            Ok(From::<Key128bit>::from(d.read_struct_field(
                "key",
                0usize,
                Decodable::decode,
            )?))
        })
    }
}

impl Encodable for AlignKey256bit {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let key = &self.key;
        e.emit_struct("AlignKey256bit", 1usize, |e| -> _ {
            e.emit_struct_field("key", 0usize, |e| -> _ { Encodable::encode(key, e) })
        })
    }
}

impl Decodable for AlignKey256bit {
    fn decode<D: Decoder>(d: &mut D) -> Result<AlignKey256bit, D::Error> {
        d.read_struct("AlignKey256bit", 1usize, |d| -> _ {
            Ok(From::<Key256bit>::from(d.read_struct_field(
                "key",
                0usize,
                Decodable::decode,
            )?))
        })
    }
}

impl Encodable for AlignMac128bit {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let mac = &self.mac;
        e.emit_struct("AlignMac128bit", 1usize, |e| -> _ {
            e.emit_struct_field("mac", 0usize, |e| -> _ { Encodable::encode(mac, e) })
        })
    }
}

impl Decodable for AlignMac128bit {
    fn decode<D: Decoder>(d: &mut D) -> Result<AlignMac128bit, D::Error> {
        d.read_struct("AlignMac128bit", 1usize, |d| -> _ {
            Ok(From::<Mac128bit>::from(d.read_struct_field(
                "mac",
                0usize,
                Decodable::decode,
            )?))
        })
    }
}

impl Encodable for AlignMac256bit {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let mac = &self.mac;
        e.emit_struct("AlignMac256bit", 1usize, |e| -> _ {
            e.emit_struct_field("mac", 0usize, |e| -> _ { Encodable::encode(mac, e) })
        })
    }
}

impl Decodable for AlignMac256bit {
    fn decode<D: Decoder>(d: &mut D) -> Result<AlignMac256bit, D::Error> {
        d.read_struct("AlignMac256bit", 1usize, |d| -> _ {
            Ok(From::<Mac256bit>::from(d.read_struct_field(
                "mac",
                0usize,
                Decodable::decode,
            )?))
        })
    }
}

impl Encodable for AlignEc256SharedKey {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let key = &self.key;
        e.emit_struct("AlignEc256SharedKey", 1usize, |e| -> _ {
            e.emit_struct_field("key", 0usize, |e| -> _ { Encodable::encode(key, e) })
        })
    }
}

impl Decodable for AlignEc256SharedKey {
    fn decode<D: Decoder>(d: &mut D) -> Result<AlignEc256SharedKey, D::Error> {
        d.read_struct("AlignEc256SharedKey", 1usize, |d| -> _ {
            Ok(From::<Ec256SharedKey>::from(d.read_struct_field(
                "key",
                0usize,
                Decodable::decode,
            )?))
        })
    }
}

impl Encodable for AlignEc256PrivateKey {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let key = &self.key;
        e.emit_struct("AlignEc256PrivateKey", 1usize, |e| -> _ {
            e.emit_struct_field("key", 0usize, |e| -> _ { Encodable::encode(key, e) })
        })
    }
}

impl Decodable for AlignEc256PrivateKey {
    fn decode<D: Decoder>(d: &mut D) -> Result<AlignEc256PrivateKey, D::Error> {
        d.read_struct("AlignEc256PrivateKey", 1usize, |d| -> _ {
            Ok(From::<Ec256PrivateKey>::from(d.read_struct_field(
                "key",
                0usize,
                Decodable::decode,
            )?))
        })
    }
}

impl Encodable for AttributesFlags {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let bits = self.bits();
        e.emit_struct("AttributesFlags", 1usize, |e| -> _ {
            e.emit_struct_field("field", 0usize, |e| -> _ { Encodable::encode(&bits, e) })
        })
    }
}

impl Decodable for AttributesFlags {
    fn decode<D: Decoder>(d: &mut D) -> Result<AttributesFlags, D::Error> {
        d.read_struct("AttributesFlags", 1usize, |d| -> _ {
            Ok(AttributesFlags::from_bits_truncate(d.read_struct_field(
                "field",
                0usize,
                Decodable::decode,
            )?))
        })
    }
}

impl Encodable for MiscSelect {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let bits = self.bits();
        e.emit_struct("MiscSelect", 1usize, |e| -> _ {
            e.emit_struct_field("field", 0usize, |e| -> _ { Encodable::encode(&bits, e) })
        })
    }
}

impl Decodable for MiscSelect {
    fn decode<D: Decoder>(d: &mut D) -> Result<MiscSelect, D::Error> {
        d.read_struct("MiscSelect", 1usize, |d| -> _ {
            Ok(MiscSelect::from_bits_truncate(d.read_struct_field(
                "field",
                0usize,
                Decodable::decode,
            )?))
        })
    }
}

impl Encodable for Attributes {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Attributes {
            flags: ref _flags,
            xfrm: ref _xfrm,
        } = *self;
        e.emit_struct("Attributes", 2usize, |e| -> _ {
            e.emit_struct_field("flags", 0usize, |e| -> _ { Encodable::encode(&*_flags, e) })?;
            e.emit_struct_field("xfrm", 1usize, |e| -> _ { Encodable::encode(&*_xfrm, e) })
        })
    }
}

impl Decodable for Attributes {
    fn decode<D: Decoder>(d: &mut D) -> Result<Attributes, D::Error> {
        d.read_struct("Attributes", 2usize, |d| -> _ {
            Ok(Attributes {
                flags: d.read_struct_field("flags", 0usize, Decodable::decode)?,
                xfrm: d.read_struct_field("xfrm", 1usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for MiscAttribute {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let MiscAttribute {
            secs_attr: ref _secs_attr,
            misc_select: ref _misc_select,
        } = *self;
        e.emit_struct("MiscAttribute", 2usize, |e| -> _ {
            e.emit_struct_field("secs_attr", 0usize, |e| -> _ {
                Encodable::encode(&*_secs_attr, e)
            })?;
            e.emit_struct_field("misc_select", 1usize, |e| -> _ {
                Encodable::encode(&*_secs_attr, e)
            })
        })
    }
}

impl Decodable for MiscAttribute {
    fn decode<D: Decoder>(d: &mut D) -> Result<MiscAttribute, D::Error> {
        d.read_struct("MiscAttribute", 2usize, |d| -> _ {
            Ok(MiscAttribute {
                secs_attr: d.read_struct_field("secs_attr", 0usize, Decodable::decode)?,
                misc_select: d.read_struct_field("misc_select", 1usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for CpuSvn {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let CpuSvn { svn: ref _svn } = *self;
        e.emit_struct("CpuSvn", 1usize, |e| -> _ {
            e.emit_struct_field("svn", 0usize, |e| -> _ { Encodable::encode(&*_svn, e) })
        })
    }
}

impl Decodable for CpuSvn {
    fn decode<D: Decoder>(d: &mut D) -> Result<CpuSvn, D::Error> {
        d.read_struct("CpuSvn", 1usize, |d| -> _ {
            Ok(CpuSvn {
                svn: d.read_struct_field("svn", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for ConfigId {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let ConfigId { id: ref _id } = *self;
        e.emit_struct("ConfigId", 1usize, |e| -> _ {
            e.emit_struct_field("id", 0usize, |e| -> _ { Encodable::encode(&*_id, e) })
        })
    }
}

impl Decodable for ConfigId {
    fn decode<D: Decoder>(d: &mut D) -> Result<ConfigId, D::Error> {
        d.read_struct("ConfigId", 1usize, |d| -> _ {
            Ok(ConfigId {
                id: d.read_struct_field("id", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for TeeAttributes {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let TeeAttributes { a: ref _a } = *self;
        e.emit_struct("TeeAttributes", 1usize, |e| -> _ {
            e.emit_struct_field("a", 0usize, |e| -> _ { Encodable::encode(&*_a, e) })
        })
    }
}

impl Decodable for TeeAttributes {
    fn decode<D: Decoder>(d: &mut D) -> Result<TeeAttributes, D::Error> {
        d.read_struct("TeeAttributes", 1usize, |d| -> _ {
            Ok(TeeAttributes {
                a: d.read_struct_field("a", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for TeeCpuSvn {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let TeeCpuSvn { svn: ref _svn } = *self;
        e.emit_struct("TeeCpuSvn", 1usize, |e| -> _ {
            e.emit_struct_field("svn", 0usize, |e| -> _ { Encodable::encode(&*_svn, e) })
        })
    }
}

impl Decodable for TeeCpuSvn {
    fn decode<D: Decoder>(d: &mut D) -> Result<TeeCpuSvn, D::Error> {
        d.read_struct("TeeCpuSvn", 1usize, |d| -> _ {
            Ok(TeeCpuSvn {
                svn: d.read_struct_field("svn", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Measurement {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Measurement { m: ref _m } = *self;
        e.emit_struct("Measurement", 1usize, |e| -> _ {
            e.emit_struct_field("m", 0usize, |e| -> _ { Encodable::encode(&*_m, e) })
        })
    }
}

impl Decodable for Measurement {
    fn decode<D: Decoder>(d: &mut D) -> Result<Measurement, D::Error> {
        d.read_struct("Measurement", 1usize, |d| -> _ {
            Ok(Measurement {
                m: d.read_struct_field("m", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for TeeMeasurement {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let TeeMeasurement { m: ref _m } = *self;
        e.emit_struct("TeeMeasurement", 1usize, |e| -> _ {
            e.emit_struct_field("m", 0usize, |e| -> _ { Encodable::encode(&*_m, e) })
        })
    }
}

impl Decodable for TeeMeasurement {
    fn decode<D: Decoder>(d: &mut D) -> Result<TeeMeasurement, D::Error> {
        d.read_struct("TeeMeasurement", 1usize, |d| -> _ {
            Ok(TeeMeasurement {
                m: d.read_struct_field("m", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for KeyId {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let KeyId { id: ref _id } = *self;
        e.emit_struct("KeyId", 1usize, |e| -> _ {
            e.emit_struct_field("id", 0usize, |e| -> _ { Encodable::encode(&*_id, e) })
        })
    }
}

impl Decodable for KeyId {
    fn decode<D: Decoder>(d: &mut D) -> Result<KeyId, D::Error> {
        d.read_struct("KeyId", 1usize, |d| -> _ {
            Ok(KeyId {
                id: d.read_struct_field("id", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for KeyName {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        match *self {
            KeyName::EInitToken => e.emit_enum("KeyName", |e| -> _ {
                e.emit_enum_variant("EInitToken", 0usize, 0usize, |_e| -> _ { Ok(()) })
            }),
            KeyName::Provision => e.emit_enum("KeyName", |e| -> _ {
                e.emit_enum_variant("Provision", 1usize, 0usize, |_e| -> _ { Ok(()) })
            }),
            KeyName::ProvisionSeal => e.emit_enum("KeyName", |e| -> _ {
                e.emit_enum_variant("ProvisionSeal", 2usize, 0usize, |_e| -> _ { Ok(()) })
            }),
            KeyName::Report => e.emit_enum("KeyName", |e| -> _ {
                e.emit_enum_variant("Report", 3usize, 0usize, |_e| -> _ { Ok(()) })
            }),
            KeyName::Seal => e.emit_enum("KeyName", |e| -> _ {
                e.emit_enum_variant("Seal", 4usize, 0usize, |_e| -> _ { Ok(()) })
            }),
        }
    }
}

impl Decodable for KeyName {
    fn decode<D: Decoder>(d: &mut D) -> Result<KeyName, D::Error> {
        d.read_enum("KeyName", |d| -> _ {
            d.read_enum_variant(
                &["EInitToken", "Provision", "ProvisionSeal", "Report", "Seal"],
                |_d, i| -> _ {
                    Ok(match i {
                        0usize => KeyName::EInitToken,
                        1usize => KeyName::Provision,
                        2usize => KeyName::ProvisionSeal,
                        3usize => KeyName::Report,
                        4usize => KeyName::Seal,
                        _ => panic!("internal error: entered unreachable code"),
                    })
                },
            )
        })
    }
}

impl Encodable for KeyPolicy {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let bits = self.bits();
        e.emit_struct("KeyPolicy", 1usize, |e| -> _ {
            e.emit_struct_field("field", 0usize, |e| -> _ { Encodable::encode(&bits, e) })
        })
    }
}

impl Decodable for KeyPolicy {
    fn decode<D: Decoder>(d: &mut D) -> Result<KeyPolicy, D::Error> {
        d.read_struct("KeyPolicy", 1usize, |d| -> _ {
            Ok(KeyPolicy::from_bits_truncate(d.read_struct_field(
                "field",
                0usize,
                Decodable::decode,
            )?))
        })
    }
}

impl Encodable for ReportData {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let ReportData { d: ref _d } = *self;
        e.emit_struct("ReportData", 1usize, |e| -> _ {
            e.emit_struct_field("d", 0usize, |e| -> _ { Encodable::encode(&*_d, e) })
        })
    }
}

impl Decodable for ReportData {
    fn decode<D: Decoder>(d: &mut D) -> Result<ReportData, D::Error> {
        d.read_struct("ReportData", 1usize, |d| -> _ {
            Ok(ReportData {
                d: d.read_struct_field("d", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for TeeReportData {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let TeeReportData { d: ref _d } = *self;
        e.emit_struct("TeeReportData", 1usize, |e| -> _ {
            e.emit_struct_field("d", 0usize, |e| -> _ { Encodable::encode(&*_d, e) })
        })
    }
}

impl Decodable for TeeReportData {
    fn decode<D: Decoder>(d: &mut D) -> Result<TeeReportData, D::Error> {
        d.read_struct("TeeReportData", 1usize, |d| -> _ {
            Ok(TeeReportData {
                d: d.read_struct_field("d", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for TeeReportType {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let TeeReportType {
            report_type: ref _report_type,
            subtype: ref _subtype,
            version: ref _version,
            reserved: ref _reserved,
        } = *self;
        e.emit_struct("TeeReportType", 4usize, |e| -> _ {
            e.emit_struct_field("report_type", 0usize, |e| -> _ {
                Encodable::encode(&*_report_type, e)
            })?;
            e.emit_struct_field("subtype", 1usize, |e| -> _ {
                Encodable::encode(&*_subtype, e)
            })?;
            e.emit_struct_field("version", 2usize, |e| -> _ {
                Encodable::encode(&*_version, e)
            })?;
            e.emit_struct_field("reserved", 3usize, |e| -> _ {
                Encodable::encode(&*_reserved, e)
            })
        })
    }
}

impl Decodable for TeeReportType {
    fn decode<D: Decoder>(d: &mut D) -> Result<TeeReportType, D::Error> {
        d.read_struct("TeeReportType", 4usize, |d| -> _ {
            Ok(TeeReportType {
                report_type: d.read_struct_field("report_type", 0usize, Decodable::decode)?,
                subtype: d.read_struct_field("subtype", 1usize, Decodable::decode)?,
                version: d.read_struct_field("version", 2usize, Decodable::decode)?,
                reserved: d.read_struct_field("reserved", 3usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for ReportBody {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let ReportBody {
            cpu_svn: ref _cpu_svn,
            misc_select: ref _misc_select,
            reserved1: ref _reserved1,
            isv_ext_prod_id: ref _isv_ext_prod_id,
            attributes: ref _attributes,
            mr_enclave: ref _mr_enclave,
            reserved2: ref _reserved2,
            mr_signer: ref _mr_signer,
            reserved3: ref _reserved3,
            config_id: ref _config_id,
            isv_prod_id: ref _isv_prod_id,
            isv_svn: ref _isv_svn,
            config_svn: ref _config_svn,
            reserved4: ref _reserved4,
            isv_family_id: ref _isv_family_id,
            report_data: ref _report_data,
        } = *self;
        e.emit_struct("ReportBody", 16usize, |e| -> _ {
            e.emit_struct_field("cpu_svn", 0usize, |e| -> _ {
                Encodable::encode(&*_cpu_svn, e)
            })?;
            e.emit_struct_field("misc_select", 1usize, |e| -> _ {
                Encodable::encode(&*_misc_select, e)
            })?;
            e.emit_struct_field("reserved1", 2usize, |e| -> _ {
                Encodable::encode(&*_reserved1, e)
            })?;
            e.emit_struct_field("isv_ext_prod_id", 3usize, |e| -> _ {
                Encodable::encode(&*_isv_ext_prod_id, e)
            })?;
            e.emit_struct_field("attributes", 4usize, |e| -> _ {
                Encodable::encode(&*_attributes, e)
            })?;
            e.emit_struct_field("mr_enclave", 5usize, |e| -> _ {
                Encodable::encode(&*_mr_enclave, e)
            })?;
            e.emit_struct_field("reserved2", 6usize, |e| -> _ {
                Encodable::encode(&*_reserved2, e)
            })?;
            e.emit_struct_field("mr_signer", 7usize, |e| -> _ {
                Encodable::encode(&*_mr_signer, e)
            })?;
            e.emit_struct_field("reserved3", 8usize, |e| -> _ {
                Encodable::encode(&*_reserved3, e)
            })?;
            e.emit_struct_field("config_id", 9usize, |e| -> _ {
                Encodable::encode(&*_config_id, e)
            })?;
            e.emit_struct_field("isv_prod_id", 10usize, |e| -> _ {
                Encodable::encode(&*_isv_prod_id, e)
            })?;
            e.emit_struct_field("isv_svn", 11usize, |e| -> _ {
                Encodable::encode(&*_isv_svn, e)
            })?;
            e.emit_struct_field("config_svn", 12usize, |e| -> _ {
                Encodable::encode(&*_config_svn, e)
            })?;
            e.emit_struct_field("reserved4", 13usize, |e| -> _ {
                Encodable::encode(&*_reserved4, e)
            })?;
            e.emit_struct_field("isv_family_id", 14usize, |e| -> _ {
                Encodable::encode(&*_isv_family_id, e)
            })?;
            e.emit_struct_field("report_data", 15usize, |e| -> _ {
                Encodable::encode(&*_report_data, e)
            })
        })
    }
}

impl Decodable for ReportBody {
    fn decode<D: Decoder>(d: &mut D) -> Result<ReportBody, D::Error> {
        d.read_struct("ReportBody", 16usize, |d| -> _ {
            Ok(ReportBody {
                cpu_svn: d.read_struct_field("cpu_svn", 0usize, Decodable::decode)?,
                misc_select: d.read_struct_field("misc_select", 1usize, Decodable::decode)?,
                reserved1: d.read_struct_field("reserved1", 2usize, Decodable::decode)?,
                isv_ext_prod_id: d.read_struct_field(
                    "isv_ext_prod_id",
                    3usize,
                    Decodable::decode,
                )?,
                attributes: d.read_struct_field("attributes", 4usize, Decodable::decode)?,
                mr_enclave: d.read_struct_field("mr_enclave", 5usize, Decodable::decode)?,
                reserved2: d.read_struct_field("reserved2", 6usize, Decodable::decode)?,
                mr_signer: d.read_struct_field("mr_signer", 7usize, Decodable::decode)?,
                reserved3: d.read_struct_field("reserved3", 8usize, Decodable::decode)?,
                config_id: d.read_struct_field("config_id", 9usize, Decodable::decode)?,
                isv_prod_id: d.read_struct_field("isv_prod_id", 10usize, Decodable::decode)?,
                isv_svn: d.read_struct_field("isv_svn", 11usize, Decodable::decode)?,
                config_svn: d.read_struct_field("config_svn", 12usize, Decodable::decode)?,
                reserved4: d.read_struct_field("reserved4", 13usize, Decodable::decode)?,
                isv_family_id: d.read_struct_field("isv_family_id", 14usize, Decodable::decode)?,
                report_data: d.read_struct_field("report_data", 15usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Report2Mac {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Report2Mac {
            report_type: ref _report_type,
            reserved1: ref _reserved1,
            cpu_svn: ref _cpu_svn,
            tee_tcb_info_hash: ref _tee_tcb_info_hash,
            tee_info_hash: ref _tee_info_hash,
            report_data: ref _report_data,
            reserved2: ref _reserved2,
            mac: ref _mac,
        } = *self;
        e.emit_struct("Report2Mac", 8usize, |e| -> _ {
            e.emit_struct_field("report_type", 0usize, |e| -> _ {
                Encodable::encode(&*_report_type, e)
            })?;
            e.emit_struct_field("reserved1", 1usize, |e| -> _ {
                Encodable::encode(&*_reserved1, e)
            })?;
            e.emit_struct_field("cpu_svn", 2usize, |e| -> _ {
                Encodable::encode(&*_cpu_svn, e)
            })?;
            e.emit_struct_field("tee_tcb_info_hash", 3usize, |e| -> _ {
                Encodable::encode(&*_tee_tcb_info_hash, e)
            })?;
            e.emit_struct_field("tee_info_hash", 4usize, |e| -> _ {
                Encodable::encode(&*_tee_info_hash, e)
            })?;
            e.emit_struct_field("report_data", 5usize, |e| -> _ {
                Encodable::encode(&*_report_data, e)
            })?;
            e.emit_struct_field("reserved2", 6usize, |e| -> _ {
                Encodable::encode(&*_reserved2, e)
            })?;
            e.emit_struct_field("mac", 7usize, |e| -> _ { Encodable::encode(&*_mac, e) })
        })
    }
}

impl Decodable for Report2Mac {
    fn decode<D: Decoder>(d: &mut D) -> Result<Report2Mac, D::Error> {
        d.read_struct("Report2Mac", 16usize, |d| -> _ {
            Ok(Report2Mac {
                report_type: d.read_struct_field("report_type", 0usize, Decodable::decode)?,
                reserved1: d.read_struct_field("reserved1", 1usize, Decodable::decode)?,
                cpu_svn: d.read_struct_field("cpu_svn", 2usize, Decodable::decode)?,
                tee_tcb_info_hash: d.read_struct_field(
                    "tee_tcb_info_hash",
                    3usize,
                    Decodable::decode,
                )?,
                tee_info_hash: d.read_struct_field("tee_info_hash", 4usize, Decodable::decode)?,
                report_data: d.read_struct_field("report_data", 5usize, Decodable::decode)?,
                reserved2: d.read_struct_field("reserved2", 6usize, Decodable::decode)?,
                mac: d.read_struct_field("mac", 7usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Report {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Report {
            body: ref _body,
            key_id: ref _key_id,
            mac: ref _mac,
        } = *self;
        e.emit_struct("Report", 3usize, |e| -> _ {
            e.emit_struct_field("body", 0usize, |e| -> _ { Encodable::encode(&*_body, e) })?;
            e.emit_struct_field("key_id", 1usize, |e| -> _ {
                Encodable::encode(&*_key_id, e)
            })?;
            e.emit_struct_field("mac", 2usize, |e| -> _ { Encodable::encode(&*_mac, e) })
        })
    }
}

impl Decodable for Report {
    fn decode<D: Decoder>(d: &mut D) -> Result<Report, D::Error> {
        d.read_struct("Report", 3usize, |d| -> _ {
            Ok(Report {
                body: d.read_struct_field("body", 0usize, Decodable::decode)?,
                key_id: d.read_struct_field("key_id", 1usize, Decodable::decode)?,
                mac: d.read_struct_field("mac", 2usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Report2 {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Report2 {
            report_mac: ref _report_mac,
            tee_tcb_info: ref _tee_tcb_info,
            reserved: ref _reserved,
            tee_info: ref _tee_info,
        } = *self;
        e.emit_struct("Report2", 4usize, |e| -> _ {
            e.emit_struct_field("report_mac", 0usize, |e| -> _ {
                Encodable::encode(&*_report_mac, e)
            })?;
            e.emit_struct_field("tee_tcb_info", 1usize, |e| -> _ {
                Encodable::encode(&*_tee_tcb_info, e)
            })?;
            e.emit_struct_field("reserved", 2usize, |e| -> _ {
                Encodable::encode(&*_reserved, e)
            })?;
            e.emit_struct_field("tee_info", 3usize, |e| -> _ {
                Encodable::encode(&*_tee_info, e)
            })
        })
    }
}

impl Decodable for Report2 {
    fn decode<D: Decoder>(d: &mut D) -> Result<Report2, D::Error> {
        d.read_struct("Report2", 3usize, |d| -> _ {
            Ok(Report2 {
                report_mac: d.read_struct_field("report_mac", 0usize, Decodable::decode)?,
                tee_tcb_info: d.read_struct_field("tee_tcb_info", 1usize, Decodable::decode)?,
                reserved: d.read_struct_field("reserved", 2usize, Decodable::decode)?,
                tee_info: d.read_struct_field("tee_info", 3usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for TargetInfo {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let TargetInfo {
            mr_enclave: ref _mr_enclave,
            attributes: ref _attributes,
            reserved1: ref _reserved1,
            config_svn: ref _config_svn,
            misc_select: ref _misc_select,
            reserved2: ref _reserved2,
            config_id: ref _config_id,
            reserved3: ref _reserved3,
        } = *self;
        e.emit_struct("TargetInfo", 8usize, |e| -> _ {
            e.emit_struct_field("mr_enclave", 0usize, |e| -> _ {
                Encodable::encode(&*_mr_enclave, e)
            })?;
            e.emit_struct_field("attributes", 1usize, |e| -> _ {
                Encodable::encode(&*_attributes, e)
            })?;
            e.emit_struct_field("reserved1", 2usize, |e| -> _ {
                Encodable::encode(&*_reserved1, e)
            })?;
            e.emit_struct_field("config_svn", 3usize, |e| -> _ {
                Encodable::encode(&*_config_svn, e)
            })?;
            e.emit_struct_field("misc_select", 4usize, |e| -> _ {
                Encodable::encode(&*_misc_select, e)
            })?;
            e.emit_struct_field("reserved2", 5usize, |e| -> _ {
                Encodable::encode(&*_reserved2, e)
            })?;
            e.emit_struct_field("config_id", 6usize, |e| -> _ {
                Encodable::encode(&*_config_id, e)
            })?;
            e.emit_struct_field("reserved3", 7usize, |e| -> _ {
                Encodable::encode(&*_reserved3, e)
            })
        })
    }
}

impl Decodable for TargetInfo {
    fn decode<D: Decoder>(d: &mut D) -> Result<TargetInfo, D::Error> {
        d.read_struct("TargetInfo", 8usize, |d| -> _ {
            Ok(TargetInfo {
                mr_enclave: d.read_struct_field("mr_enclave", 0usize, Decodable::decode)?,
                attributes: d.read_struct_field("attributes", 1usize, Decodable::decode)?,
                reserved1: d.read_struct_field("reserved1", 2usize, Decodable::decode)?,
                config_svn: d.read_struct_field("config_svn", 3usize, Decodable::decode)?,
                misc_select: d.read_struct_field("misc_select", 4usize, Decodable::decode)?,
                reserved2: d.read_struct_field("reserved2", 5usize, Decodable::decode)?,
                config_id: d.read_struct_field("config_id", 6usize, Decodable::decode)?,
                reserved3: d.read_struct_field("reserved3", 7usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for KeyRequest {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let KeyRequest {
            key_name: ref _key_name,
            key_policy: ref _key_policy,
            isv_svn: ref _isv_svn,
            reserved1: ref _reserved1,
            cpu_svn: ref _cpu_svn,
            attribute_mask: ref _attribute_mask,
            key_id: ref _key_id,
            misc_mask: ref _misc_mask,
            config_svn: ref _config_svn,
            reserved2: ref _reserved2,
        } = *self;
        e.emit_struct("KeyRequest", 10usize, |e| -> _ {
            e.emit_struct_field("key_name", 0usize, |e| -> _ {
                Encodable::encode(&*_key_name, e)
            })?;
            e.emit_struct_field("key_policy", 1usize, |e| -> _ {
                Encodable::encode(&*_key_policy, e)
            })?;
            e.emit_struct_field("isv_svn", 2usize, |e| -> _ {
                Encodable::encode(&*_isv_svn, e)
            })?;
            e.emit_struct_field("reserved1", 3usize, |e| -> _ {
                Encodable::encode(&*_reserved1, e)
            })?;
            e.emit_struct_field("cpu_svn", 4usize, |e| -> _ {
                Encodable::encode(&*_cpu_svn, e)
            })?;
            e.emit_struct_field("attribute_mask", 5usize, |e| -> _ {
                Encodable::encode(&*_attribute_mask, e)
            })?;
            e.emit_struct_field("key_id", 6usize, |e| -> _ {
                Encodable::encode(&*_key_id, e)
            })?;
            e.emit_struct_field("misc_mask", 7usize, |e| -> _ {
                Encodable::encode(&*_misc_mask, e)
            })?;
            e.emit_struct_field("config_svn", 8usize, |e| -> _ {
                Encodable::encode(&*_config_svn, e)
            })?;
            e.emit_struct_field("reserved2", 9usize, |e| -> _ {
                Encodable::encode(&*_reserved2, e)
            })
        })
    }
}

impl Decodable for KeyRequest {
    fn decode<D: Decoder>(d: &mut D) -> Result<KeyRequest, D::Error> {
        d.read_struct("KeyRequest", 10usize, |d| -> _ {
            Ok(KeyRequest {
                key_name: d.read_struct_field("key_name", 0usize, Decodable::decode)?,
                key_policy: d.read_struct_field("key_policy", 1usize, Decodable::decode)?,
                isv_svn: d.read_struct_field("isv_svn", 2usize, Decodable::decode)?,
                reserved1: d.read_struct_field("reserved1", 3usize, Decodable::decode)?,
                cpu_svn: d.read_struct_field("cpu_svn", 4usize, Decodable::decode)?,
                attribute_mask: d.read_struct_field("attribute_mask", 5usize, Decodable::decode)?,
                key_id: d.read_struct_field("key_id", 6usize, Decodable::decode)?,
                misc_mask: d.read_struct_field("misc_mask", 7usize, Decodable::decode)?,
                config_svn: d.read_struct_field("config_svn", 8usize, Decodable::decode)?,
                reserved2: d.read_struct_field("reserved2", 9usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Spid {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Spid { id: ref _id } = *self;
        e.emit_struct("Spid", 1usize, |e| -> _ {
            e.emit_struct_field("id", 0usize, |e| -> _ { Encodable::encode(&*_id, e) })
        })
    }
}

impl Decodable for Spid {
    fn decode<D: Decoder>(d: &mut D) -> Result<Spid, D::Error> {
        d.read_struct("Spid", 1usize, |d| -> _ {
            Ok(Spid {
                id: d.read_struct_field("id", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for BaseName {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let BaseName { name: ref _name } = *self;
        e.emit_struct("BaseName", 1usize, |e| -> _ {
            e.emit_struct_field("name", 0usize, |e| -> _ { Encodable::encode(&*_name, e) })
        })
    }
}

impl Decodable for BaseName {
    fn decode<D: Decoder>(d: &mut D) -> Result<BaseName, D::Error> {
        d.read_struct("BaseName", 1usize, |d| -> _ {
            Ok(BaseName {
                name: d.read_struct_field("name", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for QuoteNonce {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let QuoteNonce { rand: ref _rand } = *self;
        e.emit_struct("QuoteNonce", 1usize, |e| -> _ {
            e.emit_struct_field("rand", 0usize, |e| -> _ { Encodable::encode(&*_rand, e) })
        })
    }
}

impl Decodable for QuoteNonce {
    fn decode<D: Decoder>(d: &mut D) -> Result<QuoteNonce, D::Error> {
        d.read_struct("QuoteNonce", 1usize, |d| -> _ {
            Ok(QuoteNonce {
                rand: d.read_struct_field("rand", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for PsSecPropDesc {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let PsSecPropDesc {
            ps_sec_prop_desc: ref _ps_sec_prop_desc,
        } = *self;
        e.emit_struct("PsSecPropDesc", 1usize, |e| -> _ {
            e.emit_struct_field("ps_sec_prop_desc", 0usize, |e| -> _ {
                Encodable::encode(&*_ps_sec_prop_desc, e)
            })
        })
    }
}

impl Decodable for PsSecPropDesc {
    fn decode<D: Decoder>(d: &mut D) -> Result<PsSecPropDesc, D::Error> {
        d.read_struct("PsSecPropDesc", 1usize, |d| -> _ {
            Ok(PsSecPropDesc {
                ps_sec_prop_desc: d.read_struct_field(
                    "ps_sec_prop_desc",
                    0usize,
                    Decodable::decode,
                )?,
            })
        })
    }
}

impl Encodable for EnclaveIdentity {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let EnclaveIdentity {
            cpu_svn: ref _cpu_svn,
            attributes: ref _attributes,
            mr_enclave: ref _mr_enclave,
            mr_signer: ref _mr_signer,
            misc_select: ref _misc_select,
            isv_prod_id: ref _isv_prod_id,
            isv_svn: ref _isv_svn,
        } = *self;
        e.emit_struct("EnclaveIdentity", 7usize, |e| -> _ {
            e.emit_struct_field("cpu_svn", 0usize, |e| -> _ {
                Encodable::encode(&*_cpu_svn, e)
            })?;
            e.emit_struct_field("attributes", 1usize, |e| -> _ {
                Encodable::encode(&*_attributes, e)
            })?;
            e.emit_struct_field("mr_enclave", 2usize, |e| -> _ {
                Encodable::encode(&*_mr_enclave, e)
            })?;
            e.emit_struct_field("mr_signer", 3usize, |e| -> _ {
                Encodable::encode(&*_mr_signer, e)
            })?;
            e.emit_struct_field("misc_select", 4usize, |e| -> _ {
                Encodable::encode(&*_misc_select, e)
            })?;
            e.emit_struct_field("isv_prod_id", 5usize, |e| -> _ {
                Encodable::encode(&*_isv_prod_id, e)
            })?;
            e.emit_struct_field("isv_svn", 6usize, |e| -> _ {
                Encodable::encode(&*_isv_svn, e)
            })
        })
    }
}

impl Decodable for EnclaveIdentity {
    fn decode<D: Decoder>(d: &mut D) -> Result<EnclaveIdentity, D::Error> {
        d.read_struct("EnclaveIdentity", 7usize, |d| -> _ {
            Ok(EnclaveIdentity {
                cpu_svn: d.read_struct_field("cpu_svn", 0usize, Decodable::decode)?,
                attributes: d.read_struct_field("attributes", 1usize, Decodable::decode)?,
                mr_enclave: d.read_struct_field("mr_enclave", 2usize, Decodable::decode)?,
                mr_signer: d.read_struct_field("mr_signer", 3usize, Decodable::decode)?,
                misc_select: d.read_struct_field("misc_select", 4usize, Decodable::decode)?,
                isv_prod_id: d.read_struct_field("isv_prod_id", 5usize, Decodable::decode)?,
                isv_svn: d.read_struct_field("isv_svn", 6usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for TeeTcbSvn {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let TeeTcbSvn {
            tcb_svn: ref _tcb_svn,
        } = *self;
        e.emit_struct("TeeTcbSvn", 1usize, |e| -> _ {
            e.emit_struct_field("tcb_svn", 0usize, |e| -> _ {
                Encodable::encode(&*_tcb_svn, e)
            })
        })
    }
}

impl Decodable for TeeTcbSvn {
    fn decode<D: Decoder>(d: &mut D) -> Result<TeeTcbSvn, D::Error> {
        d.read_struct("TeeTcbSvn", 1usize, |d| -> _ {
            Ok(TeeTcbSvn {
                tcb_svn: d.read_struct_field("tcb_svn", 0usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for TeeInfo {
    #[allow(unaligned_references)]
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let TeeInfo {
            attributes: ref _attributes,
            xfam: ref _xfam,
            mr_td: ref _mr_td,
            mr_config_id: ref _mr_config_id,
            mr_owner: ref _mr_owner,
            mr_owner_config: ref _mr_owner_config,
            rt_mr: ref _rt_mr,
            reserved: ref _reserved,
        } = *self;
        e.emit_struct("TeeInfo", 8usize, |e| -> _ {
            e.emit_struct_field("attributes", 0usize, |e| -> _ {
                Encodable::encode(&*_attributes, e)
            })?;
            e.emit_struct_field("xfam", 1usize, |e| -> _ { Encodable::encode(&*_xfam, e) })?;
            e.emit_struct_field("mr_td", 2usize, |e| -> _ { Encodable::encode(&*_mr_td, e) })?;
            e.emit_struct_field("mr_config_id", 3usize, |e| -> _ {
                Encodable::encode(&*_mr_config_id, e)
            })?;
            e.emit_struct_field("mr_owner", 4usize, |e| -> _ {
                Encodable::encode(&*_mr_owner, e)
            })?;
            e.emit_struct_field("mr_owner_config", 5usize, |e| -> _ {
                Encodable::encode(&*_mr_owner_config, e)
            })?;
            e.emit_struct_field("rt_mr", 6usize, |e| -> _ { Encodable::encode(&*_rt_mr, e) })?;
            e.emit_struct_field("reserved", 7usize, |e| -> _ {
                Encodable::encode(&*_reserved, e)
            })
        })
    }
}

impl Decodable for TeeInfo {
    fn decode<D: Decoder>(d: &mut D) -> Result<TeeInfo, D::Error> {
        d.read_struct("TeeInfo", 8usize, |d| -> _ {
            Ok(TeeInfo {
                attributes: d.read_struct_field("attributes", 0usize, Decodable::decode)?,
                xfam: d.read_struct_field("xfam", 1usize, Decodable::decode)?,
                mr_td: d.read_struct_field("mr_td", 2usize, Decodable::decode)?,
                mr_config_id: d.read_struct_field("mr_config_id", 3usize, Decodable::decode)?,
                mr_owner: d.read_struct_field("mr_owner", 4usize, Decodable::decode)?,
                mr_owner_config: d.read_struct_field(
                    "mr_owner_config",
                    5usize,
                    Decodable::decode,
                )?,
                rt_mr: d.read_struct_field("rt_mr", 6usize, Decodable::decode)?,
                reserved: d.read_struct_field("reserved", 7usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for TeeTcbInfo {
    #[allow(unaligned_references)]
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let TeeTcbInfo {
            valid: ref _valid,
            tee_tcb_svn: ref _tee_tcb_svn,
            mr_seam: ref _mr_seam,
            mr_seam_signer: ref _mr_seam_signer,
            attributes: ref _attributes,
            reserved: ref _reserved,
        } = *self;
        e.emit_struct("TeeTcbInfo", 6usize, |e| -> _ {
            e.emit_struct_field("valid", 0usize, |e| -> _ { Encodable::encode(&*_valid, e) })?;
            e.emit_struct_field("tee_tcb_svn", 1usize, |e| -> _ {
                Encodable::encode(&*_tee_tcb_svn, e)
            })?;
            e.emit_struct_field("mr_seam", 2usize, |e| -> _ {
                Encodable::encode(&*_mr_seam, e)
            })?;
            e.emit_struct_field("mr_seam_signer", 3usize, |e| -> _ {
                Encodable::encode(&*_mr_seam_signer, e)
            })?;
            e.emit_struct_field("attributes", 4usize, |e| -> _ {
                Encodable::encode(&*_attributes, e)
            })?;
            e.emit_struct_field("reserved", 5usize, |e| -> _ {
                Encodable::encode(&*_reserved, e)
            })
        })
    }
}

impl Decodable for TeeTcbInfo {
    fn decode<D: Decoder>(d: &mut D) -> Result<TeeTcbInfo, D::Error> {
        d.read_struct("TeeTcbInfo", 8usize, |d| -> _ {
            Ok(TeeTcbInfo {
                valid: d.read_struct_field("valid", 0usize, Decodable::decode)?,
                tee_tcb_svn: d.read_struct_field("tee_tcb_svn", 1usize, Decodable::decode)?,
                mr_seam: d.read_struct_field("mr_seam", 2usize, Decodable::decode)?,
                mr_seam_signer: d.read_struct_field("mr_seam_signer", 3usize, Decodable::decode)?,
                attributes: d.read_struct_field("attributes", 4usize, Decodable::decode)?,
                reserved: d.read_struct_field("reserved", 5usize, Decodable::decode)?,
            })
        })
    }
}

impl Encodable for Report2Body {
    #[allow(unaligned_references)]
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        let Report2Body {
            tee_tcb_svn: ref _tee_tcb_svn,
            mr_seam: ref _mr_seam,
            mrsigner_seam: ref _mrsigner_seam,
            seam_attributes: ref _seam_attributes,
            td_attributes: ref _td_attributes,
            xfam: ref _xfam,
            mr_td: ref _mr_td,
            mr_config_id: ref _mr_config_id,
            mr_owner: ref _mr_owner,
            mr_owner_config: ref _mr_owner_config,
            rt_mr: ref _rt_mr,
            report_data: ref _report_data,
        } = *self;
        e.emit_struct("Report2Body", 12usize, |e| -> _ {
            e.emit_struct_field("tee_tcb_svn", 0usize, |e| -> _ {
                Encodable::encode(&*_tee_tcb_svn, e)
            })?;
            e.emit_struct_field("mr_seam", 1usize, |e| -> _ {
                Encodable::encode(&*_mr_seam, e)
            })?;
            e.emit_struct_field("mrsigner_seam", 2usize, |e| -> _ {
                Encodable::encode(&*_mrsigner_seam, e)
            })?;
            e.emit_struct_field("seam_attributes", 3usize, |e| -> _ {
                Encodable::encode(&*_seam_attributes, e)
            })?;
            e.emit_struct_field("td_attributes", 4usize, |e| -> _ {
                Encodable::encode(&*_td_attributes, e)
            })?;
            e.emit_struct_field("xfam", 5usize, |e| -> _ { Encodable::encode(&*_xfam, e) })?;
            e.emit_struct_field("mr_td", 6usize, |e| -> _ { Encodable::encode(&*_mr_td, e) })?;
            e.emit_struct_field("mr_config_id", 7usize, |e| -> _ {
                Encodable::encode(&*_mr_config_id, e)
            })?;
            e.emit_struct_field("mr_owner", 8usize, |e| -> _ {
                Encodable::encode(&*_mr_owner, e)
            })?;
            e.emit_struct_field("mr_owner_config", 9usize, |e| -> _ {
                Encodable::encode(&*_mr_owner_config, e)
            })?;
            e.emit_struct_field("rt_mr", 10usize, |e| -> _ {
                Encodable::encode(&*_rt_mr, e)
            })?;
            e.emit_struct_field("report_data", 11usize, |e| -> _ {
                Encodable::encode(&*_report_data, e)
            })
        })
    }
}

impl Decodable for Report2Body {
    fn decode<D: Decoder>(d: &mut D) -> Result<Report2Body, D::Error> {
        d.read_struct("Report2Body", 8usize, |d| -> _ {
            Ok(Report2Body {
                tee_tcb_svn: d.read_struct_field("tee_tcb_svn", 0usize, Decodable::decode)?,
                mr_seam: d.read_struct_field("mr_seam", 1usize, Decodable::decode)?,
                mrsigner_seam: d.read_struct_field("mrsigner_seam", 2usize, Decodable::decode)?,
                seam_attributes: d.read_struct_field(
                    "seam_attributes",
                    3usize,
                    Decodable::decode,
                )?,
                td_attributes: d.read_struct_field("td_attributes", 4usize, Decodable::decode)?,
                xfam: d.read_struct_field("xfam", 5usize, Decodable::decode)?,
                mr_td: d.read_struct_field("mr_td", 6usize, Decodable::decode)?,
                mr_config_id: d.read_struct_field("mr_config_id", 7usize, Decodable::decode)?,
                mr_owner: d.read_struct_field("mr_owner", 8usize, Decodable::decode)?,
                mr_owner_config: d.read_struct_field(
                    "mr_owner_config",
                    9usize,
                    Decodable::decode,
                )?,
                rt_mr: d.read_struct_field("rt_mr", 10usize, Decodable::decode)?,
                report_data: d.read_struct_field("report_data", 11usize, Decodable::decode)?,
            })
        })
    }
}
