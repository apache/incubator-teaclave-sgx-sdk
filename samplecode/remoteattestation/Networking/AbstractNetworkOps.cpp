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
