/*
 * Copyright (C) 2011-2016 2017 Baidu, Inc. All Rights Reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 *
 *   * Redistributions of source code must retain the above copyright
 *     notice, this list of conditions and the following disclaimer.
 *   * Redistributions in binary form must reproduce the above copyright
 *     notice, this list of conditions and the following disclaimer in
 *     the documentation and/or other materials provided with the
 *     distribution.
 *   * Neither the name of Baidu, Inc., nor the names of its
 *     contributors may be used to endorse or promote products derived
 *     from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
 * A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
 * OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
 * SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
 * LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
 * DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
 * THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
 * OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 *
 */

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
