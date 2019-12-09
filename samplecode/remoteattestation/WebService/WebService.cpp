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

#include "WebService.h"
#include "../GeneralSettings.h"

WebService* WebService::instance = NULL;

WebService::WebService() {}

WebService::~WebService() {
    if (curl)
        curl_easy_cleanup(curl);
}


WebService* WebService::getInstance() {
    if (instance == NULL) {
        instance = new WebService();
    }

    return instance;
}


void WebService::init() {
    curl_global_init(CURL_GLOBAL_DEFAULT);

    curl = curl_easy_init();

    if (curl) {
        Log("Curl initialized successfully");
//		curl_easy_setopt( curl, CURLOPT_VERBOSE, 1L );
        curl_easy_setopt( curl, CURLOPT_SSLCERTTYPE, "PEM");
        curl_easy_setopt( curl, CURLOPT_SSLCERT, Settings::ias_crt);
        curl_easy_setopt( curl, CURLOPT_USE_SSL, CURLUSESSL_ALL);
        curl_easy_setopt( curl, CURLOPT_SSLVERSION, CURL_SSLVERSION_TLSv1_2);
        curl_easy_setopt( curl, CURLOPT_NOPROGRESS, 1L);
    } else
        Log("Curl init error", log::error);
}


vector<pair<string, string>> WebService::parseJSONfromIAS(string json) {
    Json::Value root;
    Json::Reader reader;
    bool parsingSuccessful = reader.parse(json.c_str(), root);

    if (!parsingSuccessful) {
        Log("Failed to parse JSON string from IAS", log::error);
        return vector<pair<string, string>>();
    }

    vector<pair<string,string>> values;

    string id = root.get("id", "UTF-8" ).asString();
    string timestamp = root.get("timestamp", "UTF-8" ).asString();
    string epidPseudonym = root.get("epidPseudonym", "UTF-8" ).asString();
    string isvEnclaveQuoteStatus = root.get("isvEnclaveQuoteStatus", "UTF-8" ).asString();

    values.push_back({"id", id});
    values.push_back({"timestamp", timestamp});
    values.push_back({"epidPseudonym", epidPseudonym});
    values.push_back({"isvEnclaveQuoteStatus", isvEnclaveQuoteStatus});

    return values;
}


string WebService::createJSONforIAS(uint8_t *quote, uint8_t *pseManifest, uint8_t *nonce) {
    Json::Value request;

    request["isvEnclaveQuote"] = Base64encodeUint8(quote, 1116);
//    request["pseManifest"] = Base64encodeUint8(quote, 256);		//only needed when enclave has been signed

    Json::FastWriter fastWriter;
    string output = fastWriter.write(request);

    return output;
}


size_t ias_response_header_parser(void *ptr, size_t size, size_t nmemb, void *userdata) {
    int parsed_fields = 0, response_status, content_length, ret = size * nmemb;

    char *x = (char*) calloc(size+1, nmemb);
    assert(x);
    memcpy(x, ptr, size * nmemb);
    parsed_fields = sscanf( x, "HTTP/1.1 %d", &response_status );

    if (parsed_fields == 1) {
        ((ias_response_header_t *) userdata)->response_status = response_status;
        free(x);
        return ret;
    }

    parsed_fields = sscanf( x, "content-length: %d", &content_length );
    if (parsed_fields == 1) {
        ((ias_response_header_t *) userdata)->content_length = content_length;
        free(x);
        return ret;
    }

    char *p_request_id = (char*) calloc(1, REQUEST_ID_MAX_LEN);
    parsed_fields = sscanf(x, "request-id: %s", p_request_id );

    if (parsed_fields == 1) {
        std::string request_id_str( p_request_id );
        ( ( ias_response_header_t * ) userdata )->request_id = request_id_str;
        free(x);
        free(p_request_id);
        return ret;
    }

    free(x);
    free(p_request_id);
    return ret;
}


size_t ias_reponse_body_handler( void *ptr, size_t size, size_t nmemb, void *userdata ) {
    size_t realsize = size * nmemb;
    ias_response_container_t *ias_response_container = ( ias_response_container_t * ) userdata;
    ias_response_container->p_response = (char *) realloc(ias_response_container->p_response, ias_response_container->size + realsize + 1);

    if (ias_response_container->p_response == NULL ) {
        Log("Unable to allocate extra memory", log::error);
        return 0;
    }

    memcpy( &( ias_response_container->p_response[ias_response_container->size]), ptr, realsize );
    ias_response_container->size += realsize;
    ias_response_container->p_response[ias_response_container->size] = 0;

    return realsize;
}


bool WebService::sendToIAS(string url,
                           IAS type,
                           string payload,
                           struct curl_slist *headers,
                           ias_response_container_t *ias_response_container,
                           ias_response_header_t *response_header) {

    CURLcode res = CURLE_OK;

    curl_easy_setopt( curl, CURLOPT_URL, url.c_str());
    Log("sending url: %s", url.c_str());

    if (headers) {
        curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
        curl_easy_setopt(curl, CURLOPT_POSTFIELDS, payload.c_str());
    }

    ias_response_container->p_response = (char*) malloc(1);
    ias_response_container->size = 0;

    curl_easy_setopt(curl, CURLOPT_HEADERFUNCTION, ias_response_header_parser);
    curl_easy_setopt(curl, CURLOPT_HEADERDATA, response_header);
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, ias_reponse_body_handler);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, ias_response_container);

    res = curl_easy_perform(curl);
    if (res != CURLE_OK) {
        Log("Curl cert file: %s", Settings::ias_crt);
        Log("curl_easy_perform() failed: %s", curl_easy_strerror(res));
        return false;
    }

    return true;
}


bool WebService::getSigRL(string gid, string *sigrl) {
    Log("Retrieving SigRL from IAS");

    //check if the sigrl for the gid has already been retrieved once -> to save time
    for (auto x : retrieved_sigrl) {
        if (x.first == gid) {
            *sigrl = x.second;
            return false;
        }
    }

    ias_response_container_t ias_response_container;
    ias_response_header_t response_header;

    string url = Settings::ias_url + "sigrl/" + gid;

    this->sendToIAS(url, IAS::sigrl, "", NULL, &ias_response_container, &response_header);

    Log("\tResponse status is: %d" , response_header.response_status);
    Log("\tContent-Length: %d", response_header.content_length);

    if (response_header.response_status == 200) {
        if (response_header.content_length > 0) {
            string response(ias_response_container.p_response);
            *sigrl = Base64decode(response);
        }
        retrieved_sigrl.push_back({gid, *sigrl});
    } else
        return true;

    return false;
}


bool WebService::verifyQuote(uint8_t *quote, uint8_t *pseManifest, uint8_t *nonce, vector<pair<string, string>> *result) {
    string encoded_quote = this->createJSONforIAS(quote, pseManifest, nonce);

    ias_response_container_t ias_response_container;
    ias_response_header_t response_header;

    struct curl_slist *headers = NULL;
    headers = curl_slist_append(headers, "Content-Type: application/json");

    string payload = encoded_quote;

    string url = Settings::ias_url + "report";
    this->sendToIAS(url, IAS::report, payload, headers, &ias_response_container, &response_header);


    if (response_header.response_status == 200) {
        Log("Quote attestation successful, new report has been created");

        string response(ias_response_container.p_response);

        auto res = parseJSONfromIAS(response);
        *result = res;
    } else {
        Log("Quote attestation returned status: %d", response_header.response_status);
        return true;
    }

    return false;
}
