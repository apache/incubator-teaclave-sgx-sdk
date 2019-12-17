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

#include <iostream>
#include <unistd.h>

#include "LogBase.h"
#include "NetworkManager.h"
#include "VerificationManager.h"
#include "UtilityFunctions.h"

using namespace util;

int Main(int argc, char *argv[]) {
    LogBase::Inst();

    int ret = 0;

    VerificationManager *vm = VerificationManager::getInstance();
    vm->init();
    vm->start();

    return ret;
}


int main( int argc, char **argv ) {
    try {
        int ret = Main(argc, argv);
        return ret;
    } catch (std::exception & e) {
        Log("exception: %s", e.what());
    } catch (...) {
        Log("unexpected exception");
    }

    return -1;
}
