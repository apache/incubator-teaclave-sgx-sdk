/*
 * Copyright (C) 2011-2016 2017 Baidu, Inc. All Rights Reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 *   * Redistributions of source code must retain the above copyright
 *     notice, this list of conditions and the following disclaimer.
 *   * Redistributions in binary form must reproduce the above copyright
 *     notice, this list of conditions and the following disclaimer in
 *     the documentation and/or other materials provided with the
 *     distribution.
 *   * Neither the name of Baidu, Inc., nor the names of its
 *     contributors may be used to endorse or promote products derived
 *     from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *
 */

#include "Client.h"
#include "LogBase.h"
#include "Network_def.h"
#include "Messages.pb.h"

#include <boost/lexical_cast.hpp>

using namespace util;

Client::Client(boost::asio::io_service& io_service,
               boost::asio::ssl::context& context,
               boost::asio::ip::tcp::resolver::iterator endpoint_iterator) : AbstractNetworkOps(io_service, context) {
    socket_.set_verify_mode(boost::asio::ssl::verify_peer);
    socket_.set_verify_callback(boost::bind(&Client::verify_certificate, this, _1, _2));

    this->endpoint_iterator = endpoint_iterator;
}

Client::~Client() {}


void Client::startConnection() {
    Log("Start connecting...");

    boost::system::error_code ec;
    boost::asio::connect(socket_.lowest_layer(), this->endpoint_iterator, ec);

    handle_connect(ec);
}


bool Client::verify_certificate(bool preverified, boost::asio::ssl::verify_context& ctx) {
    char subject_name[256];
    X509* cert = X509_STORE_CTX_get_current_cert(ctx.native_handle());
    X509_NAME_oneline(X509_get_subject_name(cert), subject_name, 256);

    Log("Verifying certificate: %s", subject_name);

    return preverified;
}


void Client::handle_connect(const boost::system::error_code &error) {
    if (!error) {
        Log("Connection established");

        boost::system::error_code ec;
        socket_.handshake(boost::asio::ssl::stream_base::client, ec);

        handle_handshake(ec);
    } else {
        Log("Connect failed: %s", error.message(), log::error);
    }
}


void Client::handle_handshake(const boost::system::error_code& error) {
    if (!error) {
        Log("Handshake successful");

        auto ret = this->callback_handler("", -1);
        send(ret);
    } else {
        Log("Handshake failed: %s", error.message(), log::error);
    }
}
