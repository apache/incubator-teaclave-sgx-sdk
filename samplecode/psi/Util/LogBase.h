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

#ifndef LOG_H
#define LOG_H

#include <log4cpp/Category.hh>
#include <log4cpp/Appender.hh>
#include <log4cpp/FileAppender.hh>
#include <log4cpp/OstreamAppender.hh>
#include <log4cpp/Layout.hh>
#include <log4cpp/BasicLayout.hh>
#include <log4cpp/Priority.hh>

#include <boost/format.hpp>
#include <memory>
#include <bitset>
#include <mutex>
#include <string>

using namespace std;

namespace util {

namespace log {
enum Severity {
    verbose,
    info,
    warning,
    error,
    timer,
    severity_count
};

typedef boost::format Fmt;
}


class LogBase {

public:
    static LogBase* Inst();
    void Log(const log::Fmt& msg, log::Severity s = log::info);
    bool Enable(log::Severity s, bool enable = true);
    bool IsEnabled(log::Severity s) const;
    void DisableAll(bool b);
    virtual ~LogBase();

private:
    LogBase();

private:
    std::bitset<log::severity_count> m_enabled;
    static LogBase *instance;
    log4cpp::Appender *appender;
    log4cpp::Category &root = log4cpp::Category::getRoot();

    struct timespec start;
    vector<pair<string, string>> measurements;
};

void Log(const std::string& str, log::Severity s = log::info);
void DisableAllLogs(bool b);

template <typename P1>
void Log(const std::string& fmt, const P1& p1, log::Severity s = log::info) {
    LogBase::Inst()->Log(log::Fmt(fmt) % p1, s);
}

template <typename P1, typename P2>
void Log(const std::string& fmt, const P1& p1, const P2& p2, log::Severity s = log::info) {
    LogBase::Inst()->Log( log::Fmt(fmt) % p1 % p2, s ) ;
}

template <typename P1, typename P2, typename P3>
void Log(const std::string& fmt, const P1& p1, const P2& p2, const P3& p3, log::Severity s = log::info) {
    LogBase::Inst()->Log(log::Fmt(fmt) % p1 % p2 % p3, s);
}

template <typename P1, typename P2, typename P3, typename P4>
void Log(const std::string& fmt, const P1& p1, const P2& p2, const P3& p3, const P4& p4, log::Severity s = log::info) {
    LogBase::Inst()->Log(log::Fmt(fmt) % p1 % p2 % p3 % p4, s);
}

template <typename P1, typename P2, typename P3, typename P4, typename P5>
void Log(const std::string& fmt, const P1& p1, const P2& p2, const P3& p3, const P4& p4, const P5& p5, log::Severity s = log::info) {
    LogBase::Inst()->Log(log::Fmt(fmt) % p1 % p2 % p3 % p4 % p5, s);
}

}

#endif
