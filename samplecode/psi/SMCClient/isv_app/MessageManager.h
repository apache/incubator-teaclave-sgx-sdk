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

#ifndef MESSAGEMANAGER_H
#define MESSAGEMANAGER_H

#include <string>
#include <stdio.h>
#include <limits.h>
#include <unistd.h>

#include "Worker.h"
#include "NetworkManagerClient.h"
#include "LogBase.h"
#include "Messages.pb.h"
#include "WebService.h"

using namespace std;

class MessageManager {

public:
    static MessageManager* getInstance();
    virtual ~MessageManager();
    int init(string path);
    vector<string> incomingHandler(string v, int type);
    void start();

private:
    MessageManager();
    string prepareVerificationRequest();
    string handleMSG0(Messages::MessageMsg0 m);
    string handleMSG1(Messages::MessageMSG1 msg);
    string handleMSG3(Messages::MessageMSG3 msg);
    string createInitMsg(int type, string msg);
    string handleAppAttOk(Messages::MessagePsiSalt msg);

    string handleHashData(Messages::MessagePsiResult msg, bool* finished);
    string handleHashIntersect(Messages::MessagePsiIntersect msg);

private:
    static MessageManager* instance;
    NetworkManagerClient *nm = NULL;
    PSIWorker *sp = NULL;
    WebService *ws = NULL;
};

#endif
