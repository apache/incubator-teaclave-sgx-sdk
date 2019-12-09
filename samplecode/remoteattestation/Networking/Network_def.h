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

#ifndef NETWORK_DEF_H
#define NETWORK_DEF_H

#define MAX_VERIFICATION_RESULT 2

typedef enum _ra_msg_types {
    RA_MSG0,
    RA_MSG1,
    RA_MSG2,
    RA_MSG3,
    RA_ATT_RESULT,
    RA_VERIFICATION,
    RA_APP_ATT_OK
} ra_msg_types;


typedef enum _ra_msg {
    TYPE_OK,
    TYPE_TERMINATE
} ra_msg;


#pragma pack(1)
typedef struct _ra_samp_request_header_t {
    uint8_t  type;     /* set to one of ra_msg_type_t*/
    uint32_t size;     /*size of request body*/
    uint8_t  align[3];
    uint8_t body[];
} ra_samp_request_header_t;

typedef struct _ra_samp_response_header_t {
    uint8_t  type;      /* set to one of ra_msg_type_t*/
    uint8_t  status[2];
    uint32_t size;      /*size of the response body*/
    uint8_t  align[1];
    uint8_t  body[];
} ra_samp_response_header_t;

#pragma pack()


#endif
