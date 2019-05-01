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
