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

#include "VerificationManager.h"
#include "../GeneralSettings.h"

#include  <iomanip>

using namespace util;
using namespace std;

VerificationManager* VerificationManager::instance = NULL;

VerificationManager::VerificationManager() {
    this->nm = NetworkManagerClient::getInstance(Settings::rh_port, Settings::rh_host);
    this->ws = WebService::getInstance();
    this->ws->init();
    this->sp = new ServiceProvider(this->ws);
}


VerificationManager::~VerificationManager() {}


VerificationManager* VerificationManager::getInstance() {
    if (instance == NULL) {
        instance = new VerificationManager();
    }

    return instance;
}


int VerificationManager::init() {
    if (this->sp) {
        delete this->sp;
        this->sp = new ServiceProvider(this->ws);
    }

    this->nm->Init();
    this->nm->connectCallbackHandler([this](string v, int type) {
        return this->incomingHandler(v, type);
    });
}


void VerificationManager::start() {
    this->nm->startService();
    Log("Remote attestation done");
}


string VerificationManager::handleMSG0(Messages::MessageMsg0 msg) {
    Log("MSG0 received");

    uint32_t extended_epid_group_id = msg.epid();
    int ret = this->sp->sp_ra_proc_msg0_req(extended_epid_group_id);

    if (ret == 0)
        msg.set_status(TYPE_OK);
    else
        msg.set_status(TYPE_TERMINATE);

    return nm->serialize(msg);
}


string VerificationManager::handleMSG1(Messages::MessageMSG1 msg1) {
    Log("MSG1 received");

    Messages::MessageMSG2 msg2;
    msg2.set_type(RA_MSG2);

    int ret = this->sp->sp_ra_proc_msg1_req(msg1, &msg2);

    if (ret != 0) {
        Log("Error, processing MSG1 failed");
    } else {
        Log("MSG1 processed correctly and MSG2 created");
        return nm->serialize(msg2);
    }

    return "";
}


string VerificationManager::handleMSG3(Messages::MessageMSG3 msg) {
    Log("MSG3 received");

    Messages::AttestationMessage att_msg;
    att_msg.set_type(RA_ATT_RESULT);

    int ret = this->sp->sp_ra_proc_msg3_req(msg, &att_msg);

    if (ret == -1) {
        Log("Error, processing MSG3 failed");
    } else {
        Log("MSG3 processed correctly and attestation result created");
        return nm->serialize(att_msg);
    }

    return "";
}


string VerificationManager::handleAppAttOk() {
    Log("APP attestation result received");
    return "";
}


string VerificationManager::prepareVerificationRequest() {
    Log("Prepare Verification request");

    Messages::InitialMessage msg;
    msg.set_type(RA_VERIFICATION);

    return nm->serialize(msg);
}


string VerificationManager::createInitMsg(int type, string msg) {
    Messages::InitialMessage init_msg;
    init_msg.set_type(type);
    init_msg.set_size(msg.size());

    return nm->serialize(init_msg);
}


vector<string> VerificationManager::incomingHandler(string v, int type) {
    vector<string> res;

    if (!v.empty()) {
        string s;
        bool ret;

        switch (type) {
        case RA_MSG0: {
            Messages::MessageMsg0 msg0;
            ret = msg0.ParseFromString(v);
            if (ret && (msg0.type() == RA_MSG0)) {
                s = this->handleMSG0(msg0);
                res.push_back(to_string(RA_MSG0));
            }
        }
        break;
        case RA_MSG1: {
            Messages::MessageMSG1 msg1;
            ret = msg1.ParseFromString(v);
            if (ret && (msg1.type() == RA_MSG1)) {
                s = this->handleMSG1(msg1);
                res.push_back(to_string(RA_MSG2));
            }
        }
        break;
        case RA_MSG3: {
            Messages::MessageMSG3 msg3;
            ret = msg3.ParseFromString(v);
            if (ret && (msg3.type() == RA_MSG3)) {
                s = this->handleMSG3(msg3);
                res.push_back(to_string(RA_ATT_RESULT));
            }
        }
        break;
        case RA_APP_ATT_OK: {
            Messages::SecretMessage sec_msg;
            ret = sec_msg.ParseFromString(v);
            if (ret) {
                if (sec_msg.type() == RA_APP_ATT_OK) {
                    this->handleAppAttOk();
                }
            }
        }
        break;
        default:
            Log("Unknown type: %d", type, log::error);
            break;
        }

        res.push_back(s);
    } else { 	//after handshake
        res.push_back(to_string(RA_VERIFICATION));
        res.push_back(this->prepareVerificationRequest());
    }

    return res;
}
