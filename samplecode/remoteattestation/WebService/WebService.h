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

#ifndef WEBSERVICE_H
#define WEBSERVICE_H

#include <string>
#include <stdio.h>
#include <limits.h>
#include <unistd.h>
#include <stdio.h>
#include <curl/curl.h>
#include <jsoncpp/json/json.h>
#include <iostream>

#include "LogBase.h"
#include "UtilityFunctions.h"

using namespace std;
using namespace util;

enum IAS {
    sigrl,
    report
};

struct attestation_verification_report_t {
    string report_id;
    string isv_enclave_quote_status;
    string timestamp;
};

struct attestation_evidence_payload_t {
    string isv_enclave_quote;
};

struct ias_response_header_t {
    int response_status;
    int content_length;
    std::string request_id;
};

struct ias_response_container_t {
    char *p_response;
    size_t size;
};

static int REQUEST_ID_MAX_LEN = 32;
static vector<pair<string, string>> retrieved_sigrl;

class WebService {

public:
    static WebService* getInstance();
    virtual ~WebService();
    void init();
    bool getSigRL(string gid, string *sigrl);
    bool verifyQuote(uint8_t *quote, uint8_t *pseManifest, uint8_t *nonce, vector<pair<string, string>> *result);

private:
    WebService();
    bool sendToIAS(string url, IAS type, string payload,
                   struct curl_slist *headers,
                   ias_response_container_t *ias_response_container,
                   ias_response_header_t *response_header);

    string createJSONforIAS(uint8_t *quote, uint8_t *pseManifest, uint8_t *nonce);
    vector<pair<string, string>> parseJSONfromIAS(string json);

private:
    static WebService* instance;
    CURL *curl;
};

#endif
