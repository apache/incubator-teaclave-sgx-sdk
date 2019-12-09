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

#include "UtilityFunctions.h"

using namespace util;

void SafeFree(void *ptr) {
    if (NULL != ptr) {
        free(ptr);
        ptr = NULL;
    }
}


string GetRandomString() {
    string str = lexical_cast<string>((random_generator())());
    str.erase(remove(str.begin(), str.end(), '-'), str.end());

    return str;
}


string ByteArrayToString(const uint8_t *arr, int size) {
    ostringstream convert;

    for (int a = 0; a < size; a++) {
        convert << setfill('0') << setw(2) << hex << (unsigned int)arr[a];
    }

    return convert.str();
}


string ByteArrayToStringNoFill(const uint8_t *arr, int size) {
    ostringstream convert;

    for (int a = 0; a < size; a++) {
        convert << hex << (int)arr[a];
    }

    return convert.str();
}


int HexStringToByteArray(string str, uint8_t **arr) {
    vector<uint8_t> bytes;

    for (unsigned int i=0; i<str.length(); i+=2) {
        string byteString = str.substr(i, 2);
        char byte = (char) strtol(byteString.c_str(), NULL, 16);
        bytes.push_back((unsigned char)byte);
    }

    *arr = (uint8_t*) malloc(sizeof(uint8_t) * bytes.size());
    copy(bytes.begin(), bytes.end(), *arr);

    return bytes.size();
}


int StringToByteArray(string str, uint8_t **arr) {
    vector<uint8_t> vec(str.begin(), str.end());

    *arr = (uint8_t*) malloc(sizeof(uint8_t) * vec.size());
    copy(vec.begin(), vec.end(), *arr);

    return vec.size();
}


string ByteArrayToNoHexString(const uint8_t *arr, int size) {
    std::ostringstream convert;

    for (int a = 0; a < size; a++) {
        convert << (uint8_t)arr[a];
    }

    return convert.str();
}


string UIntToString(uint32_t *arr, int size) {
    stringstream ss;

    for (int i=0; i<size; i++) {
        ss << arr[i];
    }

    return ss.str();
}


int SaveBufferToFile(string filePath, string content) {
    std::ofstream out(filePath);
    out << content;
    out.close();
    return 0;
}


int ReadFileToBuffer(string filePath, char **content) {
    ifstream t(filePath);
    string str((istreambuf_iterator<char>(t)), istreambuf_iterator<char>());

    *content = (char*) malloc(sizeof(char) * (str.size()+1));
    memset(*content, '\0', (str.size()+1));
    str.copy(*content, str.size());

    return str.size();
}


int ReadFileToBuffer(string filePath, uint8_t **content) {
    ifstream file(filePath, ios::binary | ios::ate);
    streamsize file_size = file.tellg();

    file.seekg(0, ios::beg);

    std::vector<char> buffer(file_size);

    if (file.read(buffer.data(), file_size)) {
        string str(buffer.begin(), buffer.end());

        vector<uint8_t> vec(str.begin(), str.end());

        *content = (uint8_t*) malloc(sizeof(uint8_t) * vec.size());
        copy(vec.begin(), vec.end(), *content);

        return str.length();
    }

    return -1;
}


int RemoveFile(string filePath) {
    if (remove(filePath.c_str()) != 0 ) {
        Log("Error deleting file: " + filePath);
        return 1;
    } else
        Log("File deleted successfully: " + filePath);

    return 0;
}


static sgx_errlist_t sgx_errlist[] = {
    {
        SGX_ERROR_UNEXPECTED,
        "Unexpected error occurred.",
        NULL
    },
    {
        SGX_ERROR_INVALID_PARAMETER,
        "Invalid parameter.",
        NULL
    },
    {
        SGX_ERROR_OUT_OF_MEMORY,
        "Out of memory.",
        NULL
    },
    {
        SGX_ERROR_ENCLAVE_LOST,
        "Power transition occurred.",
        "Please refer to the sample \"PowerTransition\" for details."
    },
    {
        SGX_ERROR_INVALID_ENCLAVE,
        "Invalid enclave image.",
        NULL
    },
    {
        SGX_ERROR_INVALID_ENCLAVE_ID,
        "Invalid enclave identification.",
        NULL
    },
    {
        SGX_ERROR_INVALID_SIGNATURE,
        "Invalid enclave signature.",
        NULL
    },
    {
        SGX_ERROR_OUT_OF_EPC,
        "Out of EPC memory.",
        NULL
    },
    {
        SGX_ERROR_NO_DEVICE,
        "Invalid SGX device.",
        "Please make sure SGX module is enabled in the BIOS, and install SGX driver afterwards."
    },
    {
        SGX_ERROR_MEMORY_MAP_CONFLICT,
        "Memory map conflicted.",
        NULL
    },
    {
        SGX_ERROR_INVALID_METADATA,
        "Invalid enclave metadata.",
        NULL
    },
    {
        SGX_ERROR_DEVICE_BUSY,
        "SGX device was busy.",
        NULL
    },
    {
        SGX_ERROR_INVALID_VERSION,
        "Enclave version was invalid.",
        NULL
    },
    {
        SGX_ERROR_INVALID_ATTRIBUTE,
        "Enclave was not authorized.",
        NULL
    },
    {
        SGX_ERROR_ENCLAVE_FILE_ACCESS,
        "Can't open enclave file.",
        NULL
    },
    {
        SGX_ERROR_MODE_INCOMPATIBLE,
        "Target enclave mode is incompatible with the mode of the current RTS",
        NULL
    },
    {
        SGX_ERROR_SERVICE_UNAVAILABLE,
        "sgx_create_enclave() needs the AE service to get a launch token",
        NULL
    },
    {
        SGX_ERROR_SERVICE_TIMEOUT,
        "The request to the AE service timed out",
        NULL
    },
    {
        SGX_ERROR_SERVICE_INVALID_PRIVILEGE,
        "The request requires some special attributes for the enclave, but is not privileged",
        NULL
    },
    {
        SGX_ERROR_NDEBUG_ENCLAVE,
        "The enclave is signed as a product enclave and cannot be created as a debuggable enclave",
        NULL
    },
    {
        SGX_ERROR_UNDEFINED_SYMBOL,
        "The enclave contains an import table",
        NULL
    },
    {
        SGX_ERROR_INVALID_MISC,
        "The MiscSelct/MiscMask settings are not correct",
        NULL
    },
    {
        SGX_ERROR_MAC_MISMATCH,
        "The input MAC does not match the MAC calculated",
        NULL
    }
};


void print_error_message(sgx_status_t ret) {
    size_t idx = 0;
    size_t ttl = sizeof(sgx_errlist)/sizeof (sgx_errlist[0]);

    for (idx = 0; idx < ttl; idx++) {
        if (ret == sgx_errlist[idx].err) {
            if (NULL != sgx_errlist[idx].sug)
                Log("%s", sgx_errlist[idx].sug);

            Log("%s", sgx_errlist[idx].msg);
            break;
        }
    }

    if (idx == ttl)
        Log("Unexpected error occurred");
}


string Base64decode(const string val) {
    return base64_decode(val);
}


string Base64encodeUint8(uint8_t *val, uint32_t len) {
    return base64_encode(val, len);
}

void printf_array(string tag, const uint8_t * arr, int size) {
    ostringstream s;
    for (int i = 0; i < size; i++) {
        s << (int)arr[i] << ", ";
    }
    printf("%s: %s\n", tag.c_str(), s.str().c_str());
}