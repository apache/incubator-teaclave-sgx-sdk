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

#include "MessageManager.h"
#include "../GeneralSettings.h"

#include  <iomanip>

using namespace util;
using namespace std;

MessageManager* MessageManager::instance = NULL;

MessageManager::MessageManager() {
    this->nm = NetworkManagerClient::getInstance(Settings::rh_port, Settings::rh_host);
    this->ws = WebService::getInstance();
    this->ws->init();
    this->sp = new PSIWorker(this->ws);
}

MessageManager::~MessageManager() {}

MessageManager* MessageManager::getInstance() {
    if (instance == NULL) {
        instance = new MessageManager();
    }
    return instance;
}

int MessageManager::init(string path) {
    if (this->sp) {
        delete this->sp;
        this->sp = new PSIWorker(this->ws);
    }

    this->sp->set_hash_path(path);

    this->nm->Init();
    this->nm->connectCallbackHandler([this](string v, int type) {
        return this->incomingHandler(v, type);
    });
}

void MessageManager::start() {
    this->nm->startService();
    Log("[PSI] PSI done");
}

string MessageManager::handleMSG0(Messages::MessageMsg0 msg) {
    Log("MSG0 received");

    uint32_t extended_epid_group_id = msg.epid();
    int ret = this->sp->sp_ra_proc_msg0_req(extended_epid_group_id);

    if (ret == 0)
        msg.set_status(TYPE_OK);
    else
        msg.set_status(TYPE_TERMINATE);

    return nm->serialize(msg);
}

string MessageManager::handleMSG1(Messages::MessageMSG1 msg1) {
    Log("MSG1 received");

    Messages::MessageMSG2 msg2;
    msg2.set_type(RA_MSG2);
    msg2.set_context(msg1.context());

    int ret = this->sp->sp_ra_proc_msg1_req(msg1, &msg2);

    if (ret != 0) {
        Log("Error, processing MSG1 failed");
    } else {
        Log("MSG1 processed correctly and MSG2 created");
        return nm->serialize(msg2);
    }

    return "";
}

string MessageManager::handleMSG3(Messages::MessageMSG3 msg) {
    Log("MSG3 received");

    Messages::AttestationMessage att_msg;
    att_msg.set_type(RA_ATT_RESULT);
    att_msg.set_context(msg.context());

    int ret = this->sp->sp_ra_proc_msg3_req(msg, &att_msg);

    if (ret == -1) {
        Log("Error, processing MSG3 failed");
    } else {
        Log("MSG3 processed correctly and attestation result created");
        return nm->serialize(att_msg);
    }

    return "";
}

string MessageManager::handleAppAttOk(Messages::MessagePsiSalt msg) {
    Log("APP attestation result received");

    if (msg.state() == 0) {
        return "";
    }

    if (this->sp->set_hash_salt(msg) == -1) {
        return "";
    }

    Messages::MessagePsiHashData hash_data;
    hash_data.set_type(RA_PSI_HASHDATA);
    hash_data.set_context(msg.context());
    hash_data.set_id(msg.id());

    int ret = this->sp->sp_psi_get_data_hash(&hash_data);

    if (ret == -1) {
        Log("Error, processing hash data failed");
    } else {
        Log("[PSI] Send Hash data firstly");
        return nm->serialize(hash_data);
    }

    return "";
}

string MessageManager::handleHashData(Messages::MessagePsiResult msg, bool* finished) {

    if (this->sp->sp_psi_is_finish_get_data()) {
        Log("[PSI] Send hash data finished");

        Messages::MessagePsiHashDataFinished finish;
        finish.set_type(RA_PSI_HASHDATA_FINISHED);
        finish.set_size(sizeof(uint32_t));
        finish.set_context(msg.context());
        finish.set_id(msg.id());

        *finished = true;

        return nm->serialize(finish);

    } else {
        Messages::MessagePsiHashData hash_data;
        hash_data.set_type(RA_PSI_HASHDATA);
        hash_data.set_context(msg.context());
        hash_data.set_id(msg.id());

        int ret = this->sp->sp_psi_get_data_hash(&hash_data);

        if (ret == -1) {
            Log("Error, processing hash data failed");
        } else {
            Log("[PSI] Send hash data again");
            return nm->serialize(hash_data);
        }    
    }
    
    return "";
}

string MessageManager::handleHashIntersect(Messages::MessagePsiIntersect msg) {
    Log("[PSI] Intersect done, show result");

    this->sp->sp_psi_intersect(msg);

    return "";
}

string MessageManager::prepareVerificationRequest() {
    Log("Prepare Verification request");

    Messages::InitialMessage msg;
    msg.set_type(RA_VERIFICATION);

    return nm->serialize(msg);
}

string MessageManager::createInitMsg(int type, string msg) {
    Messages::InitialMessage init_msg;
    init_msg.set_type(type);
    init_msg.set_size(msg.size());

    return nm->serialize(init_msg);
}

vector<string> MessageManager::incomingHandler(string v, int type) {
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
        case RA_PSI_SLAT: {
            Messages::MessagePsiSalt att_ok_msg;
            ret = att_ok_msg.ParseFromString(v);
            if (ret) {
                if (att_ok_msg.type() == RA_PSI_SLAT) {
                    s = this->handleAppAttOk(att_ok_msg);
                    res.push_back(to_string(RA_PSI_HASHDATA));
                }
            }
        }
        break;
        case RA_PSI_RESULT: {
            Messages::MessagePsiResult msg;
            ret = msg.ParseFromString(v);
            if (ret) {
                if (msg.type() == RA_PSI_RESULT) {
                    if (msg.state() == 1) {
                        //no intersect result, send finished again.
                        sleep(1);
                    }
                    bool finished = false;
                    s = this->handleHashData(msg, &finished);
                    if (finished) {
                        res.push_back(to_string(RA_PSI_HASHDATA_FINISHED));
                    } else {
                        res.push_back(to_string(RA_PSI_HASHDATA));
                    }
                }
            }
        }
        break;
        case RA_PSI_INTERSECT: {
            Messages::MessagePsiIntersect inter_msg;
            ret = inter_msg.ParseFromString(v);
            if (ret) {
                if (inter_msg.type() == RA_PSI_INTERSECT) {
                    s = this->handleHashIntersect(inter_msg);
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
