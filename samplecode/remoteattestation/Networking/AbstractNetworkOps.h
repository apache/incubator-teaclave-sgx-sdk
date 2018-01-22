// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
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
