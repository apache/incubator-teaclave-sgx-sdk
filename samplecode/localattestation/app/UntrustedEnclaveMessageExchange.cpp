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

#include <stdio.h>
#include "sgx_eid.h"
#include "error_codes.h"
#include "sgx_urts.h"
#include "UntrustedEnclaveMessageExchange.h"
#include "sgx_dh.h"
#include <map>

std::map<sgx_enclave_id_t, uint32_t>g_enclave_id_map;
std::map<sgx_enclave_id_t, std::map<sgx_enclave_id_t, size_t> >g_session_ptr_map;

//Makes an sgx_ecall to the destination enclave to get session id and message1
ATTESTATION_STATUS session_request_ocall(sgx_enclave_id_t src_enclave_id, sgx_enclave_id_t dest_enclave_id, sgx_dh_msg1_t* dh_msg1)
{
	uint32_t status = 0;
	sgx_status_t ret = SGX_SUCCESS;
	uint32_t temp_enclave_no;
	size_t session_ptr = 0;

	std::map<sgx_enclave_id_t, uint32_t>::iterator it = g_enclave_id_map.find(dest_enclave_id);
    if(it != g_enclave_id_map.end())
	{
		temp_enclave_no = it->second;
	}
    else
	{
		return INVALID_SESSION;
	}

	switch(temp_enclave_no)
	{
		case 1:
			ret = Enclave1_session_request(dest_enclave_id, &status, src_enclave_id, dh_msg1, &session_ptr);
			break;
		case 2:
			ret = Enclave2_session_request(dest_enclave_id, &status, src_enclave_id, dh_msg1, &session_ptr);
			break;
		case 3:
			ret = Enclave3_session_request(dest_enclave_id, &status, src_enclave_id, dh_msg1, &session_ptr);
			break;
	}
	if (ret == SGX_SUCCESS)
	{
		std::map<sgx_enclave_id_t, std::map<sgx_enclave_id_t, size_t> >::iterator it_ptr = g_session_ptr_map.find(dest_enclave_id);
		if(it_ptr != g_session_ptr_map.end())
		{
			it_ptr->second.insert(std::pair<sgx_enclave_id_t, size_t>(src_enclave_id, session_ptr));
		}
		else
		{
			std::map<sgx_enclave_id_t, size_t> sub_map;
			sub_map.insert(std::pair<sgx_enclave_id_t, size_t>(src_enclave_id, session_ptr));
			g_session_ptr_map.insert(std::pair<sgx_enclave_id_t, std::map<sgx_enclave_id_t, size_t> >(dest_enclave_id, sub_map));
		}

		return (ATTESTATION_STATUS)status;
	}
	else
	    return INVALID_SESSION;

}
//Makes an sgx_ecall to the destination enclave sends message2 from the source enclave and gets message 3 from the destination enclave
ATTESTATION_STATUS exchange_report_ocall(sgx_enclave_id_t src_enclave_id, sgx_enclave_id_t dest_enclave_id, sgx_dh_msg2_t *dh_msg2, sgx_dh_msg3_t *dh_msg3)
{
	uint32_t status = 0;
	sgx_status_t ret = SGX_SUCCESS;
	uint32_t temp_enclave_no;
	size_t session_ptr = 0;

	std::map<sgx_enclave_id_t, uint32_t>::iterator it = g_enclave_id_map.find(dest_enclave_id);
    if(it != g_enclave_id_map.end())
	{
		temp_enclave_no = it->second;
	}
    else
	{
		return INVALID_SESSION;
	}

	std::map<sgx_enclave_id_t, std::map<sgx_enclave_id_t, size_t> >::iterator it_ptr = g_session_ptr_map.find(dest_enclave_id);
    if(it_ptr != g_session_ptr_map.end())
	{
		std::map<sgx_enclave_id_t, size_t>::iterator it_ptr_sub = it_ptr->second.find(src_enclave_id);
		if(it_ptr_sub != it_ptr->second.end())
		{
			session_ptr = it_ptr_sub->second;
		}
	}
    else
	{
		return INVALID_SESSION;
	}

	switch(temp_enclave_no)
	{
		case 1:
			ret = Enclave1_exchange_report(dest_enclave_id, &status, src_enclave_id, dh_msg2, dh_msg3, (size_t*)session_ptr);
			break;
		case 2:
			ret = Enclave2_exchange_report(dest_enclave_id, &status, src_enclave_id, dh_msg2, dh_msg3, (size_t*)session_ptr);
			break;
		case 3:
			ret = Enclave3_exchange_report(dest_enclave_id, &status, src_enclave_id, dh_msg2, dh_msg3, (size_t*)session_ptr);
			break;
	}
	if (ret == SGX_SUCCESS)
		return (ATTESTATION_STATUS)status;
	else
	    return INVALID_SESSION;

}

//Make an sgx_ecall to the destination enclave to close the session
ATTESTATION_STATUS end_session_ocall(sgx_enclave_id_t src_enclave_id, sgx_enclave_id_t dest_enclave_id)
{
	uint32_t status = 0;
	sgx_status_t ret = SGX_SUCCESS;
	uint32_t temp_enclave_no;
	size_t session_ptr = 0;

	std::map<sgx_enclave_id_t, uint32_t>::iterator it = g_enclave_id_map.find(dest_enclave_id);
    if(it != g_enclave_id_map.end())
	{
		temp_enclave_no = it->second;
	}
    else
	{
		return INVALID_SESSION;
	}

	std::map<sgx_enclave_id_t, std::map<sgx_enclave_id_t, size_t> >::iterator it_ptr = g_session_ptr_map.find(dest_enclave_id);
    if(it_ptr != g_session_ptr_map.end())
	{
		std::map<sgx_enclave_id_t, size_t>::iterator it_ptr_sub = it_ptr->second.find(src_enclave_id);
		if(it_ptr_sub != it_ptr->second.end())
		{
			session_ptr = it_ptr_sub->second;
		}
	}
    else
	{
		return INVALID_SESSION;
	}

	switch(temp_enclave_no)
	{
		case 1:
			ret = Enclave1_end_session(dest_enclave_id, &status, src_enclave_id, (size_t*)session_ptr);
			break;
		case 2:
			ret = Enclave2_end_session(dest_enclave_id, &status, src_enclave_id, (size_t*)session_ptr);
			break;
		case 3:
			ret = Enclave3_end_session(dest_enclave_id, &status, src_enclave_id, (size_t*)session_ptr);
			break;
	}
	if (ret == SGX_SUCCESS)
		return (ATTESTATION_STATUS)status;
	else
	    return INVALID_SESSION;

}
