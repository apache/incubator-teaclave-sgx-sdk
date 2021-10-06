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

#![crate_name = "unittestsampleenclave"]
#![crate_type = "staticlib"]
#![feature(box_syntax)]
#![feature(core_intrinsics)]
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[cfg(target_env = "sgx")]
extern crate core;

extern crate sgx_types;
#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_tcrypto;
#[macro_use]
extern crate sgx_tunittest;
extern crate rand;
extern crate sgx_align_struct_attribute;
extern crate sgx_alloc;
extern crate sgx_rand;
extern crate sgx_trts;
extern crate sgx_tseal;
#[macro_use]
extern crate memoffset;
extern crate sgx_serialize;
pub use sgx_serialize::*;
#[macro_use]
extern crate sgx_serialize_derive;
extern crate sgx_libc;
extern crate sgx_signal;

pub use sgx_serialize::*;
use sgx_tunittest::*;
use sgx_types::*;

use std::string::String;
use std::vec::Vec;

mod utils;

mod test_crypto;
use test_crypto::*;

mod test_assert;
use test_assert::*;

pub mod test_rts;
use test_rts::*;

mod test_seal;
use test_seal::*;

mod test_rand;
use test_rand::*;

mod test_serialize;
use test_serialize::*;

mod test_file;
use test_file::*;

mod test_time;
use test_time::*;

mod test_rand_cratesio;
use test_rand_cratesio::*;

mod test_types;
use test_types::*;

mod test_env;
use test_env::*;

mod test_path;
use test_path::*;

mod test_thread;
use test_thread::*;

mod test_mpsc;
use test_mpsc::*;

mod test_alignbox;
use test_alignbox::*;

mod test_alignstruct;

mod test_signal;
use test_signal::*;

mod test_exception;
use test_exception::*;

mod test_fp;
use test_fp::*;

#[no_mangle]
pub extern "C" fn test_main_entrance() -> size_t {
    rsgx_unit_tests!(
        // tcrypto
        test_rsgx_sha256_slice,
        test_rsgx_sha256_handle,
        // assert
        foo_panic,
        foo_should,
        foo_assert,
        // rts::veh
        test_register_first_exception_handler,
        test_register_last_exception_handler,
        test_register_multiple_exception_handler,
        // rts::trts
        test_rsgx_get_thread_policy,
        test_trts_sizes,
        test_read_rand,
        test_data_is_within_enclave,
        test_slice_is_within_enlave,
        test_raw_is_within_enclave,
        test_data_is_outside_enclave,
        test_slice_is_outside_enclave,
        test_raw_is_outside_enclave,
        // rts::macros
        test_global_ctors_object,
        // rts::error
        test_error,
        // rts::libc
        test_rts_libc_memchr,
        test_rts_libc_memrchr,
        // rts::memchr
        test_rts_memchr_memchr,
        test_rts_memchr_memrchr,
        test_ascii,
        // rts::c_str
        test_cstr,
        // tseal
        test_seal_unseal,
        test_number_sealing, // Thanks to @silvanegli
        test_array_sealing,  // Thanks to @silvanegli
        test_mac_aadata_slice,
        test_mac_aadata_number,
        // rand
        test_rand_os_sgxrng,
        test_rand_distributions,
        test_rand_isaac_isaacrng,
        test_rand_chacharng,
        test_rand_reseeding,
        // serialize
        test_serialize_base,
        test_serialize_struct,
        test_serialize_enum,
        // std::sgxfs
        test_sgxfs,
        // std::fs
        test_fs,
        // std::fs untrusted mode
        test_fs_untrusted_fs_feature_enabled,
        // std::time
        test_std_time,
        // rand
        test_rand_cratesio,
        // types
        check_metadata_size,
        check_version,
        // env
        test_env_vars_os,
        test_env_self_exe_path,
        test_env_current_dir,
        test_env_home_dir,
        //path
        test_path_stat_is_correct_on_is_dir,
        test_path_fileinfo_false_when_checking_is_file_on_a_directory,
        test_path_directoryinfo_check_exists_before_and_after_mkdir,
        test_path_directoryinfo_readdir,
        test_path_mkdir_path_already_exists_error,
        test_path_recursive_mkdir,
        test_path_recursive_mkdir_failure,
        test_path_recursive_mkdir_slash,
        test_path_recursive_mkdir_dot,
        test_path_recursive_mkdir_empty,
        test_path_recursive_rmdir,
        test_path_recursive_rmdir_of_symlink,
        test_path_unicode_path_is_dir,
        test_path_unicode_path_exists,
        test_path_copy_file_dst_dir,
        test_path_copy_file_src_dir,
        test_path_canonicalize_works_simple,
        test_path_dir_entry_methods,
        test_path_read_dir_not_found,
        test_path_mkdir_trailing_slash,
        test_path_create_dir_all_with_junctions,
        test_path_copy_file_follows_dst_symlink,
        // thread
        test_thread_unnamed_thread,
        test_thread_named_thread,
        test_thread_run_basic,
        test_thread_spawn_sched,
        test_thread_spawn_sched_childs_on_default_sched,
        test_thread_test_avoid_copying_the_body_spawn,
        test_thread_test_avoid_copying_the_body_thread_spawn,
        test_thread_test_avoid_copying_the_body_join,
        test_thread_invalid_named_thread,
        test_thread_join_panic,
        test_thread_child_doesnt_ref_parent,
        test_thread_simple_newsched_spawn,
        test_thread_try_panic_message_static_str,
        test_thread_try_panic_message_owned_str,
        test_thread_try_panic_message_any,
        test_thread_try_panic_message_unit_struct,
        test_thread_park_timeout_unpark_before,
        test_thread_park_timeout_unpark_not_called,
        test_thread_park_timeout_unpark_called_other_thread,
        test_thread_sleep_ms_smoke,
        test_thread_size_of_option_thread_id,
        test_thread_id_equal,
        test_thread_id_not_equal,
        //test mpsc
        test_mpsc_smoke,
        test_mpsc_drop_full,
        test_mpsc_drop_full_shared,
        test_mpsc_smoke_shared,
        test_mpsc_smoke_threads,
        test_mpsc_smoke_port_gone,
        test_mpsc_smoke_shared_port_gone,
        test_mpsc_smoke_shared_port_gone2,
        test_mpsc_port_gone_concurrent,
        test_mpsc_port_gone_concurrent_shared,
        test_mpsc_smoke_chan_gone,
        test_mpsc_smoke_chan_gone_shared,
        test_mpsc_chan_gone_concurrent,
        test_mpsc_stress,
        test_mpsc_stress_shared,
        test_mpsc_send_from_outside_runtime,
        test_mpsc_recv_from_outside_runtime,
        test_mpsc_no_runtime,
        test_mpsc_oneshot_single_thread_close_port_first,
        test_mpsc_oneshot_single_thread_close_chan_first,
        test_mpsc_oneshot_single_thread_send_port_close,
        //test_mpsc_oneshot_single_thread_recv_chan_close, //should panic
        test_mpsc_oneshot_single_thread_send_then_recv,
        test_mpsc_oneshot_single_thread_try_send_open,
        test_mpsc_oneshot_single_thread_try_send_closed,
        test_mpsc_oneshot_single_thread_try_recv_open,
        test_mpsc_oneshot_single_thread_try_recv_closed,
        test_mpsc_oneshot_single_thread_peek_data,
        test_mpsc_oneshot_single_thread_peek_close,
        test_mpsc_oneshot_single_thread_peek_open,
        test_mpsc_oneshot_multi_task_recv_then_send,
        //test_mpsc_oneshot_multi_task_recv_then_close, //should panic
        test_mpsc_oneshot_multi_thread_close_stress,
        //test_mpsc_oneshot_multi_thread_send_close_stress, //should panic
        //test_mpsc_oneshot_multi_thread_recv_close_stress, //should panic
        test_mpsc_oneshot_multi_thread_send_recv_stress,
        test_mpsc_stream_send_recv_stress,
        test_mpsc_oneshot_single_thread_recv_timeout,
        test_mpsc_stress_recv_timeout_two_threads,
        test_mpsc_recv_timeout_upgrade,
        test_mpsc_stress_recv_timeout_shared,
        test_mpsc_very_long_recv_timeout_wont_panic,
        test_mpsc_recv_a_lot,
        test_mpsc_shared_recv_timeout,
        test_mpsc_shared_chan_stress,
        test_mpsc_test_nested_recv_iter,
        test_mpsc_test_recv_iter_break,
        test_mpsc_test_recv_try_iter,
        test_mpsc_test_recv_into_iter_owned,
        test_mpsc_try_recv_states,
        test_mpsc_destroy_upgraded_shared_port_when_sender_still_active,
        test_mpsc_issue_32114,
        test_mpsc_sync_smoke,
        test_mpsc_sync_drop_full,
        test_mpsc_sync_smoke_shared,
        test_mpsc_sync_recv_timeout,
        test_mpsc_sync_smoke_threads,
        test_mpsc_sync_smoke_port_gone,
        test_mpsc_sync_smoke_shared_port_gone2,
        test_mpsc_sync_port_gone_concurrent,
        test_mpsc_sync_port_gone_concurrent_shared,
        test_mpsc_sync_smoke_chan_gone,
        test_mpsc_sync_smoke_chan_gone_shared,
        test_mpsc_sync_chan_gone_concurrent,
        test_mpsc_sync_stress,
        test_mpsc_sync_stress_recv_timeout_two_threads,
        test_mpsc_sync_stress_recv_timeout_shared,
        test_mpsc_sync_stress_shared,
        test_mpsc_sync_oneshot_single_thread_close_port_first,
        test_mpsc_sync_oneshot_single_thread_send_port_close,
        //test_mpsc_sync_oneshot_single_thread_recv_chan_close, //should panic
        test_mpsc_sync_oneshot_single_thread_send_then_recv,
        test_mpsc_sync_oneshot_single_thread_try_send_open,
        test_mpsc_sync_oneshot_single_thread_try_send_closed,
        test_mpsc_sync_oneshot_single_thread_try_send_closed2,
        test_mpsc_sync_oneshot_single_thread_try_recv_open,
        test_mpsc_sync_oneshot_single_thread_try_recv_closed,
        test_mpsc_sync_oneshot_single_thread_try_recv_closed_with_data,
        test_mpsc_sync_oneshot_single_thread_peek_data,
        test_mpsc_sync_oneshot_single_thread_peek_close,
        test_mpsc_sync_oneshot_single_thread_peek_open,
        test_mpsc_sync_oneshot_multi_task_recv_then_send,
        //test_mpsc_sync_oneshot_multi_task_recv_then_close, //should panic
        test_mpsc_sync_oneshot_multi_thread_close_stress,
        //test_mpsc_sync_oneshot_multi_thread_send_close_stress, //should panic
        //test_mpsc_sync_oneshot_multi_thread_recv_close_stress, //should panic
        test_mpsc_sync_oneshot_multi_thread_send_recv_stress,
        test_mpsc_sync_stream_send_recv_stress,
        test_mpsc_sync_recv_a_lot,
        test_mpsc_sync_shared_sync_chan_stress,
        test_mpsc_sync_test_nested_recv_iter,
        test_mpsc_sync_test_recv_iter_break,
        test_mpsc_sync_try_recv_states,
        test_mpsc_sync_destroy_upgraded_shared_port_when_sender_still_active,
        test_mpsc_sync_send1,
        test_mpsc_sync_send2,
        test_mpsc_sync_send3,
        test_mpsc_sync_send4,
        test_mpsc_sync_try_send1,
        test_mpsc_sync_try_send2,
        test_mpsc_sync_try_send3,
        test_mpsc_sync_issue_15761,
        //test alignbox
        test_alignbox,
        test_alignbox_heap_init,
        test_alignbox_clone,
        test_alignbox_clonefrom,
        test_alignbox_clonefrom_no_eq_size,
        //test signal
        test_signal_forbidden,
        test_signal_without_pid,
        test_signal_with_pid,
        test_signal_register_unregister,
        test_signal_register_unregister1,
        //test float point
        test_fp64,
        //test exception
        test_exception_handler,
    )
}
