#![cfg_attr(all(feature = "enclave", not(target_env = "sgx")), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]

#[macro_use] extern crate cfg_if;

cfg_if!{
    if #[cfg(all(feature = "enclave", not(target_env = "sgx")))] {
        extern crate sgx_tstd;
        use sgx_tstd::{prelude::v1::*,
                       io::{Read, Write, BufReader, Error},
                       vec,
                       mem::transmute};
    } else {
        use std::{mem::transmute,
                  io::{Read, Write, BufReader, Error}};
    }
}

extern crate rustls;

fn get_send_vec(mut to_send : &mut Vec<u8>) -> Vec<u8> {
    let buf_len : u64 = to_send.len() as u64;
    let lbuf: [u8; 8] = unsafe { transmute(buf_len.to_be()) };
    let mut all_data : Vec<u8> = lbuf.to_vec();
    all_data.append(&mut to_send);

    all_data
}

pub fn tls_send_vec<'a, S, T>(sock : &mut rustls::Stream<'a, S, T>,
                              mut buff : Vec<u8>) -> Result<(), Error>
    where S: 'a + rustls::Session,
              T: 'a + Read + Write {
    let send_vec = get_send_vec(&mut buff);
    sock.write_all(&send_vec)?;
    sock.flush()?;

    Ok(())
}

pub fn tls_receive_vec<'a, S, T>(sock : &mut rustls::Stream<'a, S, T>)
        -> Result<Vec<u8>, Error>
        where S: 'a + rustls::Session,
              T: 'a + Read + Write {

    let mut br = BufReader::new(sock);
    let mut lbuf : [u8;8] = [0;8];

    br.read_exact(&mut lbuf)?;
    let buf_len : u64 = u64::from_be(unsafe{transmute::<[u8;8],u64>(lbuf)});
    let mut recv_buf : Vec<u8> = vec![0u8;buf_len as usize];
    br.read_exact(&mut recv_buf)?;

    Ok(recv_buf)
}
