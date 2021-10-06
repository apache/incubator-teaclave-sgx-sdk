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

use sgx_alloc::alignbox::*;
use sgx_types::*;
use std::mem;

impl_struct! {
    pub struct user_struct {
        pub a: u8,
        pub b: [u16; 2],
        pub c: u32,
        pub d: [u64; 6],
    }
}

impl_struct! {
    pub struct struct_align_128_t {
        pub key: sgx_key_128bit_t,
    }
}

impl_struct! {
    pub struct struct_no_align_t {
        pub key1: sgx_key_128bit_t,
        pub key2: sgx_key_128bit_t,
        pub key3: sgx_key_128bit_t,
        pub key4: sgx_key_128bit_t,
    }
}

impl_struct! {
    #[derive(PartialEq, Debug)]
    pub struct struct_align_t{
        pub key1: sgx_key_128bit_t,
        pub pad1: [u8; 16],
        pub key2: sgx_key_128bit_t ,
        pub pad2: [u8; 16],
        pub key3: sgx_key_128bit_t,
        pub pad3: [u8; 16],
        pub key4: sgx_key_128bit_t,
    }
}

pub fn test_alignbox() {
    //align
    {
        let aligned_box: Option<AlignBox<struct_align_128_t>> =
            AlignBox::<struct_align_128_t>::new();
        assert!(aligned_box.is_some());
    }
    //no align
    {
        let str_slice: &[AlignReq] = &[];
        let aligned_box: Option<AlignBox<user_struct>> =
            AlignBox::<user_struct>::new_with_req(1, &str_slice);
        assert!(aligned_box.is_none());
    }
    //align
    {
        let str_slice: &[AlignReq] = &[
            AlignReq {
                offset: offset_of!(user_struct, a),
                len: mem::size_of::<u8>(),
            },
            AlignReq {
                offset: offset_of!(user_struct, b),
                len: mem::size_of::<[u16; 2]>(),
            },
            AlignReq {
                offset: offset_of!(user_struct, d),
                len: mem::size_of::<[u64; 6]>(),
            },
        ];
        let aligned_box: Option<AlignBox<user_struct>> =
            AlignBox::<user_struct>::new_with_req(1, &str_slice);
        assert!(aligned_box.is_some());
    }
    //no align
    {
        let str_slice: &[AlignReq] = &[
            AlignReq {
                offset: offset_of!(user_struct, a),
                len: mem::size_of::<u8>(),
            },
            AlignReq {
                offset: offset_of!(user_struct, b),
                len: mem::size_of::<[u16; 2]>(),
            },
            AlignReq {
                offset: offset_of!(user_struct, d),
                len: mem::size_of::<[u64; 6]>(),
            },
        ];
        let aligned_box: Option<AlignBox<user_struct>> =
            AlignBox::<user_struct>::new_with_req(16, &str_slice);
        assert!(aligned_box.is_none());
    }
    //no align
    {
        let str_slice: &[AlignReq] = &[
            AlignReq {
                offset: offset_of!(struct_align_t, key1),
                len: mem::size_of::<sgx_key_128bit_t>(),
            },
            AlignReq {
                offset: offset_of!(struct_align_t, key2),
                len: mem::size_of::<sgx_key_128bit_t>(),
            },
            AlignReq {
                offset: offset_of!(struct_align_t, key3),
                len: mem::size_of::<sgx_key_128bit_t>(),
            },
            AlignReq {
                offset: offset_of!(struct_align_t, key4),
                len: mem::size_of::<sgx_key_128bit_t>(),
            },
        ];
        let aligned_box: Option<AlignBox<struct_align_t>> =
            AlignBox::<struct_align_t>::new_with_req(32, &str_slice);
        assert!(aligned_box.is_none());
    }
    //align
    {
        let str_slice: &[AlignReq] = &[
            AlignReq {
                offset: offset_of!(struct_align_t, key1),
                len: mem::size_of::<sgx_key_128bit_t>(),
            },
            AlignReq {
                offset: offset_of!(struct_align_t, key2),
                len: mem::size_of::<sgx_key_128bit_t>(),
            },
            AlignReq {
                offset: offset_of!(struct_align_t, key3),
                len: mem::size_of::<sgx_key_128bit_t>(),
            },
            AlignReq {
                offset: offset_of!(struct_align_t, key4),
                len: mem::size_of::<sgx_key_128bit_t>(),
            },
        ];
        let aligned_box: Option<AlignBox<struct_align_t>> =
            AlignBox::<struct_align_t>::new_with_req(16, &str_slice);
        assert!(aligned_box.is_some());
    }
}

pub fn test_alignbox_heap_init() {
    let str_slice: &[AlignReq] = &[
        AlignReq {
            offset: offset_of!(struct_align_t, key1),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
        AlignReq {
            offset: offset_of!(struct_align_t, key2),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
        AlignReq {
            offset: offset_of!(struct_align_t, key3),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
        AlignReq {
            offset: offset_of!(struct_align_t, key4),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
    ];

    let stack_align_obj = struct_align_t {
        key1: [
            0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
            0xfe, 0xff,
        ],
        pad1: [0x00; 16],
        key2: [
            0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
            0xfe, 0xff,
        ],
        pad2: [0x00; 16],
        key3: [
            0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
            0xfe, 0xff,
        ],
        pad3: [0x00; 16],
        key4: [
            0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
            0xfe, 0xff,
        ],
    };
    let heap_align_obj = AlignBox::<struct_align_t>::heap_init_with_req(
        |mut t| {
            t.key1 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
            t.pad1 = [0x00; 16];
            t.key2 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
            t.pad2 = [0x00; 16];
            t.key3 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
            t.pad3 = [0x00; 16];
            t.key4 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
        },
        16,
        &str_slice,
    );
    assert!(heap_align_obj.is_some());
    assert_eq!(stack_align_obj, *(heap_align_obj.unwrap()));
}

pub fn test_alignbox_clone() {
    let str_slice: &[AlignReq] = &[
        AlignReq {
            offset: offset_of!(struct_align_t, key1),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
        AlignReq {
            offset: offset_of!(struct_align_t, key2),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
        AlignReq {
            offset: offset_of!(struct_align_t, key3),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
        AlignReq {
            offset: offset_of!(struct_align_t, key4),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
    ];
    let heap_align_obj = AlignBox::<struct_align_t>::heap_init_with_req(
        |mut t| {
            t.key1 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
            t.pad1 = [0x00; 16];
            t.key2 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
            t.pad2 = [0x00; 16];
            t.key3 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
            t.pad3 = [0x00; 16];
            t.key4 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
        },
        16,
        &str_slice,
    );
    assert!(heap_align_obj.is_some());
    let align_box_clone = heap_align_obj.clone();
    assert_eq!(*align_box_clone.unwrap(), *heap_align_obj.unwrap());
}

pub fn test_alignbox_clonefrom() {
    let str_slice: &[AlignReq] = &[
        AlignReq {
            offset: offset_of!(struct_align_t, key1),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
        AlignReq {
            offset: offset_of!(struct_align_t, key2),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
        AlignReq {
            offset: offset_of!(struct_align_t, key3),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
        AlignReq {
            offset: offset_of!(struct_align_t, key4),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
    ];
    let heap_align_obj = AlignBox::<struct_align_t>::heap_init_with_req(
        |mut t| {
            t.key1 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
            t.pad1 = [0x00; 16];
            t.key2 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
            t.pad2 = [0x00; 16];
            t.key3 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
            t.pad3 = [0x00; 16];
            t.key4 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
        },
        16,
        &str_slice,
    );
    assert!(heap_align_obj.is_some());

    let mut heap_align_zero_obj = AlignBox::<struct_align_t>::new_with_req(16, &str_slice);
    assert!(heap_align_zero_obj.is_some());
    heap_align_zero_obj.clone_from(&heap_align_obj);
    assert_eq!(*(heap_align_zero_obj.unwrap()), *(heap_align_obj.unwrap()));
}

pub fn test_alignbox_clonefrom_no_eq_size() {
    let str_slice: &[AlignReq] = &[
        AlignReq {
            offset: offset_of!(struct_align_t, key1),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
        AlignReq {
            offset: offset_of!(struct_align_t, key2),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
        AlignReq {
            offset: offset_of!(struct_align_t, key3),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
        AlignReq {
            offset: offset_of!(struct_align_t, key4),
            len: mem::size_of::<sgx_key_128bit_t>(),
        },
    ];
    let heap_align_obj = AlignBox::<struct_align_t>::heap_init_with_req(
        |mut t| {
            t.key1 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
            t.pad1 = [0x00; 16];
            t.key2 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
            t.pad2 = [0x00; 16];
            t.key3 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
            t.pad3 = [0x00; 16];
            t.key4 = [
                0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd,
                0xfe, 0xff,
            ];
        },
        16,
        &str_slice,
    );
    assert!(heap_align_obj.is_some());

    let mut heap_align_zero_obj = AlignBox::<struct_align_t>::new_with_req(1, &str_slice);
    assert!(heap_align_zero_obj.is_some());
    heap_align_zero_obj.clone_from(&heap_align_obj);
    assert_eq!(*(heap_align_zero_obj.unwrap()), *(heap_align_obj.unwrap()));
}
