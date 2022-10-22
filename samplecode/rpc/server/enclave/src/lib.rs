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

extern crate sgx_libc;
extern crate sgx_types;

use sgx_types::error::SgxStatus;

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};
use tonic::{transport::Server, Request, Response, Status};

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {request:?}");

        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }
}

// uncomment this then use match main() in run_server.
// in that case, TCS num (specified in config.xml) has to be core count + 1
//#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let greeter = MyGreeter::default();

    println!("GreeterServer listening on {addr}");

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}

/// # Safety
#[no_mangle]
pub extern "C" fn run_server() -> SgxStatus {
    let result = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(32) // TCS = 32 + 1 = 33. 1 reserved for initializer
        .enable_all()
        .build()
        .map(|rt| rt.block_on(main()));

    match result {
        Ok(Ok(_)) => SgxStatus::Success,
        Ok(Err(e)) => {
            println!("Failed to run server: {e}");
            SgxStatus::Unexpected
        }
        Err(e) => {
            println!("Failed to create tokio runtime in enclave: {e}");
            SgxStatus::Unexpected
        }
    }

    // The following code snippet works with #[tokio::main]
    // annotated main function. To use it, one need to adjust TCS number
    // in config.xml to make it larger than core count (at least core count
    // + 1).

    //match main() {
    //    Ok(_) => SgxStatus::Success,
    //    Err(e) => {
    //        println!("Failed to run server: {e}");
    //        SgxStatus::Unexpected
    //    }
    //}
}
