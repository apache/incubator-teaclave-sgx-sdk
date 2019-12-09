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

#ifndef NETWORKMANAGER_H
#define NETWORKMANAGER_H

#include "Server.h"
#include "Client.h"
#include "LogBase.h"
#include "Network_def.h"

#include <string>
#include <stdio.h>
#include <limits.h>
#include <unistd.h>
#include <functional>
#include <iostream>
#include <algorithm>

using namespace std;
using namespace util;

class NetworkManager {

    typedef boost::asio::ssl::stream<boost::asio::ip::tcp::socket> ssl_socket;

public:
    NetworkManager();
    virtual ~NetworkManager();
    void sendMsg();
    void Init();
    void setPort(int port);
    void printMsg(bool send, const char* msg);

    template <typename T>
    string serialize(T msg) {
        string s;
        if (msg.SerializeToString(&s)) {
            Log("Serialization successful");
            return s;
        } else {
            Log("Serialization failed", log::error);
            return "";
        }
    }

public:
    boost::asio::io_service io_service;
    int port;
};


#endif
