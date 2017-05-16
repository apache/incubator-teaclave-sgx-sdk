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
#include "AbstractNetworkOps.h"
#include <boost/lexical_cast.hpp>
#include <boost/algorithm/string.hpp>

using namespace util;

AbstractNetworkOps::AbstractNetworkOps(boost::asio::io_service& io_service, boost::asio::ssl::context& context) : socket_(io_service, context) {}

AbstractNetworkOps::~AbstractNetworkOps() {}


AbstractNetworkOps::ssl_socket::lowest_layer_type& AbstractNetworkOps::socket() {
    return socket_.lowest_layer();
}


void AbstractNetworkOps::saveCloseSocket() {
    boost::system::error_code ec;

    socket_.lowest_layer().cancel();

    if (ec) {
        stringstream ss;
        Log("Socket shutdown error: %s", ec.message());
    } else {
        socket_.lowest_layer().close();
    }
}


void AbstractNetworkOps::read() {
    char buffer_header[20];
    memset(buffer_header, '\0', 20);

    boost::system::error_code ec;
    int read = boost::asio::read(socket_, boost::asio::buffer(buffer_header, 20), ec);

    if (ec) {
        if ((boost::asio::error::eof == ec) || (boost::asio::error::connection_reset == ec)) {
            Log("Connection has been closed by remote host");
        } else {
            Log("Unknown socket error while reading occured!", log::error);
        }
    } else {
        vector<string> incomming;
        boost::split(incomming, buffer_header, boost::is_any_of("@"));

        int msg_size = boost::lexical_cast<int>(incomming[0]);
        int type = boost::lexical_cast<int>(incomming[1]);

        char *buffer = (char*) malloc(sizeof(char) * msg_size);
        memset(buffer, '\0', sizeof(char)*msg_size);

        read = boost::asio::read(socket_, boost::asio::buffer(buffer, msg_size));

        process_read(buffer, msg_size, type);
    }
}


void AbstractNetworkOps::send(vector<string> v) {
    string type = v[0];
    string msg = v[1];

    if (msg.size() > 0) {
        const char *msg_c = msg.c_str();
        int msg_length = msg.size();

        string header = to_string(msg_length) + "@" + type;

        char buffer_header[20];
        memset(buffer_header, '\0', 20);
        memcpy(buffer_header, header.c_str(), header.length());

        boost::asio::write(socket_, boost::asio::buffer(buffer_header, 20));

        char *buffer_msg = (char*) malloc(sizeof(char) * msg_length);

        memset(buffer_msg, '\0', sizeof(char) * msg_length);
        memcpy(buffer_msg, msg_c, msg_length);

        boost::asio::write(socket_, boost::asio::buffer(buffer_msg, msg_length));

        free(buffer_msg);

        this->read();
    } else {
        this->saveCloseSocket();
    }
}


void AbstractNetworkOps::setCallbackHandler(CallbackHandler cb) {
    this->callback_handler = cb;
}


void AbstractNetworkOps::process_read(char* buffer, int msg_size, int type) {
    std::string str(reinterpret_cast<const char*>(buffer), msg_size);

    free(buffer);

    auto msg = this->callback_handler(str, type);

    if (msg.size() > 0 && msg[0].size() > 0) {
        Log("Send to client");
        send(msg);
    } else {
        Log("Close connection");
        this->saveCloseSocket();
    }
}
