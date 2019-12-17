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

#include "LogBase.h"
#include <iostream>

namespace util {

LogBase* LogBase::instance = NULL;

LogBase* LogBase::Inst() {
    if (instance == NULL) {
        instance = new LogBase();
    }

    return instance;
}


LogBase::LogBase() {
    m_enabled[log::verbose]	= false;
    m_enabled[log::info] = true;
    m_enabled[log::warning]	= true;
    m_enabled[log::error] = true;
    m_enabled[log::timer] = false;

    this->appender = new log4cpp::OstreamAppender("console", &std::cout);
    this->appender->setLayout(new log4cpp::BasicLayout());

    root.setPriority(log4cpp::Priority::INFO);
    root.addAppender(this->appender);
}


LogBase::~LogBase() {}


void LogBase::Log(const log::Fmt& msg, log::Severity s) {
    if (IsEnabled(s) && !IsEnabled(log::timer)) {
        switch (s) {
        case log::info:
            root.info(msg.str());
            break;
        case log::error:
            root.error(msg.str());
            break;
        case log::warning:
            root.warn(msg.str());
            break;
        }
    }
}


bool LogBase::Enable(log::Severity s, bool enable) {
    bool prev = m_enabled[s];
    m_enabled[s] = enable;

    return prev;
}


void LogBase::DisableAll(bool b) {
    m_enabled[log::verbose]	= b;
    m_enabled[log::info] = b;
    m_enabled[log::warning]	= b;
    m_enabled[log::error] = b;
    m_enabled[log::timer] = b;
}


bool LogBase::IsEnabled( log::Severity s ) const {
    return m_enabled[s];
}


void Log(const string& str, log::Severity s) {
    LogBase::Inst()->Log(log::Fmt(str), s);
}


void DisableAllLogs(bool b) {
    LogBase::Inst()->DisableAll(b);
}



}
