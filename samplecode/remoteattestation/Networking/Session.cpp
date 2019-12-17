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

#include "Session.h"

#include <boost/lexical_cast.hpp>

using namespace util;

Session::Session(boost::asio::io_service& io_service, boost::asio::ssl::context& context) : AbstractNetworkOps(io_service, context) {}

Session::~Session() {}


void Session::start() {
    Log("Connection from %s", socket().remote_endpoint().address().to_string());

    socket_.async_handshake(boost::asio::ssl::stream_base::server,
                            boost::bind(&Session::handle_handshake, this,
                                        boost::asio::placeholders::error));
}


void Session::handle_handshake(const boost::system::error_code& error) {
    if (!error) {
        Log("Handshake successful");
        this->read();
    } else {
        Log("Handshake was not successful: %s", error.message(), log::error);
        delete this;
    }
}
