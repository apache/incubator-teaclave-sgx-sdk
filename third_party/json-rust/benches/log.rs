// #![feature(custom_derive)]
// #![feature(custom_attribute)]
#![feature(test)]
// #![feature(plugin)]
// #![plugin(serde_macros)]

// for the Country enum
// #![recursion_limit="259"]

extern crate json;
extern crate test;
// extern crate serde;
// extern crate serde_json;
// extern crate rustc_serialize;
// extern crate num_traits;

#[macro_use]
// mod macros;

use test::Bencher;

// #[derive(Debug, PartialEq, RustcEncodable, RustcDecodable, Serialize, Deserialize)]
// struct Http {
//     protocol: HttpProtocol,
//     status: u32,
//     host_status: u32,
//     up_status: u32,
//     method: HttpMethod,
//     content_type: String,
//     user_agent: String,
//     referer: String,
//     request_uri: String,
// }

// c_enum!(HttpProtocol {
//     HTTP_PROTOCOL_UNKNOWN,
//     HTTP10,
//     HTTP11,
// });

// c_enum!(HttpMethod {
//     METHOD_UNKNOWN,
//     GET,
//     POST,
//     DELETE,
//     PUT,
//     HEAD,
//     PURGE,
//     OPTIONS,
//     PROPFIND,
//     MKCOL,
//     PATCH,
// });

// c_enum!(CacheStatus {
//     CACHESTATUS_UNKNOWN,
//     Miss,
//     Expired,
//     Hit,
// });

// #[derive(Debug, PartialEq, RustcEncodable, RustcDecodable, Serialize, Deserialize)]
// struct Origin {
//     ip: String,
//     port: u32,
//     hostname: String,
//     protocol: OriginProtocol,
// }

// c_enum!(OriginProtocol {
//     ORIGIN_PROTOCOL_UNKNOWN,
//     HTTP,
//     HTTPS,
// });

// c_enum!(ZonePlan {
//     ZONEPLAN_UNKNOWN,
//     FREE,
//     PRO,
//     BIZ,
//     ENT,
// });

// c_enum!(Country {
//     UNKNOWN,
//     A1, A2, O1, AD, AE, AF, AG, AI, AL, AM, AO, AP, AQ, AR, AS, AT, AU, AW, AX,
//     AZ, BA, BB, BD, BE, BF, BG, BH, BI, BJ, BL, BM, BN, BO, BQ, BR, BS, BT, BV,
//     BW, BY, BZ, CA, CC, CD, CF, CG, CH, CI, CK, CL, CM, CN, CO, CR, CU, CV, CW,
//     CX, CY, CZ, DE, DJ, DK, DM, DO, DZ, EC, EE, EG, EH, ER, ES, ET, EU, FI, FJ,
//     FK, FM, FO, FR, GA, GB, GD, GE, GF, GG, GH, GI, GL, GM, GN, GP, GQ, GR, GS,
//     GT, GU, GW, GY, HK, HM, HN, HR, HT, HU, ID, IE, IL, IM, IN, IO, IQ, IR, IS,
//     IT, JE, JM, JO, JP, KE, KG, KH, KI, KM, KN, KP, KR, KW, KY, KZ, LA, LB, LC,
//     LI, LK, LR, LS, LT, LU, LV, LY, MA, MC, MD, ME, MF, MG, MH, MK, ML, MM, MN,
//     MO, MP, MQ, MR, MS, MT, MU, MV, MW, MX, MY, MZ, NA, NC, NE, NF, NG, NI, NL,
//     NO, NP, NR, NU, NZ, OM, PA, PE, PF, PG, PH, PK, PL, PM, PN, PR, PS, PT, PW,
//     PY, QA, RE, RO, RS, RU, RW, SA, SB, SC, SD, SE, SG, SH, SI, SJ, SK, SL, SM,
//     SN, SO, SR, SS, ST, SV, SX, SY, SZ, TC, TD, TF, TG, TH, TJ, TK, TL, TM, TN,
//     TO, TR, TT, TV, TW, TZ, UA, UG, UM, US, UY, UZ, VA, VC, VE, VG, VI, VN, VU,
//     WF, WS, XX, YE, YT, ZA, ZM, ZW,
// });

// #[derive(Debug, PartialEq, RustcEncodable, RustcDecodable, Serialize, Deserialize)]
// struct Log {
//     timestamp: i64,
//     zone_id: u32,
//     zone_plan: ZonePlan,
//     http: Http,
//     origin: Origin,
//     country: Country,
//     cache_status: CacheStatus,
//     server_ip: String,
//     server_name: String,
//     remote_ip: String,
//     bytes_dlv: u64,
//     ray_id: String,
// }

const JSON_STR: &'static str = r#"{"timestamp":2837513946597,"zone_id":123456,"zone_plan":1,"http":{"protocol":2,"status":200,"host_status":503,"up_status":520,"method":1,"content_type":"text/html","user_agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/33.0.1750.146 Safari/537.36","referer":"https://www.cloudflare.com/","request_uri":"/cdn-cgi/trace"},"origin":{"ip":"1.2.3.4","port":8000,"hostname":"www.example.com","protocol":2},"country":238,"cache_status":3,"server_ip":"192.168.1.1","server_name":"metal.cloudflare.com","remote_ip":"10.1.2.3","bytes_dlv":123456,"ray_id":"10c73629cce30078-LAX"}"#;

const JSON_FLOAT_STR: &'static str = r#"[[-65.613616999999977,43.420273000000009],[-65.619720000000029,43.418052999999986],[-65.625,43.421379000000059],[-65.636123999999882,43.449714999999969],[-65.633056999999951,43.474709000000132],[-65.611389000000031,43.513054000000068],[-65.605835000000013,43.516105999999979],[-65.598343,43.515830999999935],[-65.566101000000003,43.508331000000055],[-65.561935000000005,43.504439999999988],[-65.55999799999995,43.499718000000087],[-65.573333999999988,43.476379000000065],[-65.593612999999948,43.444153000000028],[-65.613616999999977,43.420273000000009],[-59.816947999999911,43.928328999999962],[-59.841667000000029,43.918602000000021],[-59.866393999999957,43.909987999999998],[-59.879722999999956,43.906654000000003],[-59.895835999999974,43.904160000000047]]"#;

// #[bench]
// fn rustc_serialize_parse(b: &mut Bencher) {
//     b.bytes = JSON_STR.len() as u64;

//     b.iter(|| {
//         rustc_serialize::json::Json::from_str(JSON_STR).unwrap()
//     });
// }

// #[bench]
// fn rustc_serialize_stringify(b: &mut Bencher) {
//     let data = rustc_serialize::json::Json::from_str(JSON_STR).unwrap();

//     b.bytes = rustc_serialize::json::encode(&data).unwrap().len() as u64;

//     b.iter(|| {
//         rustc_serialize::json::encode(&data).unwrap();
//     })
// }

// #[bench]
// fn rustc_serialize_struct_parse(b: &mut Bencher) {
//     use rustc_serialize::json::Json;

//     b.bytes = JSON_STR.len() as u64;

//     b.iter(|| {
//         let json = Json::from_str(JSON_STR).unwrap();
//         let mut decoder = rustc_serialize::json::Decoder::new(json);
//         let log: Log = rustc_serialize::Decodable::decode(&mut decoder).unwrap();
//         log
//     });
// }

// #[bench]
// fn rustc_serialize_struct_stringify(b: &mut Bencher) {
//     use rustc_serialize::json::Json;

//     b.bytes = JSON_STR.len() as u64;

//     let json = Json::from_str(JSON_STR).unwrap();
//     let mut decoder = rustc_serialize::json::Decoder::new(json);
//     let log: Log = rustc_serialize::Decodable::decode(&mut decoder).unwrap();

//     b.iter(|| {
//         rustc_serialize::json::encode(&log).unwrap();
//     })
// }

// #[bench]
// fn serde_json_parse(b: &mut Bencher) {
//     b.bytes = JSON_STR.len() as u64;

//     b.iter(|| {
//         serde_json::from_str::<serde_json::Value>(JSON_STR).unwrap();
//     });
// }

// #[bench]
// fn serde_json_stringify(b: &mut Bencher) {
//     let data = serde_json::from_str::<serde_json::Value>(JSON_STR).unwrap();

//     b.bytes = serde_json::to_string(&data).unwrap().len() as u64;

//     b.iter(|| {
//         serde_json::to_string(&data).unwrap();
//     })
// }

// #[bench]
// fn serde_json_floats_parse(b: &mut Bencher) {
//     b.bytes = JSON_FLOAT_STR.len() as u64;

//     b.iter(|| {
//         serde_json::from_str::<serde_json::Value>(JSON_FLOAT_STR).unwrap();
//     });
// }

// #[bench]
// fn serde_json_floats_stringify(b: &mut Bencher) {
//     let data = serde_json::from_str::<serde_json::Value>(JSON_FLOAT_STR).unwrap();

//     b.bytes = serde_json::to_string(&data).unwrap().len() as u64;

//     b.iter(|| {
//         serde_json::to_string(&data).unwrap();
//     })
// }

// #[bench]
// fn serde_json_struct_parse(b: &mut Bencher) {
//     b.bytes = JSON_STR.len() as u64;

//     b.iter(|| {
//         serde_json::from_str::<Log>(JSON_STR).unwrap();
//     });
// }

// #[bench]
// fn serde_json_struct_stringify(b: &mut Bencher) {
//     b.bytes = JSON_STR.len() as u64;

//     let data = serde_json::from_str::<Log>(JSON_STR).unwrap();

//     b.iter(|| {
//         serde_json::to_string(&data).unwrap();
//     });
// }

#[bench]
fn json_rust_parse(b: &mut Bencher) {
    b.bytes = JSON_STR.len() as u64;

    b.iter(|| {
        json::parse(JSON_STR).unwrap();
    });
}

#[bench]
fn json_rust_parse_floats(b: &mut Bencher) {
    b.bytes = JSON_FLOAT_STR.len() as u64;

    b.iter(|| {
        json::parse(JSON_FLOAT_STR).unwrap();
    });
}

#[bench]
fn json_rust_stringify(b: &mut Bencher) {
    let data = json::parse(JSON_STR).unwrap();

    b.bytes = data.dump().len() as u64;

    b.iter(|| {
        data.dump();
    })
}

#[bench]
fn json_rust_stringify_io_write(b: &mut Bencher) {
    let data = json::parse(JSON_STR).unwrap();

    b.bytes = data.dump().len() as u64;

    let mut target = Vec::new();

    b.iter(|| {
        data.to_writer(&mut target);
    })
}

#[bench]
fn json_rust_stringify_floats(b: &mut Bencher) {
    let data = json::parse(JSON_FLOAT_STR).unwrap();

    b.bytes = data.dump().len() as u64;

    b.iter(|| {
        data.dump();
    })
}

#[bench]
fn json_rust_stringify_floats_io_write(b: &mut Bencher) {
    let data = json::parse(JSON_FLOAT_STR).unwrap();

    b.bytes = data.dump().len() as u64;

    let mut target = Vec::new();

    b.iter(|| {
        data.to_writer(&mut target);
    })
}
