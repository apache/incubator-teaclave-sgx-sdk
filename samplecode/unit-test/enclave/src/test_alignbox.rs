// Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::mem;
use sgx_tse::alignbox::*;
use sgx_types::*;

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
        let aligned_box: Option<AlignBox<struct_align_128_t>> = AlignBox::<struct_align_128_t>::new();
        assert_eq!(aligned_box.is_some(), true);
    }
    //no align
    {
        let str_slice: &[align_req_t] = &[];
        let aligned_box: Option<AlignBox<user_struct>> = AlignBox::<user_struct>::new_with_req(1, &str_slice);
        assert_eq!(aligned_box.is_none(), true);
    }
    //align
    {
        let str_slice: &[align_req_t] = &[
            align_req_t {
                offset:offset_of!(user_struct, a),
                len:mem::size_of::<u8>(),
            },
            align_req_t {
                offset:offset_of!(user_struct, b),
                len:mem::size_of::<[u16; 2]>(),
            },
            align_req_t {
                offset: offset_of!(user_struct, d),
                len:mem::size_of::<[u64; 6]>(),
            }
        ];
        let aligned_box: Option<AlignBox<user_struct>> = AlignBox::<user_struct>::new_with_req(1, &str_slice);
        assert_eq!(aligned_box.is_some(), true);
    }
    //no align
    {
        let str_slice: &[align_req_t] = &[
            align_req_t {
                offset:offset_of!(user_struct, a),
                len:mem::size_of::<u8>(),
            },
            align_req_t {
                offset:offset_of!(user_struct, b),
                len:mem::size_of::<[u16; 2]>(),
            },
            align_req_t {
                offset: offset_of!(user_struct, d),
                len:mem::size_of::<[u64; 6]>(),
            }
        ];
        let aligned_box: Option<AlignBox<user_struct>>  = AlignBox::<user_struct>::new_with_req(16, &str_slice);
        assert_eq!(aligned_box.is_none(), true);
    }
    //no align
    {
        let str_slice: &[align_req_t] = &[
            align_req_t {
                offset:offset_of!(struct_align_t, key1),
                len:mem::size_of::<sgx_key_128bit_t>(),
            },
            align_req_t {
                offset: offset_of!(struct_align_t, key2),
                len:mem::size_of::<sgx_key_128bit_t>(),
            },
            align_req_t {
                offset:offset_of!(struct_align_t, key3),
                len:mem::size_of::<sgx_key_128bit_t>(),
            },
            align_req_t {
                offset: offset_of!(struct_align_t, key4),
                len:mem::size_of::<sgx_key_128bit_t>(),
            }
        ];
        let aligned_box: Option<AlignBox<struct_align_t>>  = AlignBox::<struct_align_t>::new_with_req(32, &str_slice);
        assert_eq!(aligned_box.is_none(), true);
    }
    //align
    {
        let str_slice: &[align_req_t] = &[
            align_req_t {
                offset:offset_of!(struct_align_t, key1),
                len:mem::size_of::<sgx_key_128bit_t>(),
            },
            align_req_t {
                offset: offset_of!(struct_align_t, key2),
                len:mem::size_of::<sgx_key_128bit_t>(),
            },
            align_req_t {
                offset:offset_of!(struct_align_t, key3),
                len:mem::size_of::<sgx_key_128bit_t>(),
            },
            align_req_t {
                offset: offset_of!(struct_align_t, key4),
                len:mem::size_of::<sgx_key_128bit_t>(),
            }
        ];
        let aligned_box: Option<AlignBox<struct_align_t>>  = AlignBox::<struct_align_t>::new_with_req(16, &str_slice);
        assert_eq!(aligned_box.is_some(), true);
    }
}

pub fn test_alignbox_heap_init() {
    let str_slice: &[align_req_t] = &[
        align_req_t {
            offset:offset_of!(struct_align_t, key1),
            len:mem::size_of::<sgx_key_128bit_t>(),
        },
        align_req_t {
            offset: offset_of!(struct_align_t, key2),
            len:mem::size_of::<sgx_key_128bit_t>(),
        },
        align_req_t {
            offset:offset_of!(struct_align_t, key3),
            len:mem::size_of::<sgx_key_128bit_t>(),
        },
        align_req_t {
            offset: offset_of!(struct_align_t, key4),
            len:mem::size_of::<sgx_key_128bit_t>(),
        }
    ];

    let stack_align_obj = struct_align_t {
        key1: [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff],
        pad1: [0x00; 16],
        key2: [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff],
        pad2: [0x00; 16],
        key3: [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff],
        pad3: [0x00; 16],
        key4: [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff],
    };
    let heap_align_obj = AlignBox::<struct_align_t>::heap_init_with_req(|mut t| {
        t.key1 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        t.pad1 = [0x00; 16];
        t.key2 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        t.pad2 = [0x00; 16];
        t.key3 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        t.pad3 = [0x00; 16];
        t.key4 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
    }, 16, &str_slice);
    assert_eq!(heap_align_obj.is_some(), true);
    assert_eq!(stack_align_obj, *(heap_align_obj.unwrap()));
}

pub fn test_alignbox_clone() {
    let str_slice: &[align_req_t] = &[
        align_req_t {
            offset:offset_of!(struct_align_t, key1),
            len:mem::size_of::<sgx_key_128bit_t>(),
        },
        align_req_t {
            offset: offset_of!(struct_align_t, key2),
            len:mem::size_of::<sgx_key_128bit_t>(),
        },
        align_req_t {
            offset:offset_of!(struct_align_t, key3),
            len:mem::size_of::<sgx_key_128bit_t>(),
        },
        align_req_t {
            offset: offset_of!(struct_align_t, key4),
            len:mem::size_of::<sgx_key_128bit_t>(),
        }
    ];
    let heap_align_obj = AlignBox::<struct_align_t>::heap_init_with_req(|mut t| {
        t.key1 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        t.pad1 = [0x00; 16];
        t.key2 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        t.pad2 = [0x00; 16];
        t.key3 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        t.pad3 = [0x00; 16];
        t.key4 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
    }, 16, &str_slice);
    assert_eq!(heap_align_obj.is_some(), true);
    let align_box_clone = heap_align_obj.clone();
   assert_eq!(*align_box_clone.unwrap(), *heap_align_obj.unwrap());
}

pub fn test_alignbox_clonefrom() {
    let str_slice: &[align_req_t] = &[
        align_req_t {
            offset:offset_of!(struct_align_t, key1),
            len:mem::size_of::<sgx_key_128bit_t>(),
        },
        align_req_t {
            offset: offset_of!(struct_align_t, key2),
            len:mem::size_of::<sgx_key_128bit_t>(),
        },
        align_req_t {
            offset:offset_of!(struct_align_t, key3),
            len:mem::size_of::<sgx_key_128bit_t>(),
        },
        align_req_t {
            offset: offset_of!(struct_align_t, key4),
            len:mem::size_of::<sgx_key_128bit_t>(),
        }
    ];
    let heap_align_obj = AlignBox::<struct_align_t>::heap_init_with_req(|mut t| {
        t.key1 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        t.pad1 = [0x00; 16];
        t.key2 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        t.pad2 = [0x00; 16];
        t.key3 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        t.pad3 = [0x00; 16];
        t.key4 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
    }, 16, &str_slice);
    assert_eq!(heap_align_obj.is_some(), true);

    let mut heap_align_zero_obj = AlignBox::<struct_align_t>::new_with_req(16, &str_slice);
    assert_eq!(heap_align_zero_obj.is_some(), true);
    heap_align_zero_obj.clone_from(&heap_align_obj);
    assert_eq!(*(heap_align_zero_obj.unwrap()), *(heap_align_obj.unwrap()));
}

pub fn test_alignbox_clonefrom_no_eq_size() {
    let str_slice: &[align_req_t] = &[
        align_req_t {
            offset:offset_of!(struct_align_t, key1),
            len:mem::size_of::<sgx_key_128bit_t>(),
        },
        align_req_t {
            offset: offset_of!(struct_align_t, key2),
            len:mem::size_of::<sgx_key_128bit_t>(),
        },
        align_req_t {
            offset:offset_of!(struct_align_t, key3),
            len:mem::size_of::<sgx_key_128bit_t>(),
        },
        align_req_t {
            offset: offset_of!(struct_align_t, key4),
            len:mem::size_of::<sgx_key_128bit_t>(),
        }
    ];
    let heap_align_obj = AlignBox::<struct_align_t>::heap_init_with_req(|mut t| {
        t.key1 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        t.pad1 = [0x00; 16];
        t.key2 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        t.pad2 = [0x00; 16];
        t.key3 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
        t.pad3 = [0x00; 16];
        t.key4 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
    }, 16, &str_slice);
    assert_eq!(heap_align_obj.is_some(), true);

    let mut heap_align_zero_obj = AlignBox::<struct_align_t>::new_with_req(1, &str_slice);
    assert_eq!(heap_align_zero_obj.is_some(), true);
    heap_align_zero_obj.clone_from(&heap_align_obj);
    assert_eq!(*(heap_align_zero_obj.unwrap()), *(heap_align_obj.unwrap()));
}
