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

#include "Server.h"
#include "../GeneralSettings.h"

using namespace util;

Server::Server(boost::asio::io_service& io_service, int port) : io_service_(io_service), acceptor_(io_service,
            boost::asio::ip::tcp::endpoint(boost::asio::ip::tcp::v4(), port)),
    context_(boost::asio::ssl::context::sslv23) {

    this->context_.set_options(boost::asio::ssl::context::default_workarounds
                               | boost::asio::ssl::context::no_sslv2
                               | boost::asio::ssl::context::single_dh_use);

    this->context_.use_certificate_chain_file(Settings::server_crt);
    this->context_.use_private_key_file(Settings::server_key, boost::asio::ssl::context::pem);

    Log("Certificate \"" + Settings::server_crt + "\" set");
    Log("Server running on port: %d", port);
}


Server::~Server() {}


void Server::start_accept() {
    Session *new_session = new Session(io_service_, context_);
    new_session->setCallbackHandler(this->callback_handler);
    acceptor_.async_accept(new_session->socket(), boost::bind(&Server::handle_accept, this, new_session, boost::asio::placeholders::error));
}


void Server::handle_accept(Session* new_session, const boost::system::error_code& error) {
    if (!error) {
        Log("New accept request, starting new session");
        new_session->start();
    } else {
        delete new_session;
    }

    start_accept();
}

void Server::connectCallbackHandler(CallbackHandler cb) {
    this->callback_handler = cb;
}
