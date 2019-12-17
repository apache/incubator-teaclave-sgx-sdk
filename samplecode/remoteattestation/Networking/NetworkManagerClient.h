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

#include "NetworkManager.h"

class NetworkManagerClient : public NetworkManager {

public:
    static NetworkManagerClient* getInstance(int port, std::string host = "localhost");
    void Init();
    void connectCallbackHandler(CallbackHandler cb);
    void startService();
    void setHost(std::string host);

private:
    NetworkManagerClient();

private:
    static NetworkManagerClient* instance;
    std::string host;
    Client *client = NULL;
};

