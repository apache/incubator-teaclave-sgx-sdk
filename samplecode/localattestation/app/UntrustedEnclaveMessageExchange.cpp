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
