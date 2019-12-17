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

    this->read_state = MSG_HEADER;
    memset(this->read_buffer_header, '\0', 20);
    boost::asio::async_read(
        socket_,
        boost::asio::buffer(this->read_buffer_header, 20),
        boost::bind(
            &AbstractNetworkOps::handle_read,
            this,
            boost::asio::placeholders::error,
            boost::asio::placeholders::bytes_transferred));
}

void AbstractNetworkOps::handle_read(const boost::system::error_code& error, size_t bytes_transferred)
{
    if (error) {
        if ((boost::asio::error::eof == error) || (boost::asio::error::connection_reset == error)) {
            Log("Connection has been closed by remote host");
        } else {
            Log("Unknown socket error while reading occured!", log::error);
        }
        return;
    }

    vector<string> header;
    boost::split(header, this->read_buffer_header, boost::is_any_of("@"));

    int msg_size = boost::lexical_cast<int>(header[0]);
    int type = boost::lexical_cast<int>(header[1]);

    if (this->read_state == MSG_HEADER) {

        this->read_state = MSG_BODY;
        this->read_buffer_message = (char*) malloc(sizeof(char) * msg_size);
        memset(this->read_buffer_message, '\0', sizeof(char) * msg_size);
        boost::asio::async_read(
            socket_,
            boost::asio::buffer(this->read_buffer_message, msg_size),
            boost::bind(
                &AbstractNetworkOps::handle_read,
                this,
                boost::asio::placeholders::error,
                boost::asio::placeholders::bytes_transferred));

    } else if (this->read_state == MSG_BODY) {
        process_read(this->read_buffer_message, msg_size, type);
    }
}

void AbstractNetworkOps::send(vector<string> v) {
    string type = v[0];
    string msg = v[1];

    if (msg.size() <= 0) {
        this->saveCloseSocket();
        return;
    }

    const char *msg_c = msg.c_str();
    int msg_length = msg.size();

    string header = to_string(msg_length) + "@" + type;
    memset(this->write_buffer_header, '\0', 20);
    memcpy(this->write_buffer_header, header.c_str(), header.length());

    this->write_buffer_message = (char*) malloc(sizeof(char) * msg_length);
    memset(this->write_buffer_message, '\0', sizeof(char) * msg_length);
    memcpy(this->write_buffer_message, msg_c, msg_length);
    this->write_state = MSG_HEADER;

    boost::asio::async_write(
        socket_,
        boost::asio::buffer(this->write_buffer_header, 20),
        boost::asio::transfer_at_least(20),
        boost::bind(
            &AbstractNetworkOps::handle_write,
            this,
            boost::asio::placeholders::error,
            boost::asio::placeholders::bytes_transferred));
}

void AbstractNetworkOps::handle_write(const boost::system::error_code& error, size_t bytes_transferred)
{
    if (error) {
        if ((boost::asio::error::eof == error) || (boost::asio::error::connection_reset == error)) {
            Log("Connection has been closed by remote host");
        } else {
            Log("Unknown socket error while writing occured!", log::error);
        }
        return;
    }

    vector<string> header;
    boost::split(header, this->write_buffer_header, boost::is_any_of("@"));

    int msg_size = boost::lexical_cast<int>(header[0]);
    int type = boost::lexical_cast<int>(header[1]);

    if (this->write_state == MSG_HEADER) {

        this->write_state = MSG_BODY;
        boost::asio::async_write(
            socket_,
            boost::asio::buffer(this->write_buffer_message, msg_size),
            boost::asio::transfer_at_least(msg_size),
            boost::bind(
                &AbstractNetworkOps::handle_write,
                this,
                boost::asio::placeholders::error,
                boost::asio::placeholders::bytes_transferred));

    } else if (this->write_state == MSG_BODY) {

        free(this->write_buffer_message);
        this->read();
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
        send(msg);
    } else {
        Log("Close connection");
        this->saveCloseSocket();
    }
}
