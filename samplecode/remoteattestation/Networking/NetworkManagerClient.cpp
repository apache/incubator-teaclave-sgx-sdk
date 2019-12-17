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

#include "NetworkManagerClient.h"
#include "../GeneralSettings.h"

NetworkManagerClient* NetworkManagerClient::instance = NULL;

NetworkManagerClient::NetworkManagerClient() {}


void NetworkManagerClient::Init() {
    if (client) {
        delete client;
        client = NULL;
    }

    boost::asio::ip::tcp::resolver resolver(this->io_service);
    boost::asio::ip::tcp::resolver::query query(this->host, std::to_string(this->port).c_str());
    boost::asio::ip::tcp::resolver::iterator iterator = resolver.resolve(query);

    boost::asio::ssl::context ctx(boost::asio::ssl::context::sslv23);
    ctx.load_verify_file(Settings::server_crt);

    this->client = new Client(io_service, ctx, iterator);
}


NetworkManagerClient* NetworkManagerClient::getInstance(int port,  std::string host) {
    if (instance == NULL) {
        instance = new NetworkManagerClient();
        instance->setPort(port);
        instance->setHost(host);
    }

    return instance;
}


void NetworkManagerClient::startService() {
    this->client->startConnection();
}


void NetworkManagerClient::setHost(std::string host) {
    this->host = host;
}


void NetworkManagerClient::connectCallbackHandler(CallbackHandler cb) {
    this->client->setCallbackHandler(cb);
}
