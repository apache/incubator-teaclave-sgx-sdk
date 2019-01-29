#include <stdint.h>
#include <stdlib.h>

void
aesni_setup_round_key_128(uint8_t* key, uint8_t* round_key)
{
    #ifdef __SSE__
    asm volatile(
        " \
            movdqu (%1), %%xmm1; \
            movdqu %%xmm1, (%0); \
            add $0x10, %0; \
            \
            aeskeygenassist $0x01, %%xmm1, %%xmm2; \
            call 1f; \
            aeskeygenassist $0x02, %%xmm1, %%xmm2; \
            call 1f; \
            aeskeygenassist $0x04, %%xmm1, %%xmm2; \
            call 1f; \
            aeskeygenassist $0x08, %%xmm1, %%xmm2; \
            call 1f; \
            aeskeygenassist $0x10, %%xmm1, %%xmm2; \
            call 1f; \
            aeskeygenassist $0x20, %%xmm1, %%xmm2; \
            call 1f; \
            aeskeygenassist $0x40, %%xmm1, %%xmm2; \
            call 1f; \
            aeskeygenassist $0x80, %%xmm1, %%xmm2; \
            call 1f; \
            aeskeygenassist $0x1b, %%xmm1, %%xmm2; \
            call 1f; \
            aeskeygenassist $0x36, %%xmm1, %%xmm2; \
            call 1f; \
            \
            jmp 2f; \
            \
            1: \
            pshufd $0xff, %%xmm2, %%xmm2; \
            vpslldq $0x04, %%xmm1, %%xmm3; \
            pxor %%xmm3, %%xmm1; \
            vpslldq $0x4, %%xmm1, %%xmm3; \
            pxor %%xmm3, %%xmm1; \
            vpslldq $0x04, %%xmm1, %%xmm3; \
            pxor %%xmm3, %%xmm1; \
            pxor %%xmm2, %%xmm1; \
            movdqu %%xmm1, (%0); \
            add $0x10, %0; \
            ret; \
            \
            2: \
        "
    : "+r" (round_key)
    : "r" (key)
    : "xmm1", "xmm2", "xmm3", "memory"
    );
    #else
    exit(1);
    #endif
}

void
aesni_encrypt_block(uint8_t rounds, uint8_t* input, uint8_t* round_keys, uint8_t* output)
{
    #ifdef __SSE__
    asm volatile(
    " \
        /* Copy the data to encrypt to xmm1 */ \
        movdqu (%2), %%xmm1; \
        \
        /* Perform round 0 - the whitening step */ \
        movdqu (%1), %%xmm0; \
        add $0x10, %1; \
        pxor %%xmm0, %%xmm1; \
        \
        /* Perform all remaining rounds (except the final one) */ \
        1: \
        movdqu (%1), %%xmm0; \
        add $0x10, %1; \
        aesenc %%xmm0, %%xmm1; \
        sub $0x01, %0; \
        cmp $0x01, %0; \
        jne 1b; \
        \
        /* Perform the last round */ \
        movdqu (%1), %%xmm0; \
        aesenclast %%xmm0, %%xmm1; \
        \
        /* Finally, move the result from xmm1 to outp */ \
        movdqu %%xmm1, (%3); \
    "
    : "+&r" (rounds), "+&r" (round_keys) // outputs
    : "r" (input), "r" (output) // inputs
    : "xmm0", "xmm1", "memory", "cc" // clobbers
    );
    #else
    exit(1);
    #endif
}
