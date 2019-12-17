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

#ifndef ABSTRACTNETWORKOPS_H
#define ABSTRACTNETWORKOPS_H

#include "LogBase.h"

#include <vector>
#include <cstdlib>
#include <iostream>
#include <boost/bind.hpp>
#include <boost/asio.hpp>
#include <boost/asio/ssl.hpp>
#include <boost/asio/ip/tcp.hpp>
#include <functional>
#include <boost/asio/buffer.hpp>
#include <boost/asio.hpp>
#include <boost/algorithm/string.hpp>

using namespace std;

typedef function<vector<string>(string, int)> CallbackHandler;

class AbstractNetworkOps {

    typedef boost::asio::ssl::stream<boost::asio::ip::tcp::socket> ssl_socket;

public:
    AbstractNetworkOps();
    AbstractNetworkOps(boost::asio::io_service& io_service, boost::asio::ssl::context& context);
    virtual ~AbstractNetworkOps();
    ssl_socket::lowest_layer_type& socket();
    void setCallbackHandler(CallbackHandler cb);

protected:
    ssl_socket socket_;
    enum { max_length = 1024 };
    CallbackHandler callback_handler = NULL;

protected:
    void read();
    void send(vector<string>);
    void process_read(char* buffer, int size, int type);

private:
    void saveCloseSocket();

};


#endif
