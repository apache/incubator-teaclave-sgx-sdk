#include "sha256.h"

Sha256::Sha256() {
    sample_status_t sample_ret = SAMPLE_SUCCESS;

    // Verify the report_data in the Quote matches the expected value.
    // The first 32 bytes of report_data are SHA256 HASH of {ga|gb|vk}.
    // The second 32 bytes of report_data are set to zero.
    sample_ret = sample_sha256_init(&sha_handle);
    if (sample_ret != SAMPLE_SUCCESS) {
        Log("Error, init hash failed", log::error);
    }
}

Sha256::~Sha256() {
    sample_sha256_close(sha_handle);
    sha_handle = NULL;
}

int Sha256::update(uint8_t* data, uint32_t size) {
    sample_status_t sample_ret = SAMPLE_SUCCESS;

    sample_ret = sample_sha256_update(data, size, sha_handle);
    if (sample_ret != SAMPLE_SUCCESS) {
        Log("Error, udpate hash failed", log::error);
        return -1;
    }

    return 0;
}

int Sha256::hash(sample_sha256_hash_t* report_data) {
    sample_status_t sample_ret = SAMPLE_SUCCESS;

    sample_ret = sample_sha256_get_hash(sha_handle, report_data);
    if (sample_ret != SAMPLE_SUCCESS) {
        Log("Error, Get hash failed", log::error);
        return -1;
    }

    return 0;
}
