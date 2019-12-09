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

#ifndef _EDL_DIRENT_H
#define _EDL_DIRENT_H

struct dirent_t
{
    uint64_t d_ino;
    int64_t d_off;
    unsigned short int d_reclen;
    unsigned char d_type;
    char d_name[256];
};

struct dirent64_t
{
    uint64_t d_ino;
    int64_t d_off;
    unsigned short int d_reclen;
    unsigned char d_type;
    char d_name[256];
};

#endif
