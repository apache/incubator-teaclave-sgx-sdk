#ifndef SHA256_H
#define SHA256_H

#include <stdio.h>
#include <stdlib.h>
#include <stddef.h>

#include "LogBase.h"
#include "sample_libcrypto.h"

using namespace util;

class Sha256 {

public:
    Sha256();
    ~Sha256();

    int update(uint8_t* data, uint32_t size);
    int hash(sample_sha256_hash_t* report_data);

private:
    sample_sha_state_handle_t sha_handle;
};


#endif//SHA256_H