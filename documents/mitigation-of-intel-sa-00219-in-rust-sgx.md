---
permalink: /sgx-sdk-docs/mitigation-of-intel-sa-00219
---
# Background

Intel issued [Intel SA-00219](https://www.intel.com/content/www/us/en/security-center/advisory/intel-sa-00219.html) on Nov 12, 2019, with CVE number [CVE-2019-0117](http://cve.mitre.org/cgi-bin/cvename.cgi?name=CVE-2019-0117). Intel also published a [guidance](https://software.intel.com/en-us/download/intel-sgx-sdk-developer-guidance-intel-sa-00219) to instruct the developers/researchers. Then Intel released [Intel SGX SDK v2.7.1](https://01.org/intel-softwareguard-extensions/downloads/intel-sgx-linux-2.7.1-release-version-string-2.7.101.3), including new memory allocation primitives and corresponding patches in PSW enclaves.

This article is to help people understand Intel-SA-00219, and how Rust SGX SDK handles it. Please feel free to reach me at dingelish@gmail.com, or dingyu@apache.org.

## The problem statement and my thoughts

The only statement I found is on the [Intel-SA-00219 page](https://www.intel.com/content/www/us/en/security-center/advisory/intel-sa-00219.html):

> Organize the code/data within enclave memory to avoid putting sensitive materials in DWORD0 and DWORD1 of cache line. The effectiveness of this mitigation is dependent on the ability for the software to avoid the affected memory region. To assist the enclave application providers to modify their code, Intel is releasing SGX SDK update (Windows version 2.5.101.3, Linux version 2.7.101.3) with new memory allocation APIs to avoid the affected memory region. More details about the APIs can be found [here](https://software.intel.com/en-us/download/intel-sgx-sdk-developer-guidance-intel-sa-00219).

Intel does not directly describe the vulnerability here. But it's clear that the 64-byte cache line would contain 8-byte or sensitive data, which can be keys protected by Intel SGX. So the following memory layout can be problematic in SGX:

```
 --------------------------------------------------------------------------------------
| attacker accessible data A | private key (inaccessible) | attacker accessible data B |
 --------------------------------------------------------------------------------------
```

It's equal to a vulnerable data structure like:

```
struct foo {
    uint64_t A;
    uint64_t secret;
    uint64_t B;
}
```

where `foo.A` and `foo.B` are accessible by design, while `foo.secret` is not.

If an attacker somehow can access either A or B, he probably will have first or last 8-byte of the "inaccessible" secret in cache line. Then something bad may happen.

So, the most straightforward mitigation is to insert additional "guard bytes" before and after the sensitive data:

```
 ----------------------------------------------------------------------------------------------
| attacker data A | 8-byte guard | private key (inaccessible) | 8-byte guard | attacker data B |
 ----------------------------------------------------------------------------------------------
```

It results in a modified structure like

```
struct foo {
    uint64_t A;
    (private) uint64_t _guard0;
    uint64_t secret;
    (private) uint64_t _guard1;
    uint64_t B;
}
```

Further investigation from Intel's code reveals that `_guard1` is not required. So it can be:

```
     -------------------------------------------------------------------------------
    | attacker data A | 8-byte guard | private key (inaccessible) | attacker data B |
     -------------------------------------------------------------------------------
```

## Intel's new allocator primitive

Intel's guidance provides:

(1) A C++ template `custom_alignment_aligned`
(2) A C function `sgx_get_aligned_ptr` and one of its parameter's type `struct align_req_t`
(3) A dynamic memory allocator function `sgx_aligned_malloc`

After spending hours on Intel's code, I realized that these primitives are helping developers allocate a larger object which:

a) contains all fields of the original object.
b) adds "guard bytes" before and after each "specified secret field".
c) align each "specified secret field" on demand

## Intel's patches on PSW enclaves

The most easy to understand example is from `psw/ae/pse_op/session_mgr.cpp`:

```diff
@@ -417,7 +461,12 @@ pse_op_error_t pse_exchange_report(uint64_t tick,
 {
     pse_op_error_t status = OP_SUCCESS;
     sgx_dh_session_t sgx_dh_session;
-    sgx_key_128bit_t aek;
+    //
+    // securely align aek
+    //
+    //sgx_key_128bit_t aek;
+    sgx::custom_alignment_aligned<sgx_key_128bit_t, sizeof(sgx_key_128bit_t), 0, sizeof(sgx_key_128bit_t)> oaek;
+    sgx_key_128bit_t& aek = oaek.v;
     sgx_dh_session_enclave_identity_t initiator_identity;
     cse_sec_prop_t * pcse_sec = NULL;
     secu_info_t* psec_info = NULL;
```

The template generates a larger struct `oaek`. Size of `sgx_key_128bit_t` is 16 bytes, and `sizeof(oaek)` equals to 32. And the offset of `oaek.v` is 8.

And in the same file, another fix is:

```diff
--- a/psw/ae/pse/pse_op/session_mgr.cpp
+++ b/psw/ae/pse/pse_op/session_mgr.cpp
@@ -29,21 +29,65 @@
  *
  */

-
+#include <sgx_secure_align.h>
 #include "utility.h"
 #include "session_mgr.h"
 #include "pse_op_t.h"
 #include "sgx_dh.h"

 // ISV enclave <-> pse-op sessions
-static pse_session_t        g_session[SESSION_CONNECTION];
+//
+// securely align all ISV enclave - pse sessions' secrets
+//
+static sgx::custom_alignment_aligned<pse_session_t, 16, __builtin_offsetof(pse_session_t, active.AEK), 16> og_session[SESSION_CONNECTION];
+//
+// following allows existing references to g_session[index]
+// to not have to change
+//
+class CSessions
+{
+public:
+    pse_session_t& operator[](int index) {
+        return og_session[index].v;
+    }
+};
+static CSessions g_session;
 static uint32_t             g_session_count = 0;
```

It seems that the original global `g_session` array is vulnerabile to INTEL-SA-00219. So Intel created a new structure `CSessions` and reloaded the `[]` operator, and used `custom_alignment_aligned` template to create the array of guarded `CSessions`.

We can see some more complex samples in the same file, such as:

```diff
 // ephemeral session global variables
 static uint8_t              g_nonce_r_pse[EPH_SESSION_NONCE_SIZE] = {0};      // nonce R(PSE) for ephemeral session establishment
 static uint8_t              g_nonce_r_cse[EPH_SESSION_NONCE_SIZE] = {0};      // nonce R(CSE) for ephemeral session establishment
-static pairing_data_t       g_pairing_data;                       // unsealed pairing data
-eph_session_t               g_eph_session;                        // ephemeral session information
+
+//
+// securely align pairing data
+// Id_pse and Id_cse aren't secrets
+// I don't think pairingNonce is a secret and even if it is, we can't align
+// all of [mk, sk, pairingID, pairingNonce]
+//
+//static pairing_data_t       g_pairing_data;                       // unsealed pairing data
+static sgx::custom_alignment<pairing_data_t,
+    //__builtin_offsetof(pairing_data_t, secret_data.Id_pse), sizeof(((pairing_data_t*)0)->secret_data.Id_pse),
+    //__builtin_offsetof(pairing_data_t, secret_data.Id_cse), sizeof(((pairing_data_t*)0)->secret_data.Id_cse),
+    __builtin_offsetof(pairing_data_t, secret_data.mk), sizeof(((pairing_data_t*)0)->secret_data.mk),
+    __builtin_offsetof(pairing_data_t, secret_data.sk), sizeof(((pairing_data_t*)0)->secret_data.sk),
+    __builtin_offsetof(pairing_data_t, secret_data.pairingID), sizeof(((pairing_data_t*)0)->secret_data.pairingID)
+    //__builtin_offsetof(pairing_data_t, secret_data.pairingNonce), sizeof(((pairing_data_t*)0)->secret_data.pairingNonce)
+    > opairing_data;
+pairing_data_t& g_pairing_data = opairing_data.v;
+//
+// securely align pse - cse/psda ephemeral session secrets
+//
+//eph_session_t               g_eph_session;                        // ephemeral session information
+sgx::custom_alignment<eph_session_t,
+    __builtin_offsetof(eph_session_t, TSK), sizeof(((eph_session_t*)0)->TSK),
+    __builtin_offsetof(eph_session_t, TMK), sizeof(((eph_session_t*)0)->TMK)
+> oeph_session;
+//
+// this reference trick requires change to declaration
+// in other files, but still cleaner than changing
+// all references
+//
+eph_session_t& g_eph_session = oeph_session.v;

 /**
  * @brief Check the status of the ephemeral session
```

To understand it, let me expand `struct pairing_data_t` here:

```
/* Pairing blob unsealed and usable inside of enclave*/
typedef struct _pairing_data_t
{
    se_plaintext_pairing_data_t plaintext; // does not involved
    struct se_secret_pairing_data_t {
            SHA256_HASH         Id_pse;
            SHA256_HASH         Id_cse;
            SIGMA_MAC_KEY       mk;
            SIGMA_SECRET_KEY    sk;
            SIGMA_SECRET_KEY    pairingID;  // old_sk used for repairing check
            Nonce128_t          pairingNonce;
            EcDsaPrivKey        VerifierPrivateKey;
    } secret_data;
} pairing_data_t;
```

The patch seems to protect `mk`, `sk`, and `pairingID`, and all the other fields are commented out. What's more, this patch uses a **undocumented** template `sgx::custom_alignment` defined as:

```cpp
    template <class T, std::size_t... OLs>
    using custom_alignment = custom_alignment_aligned<T, alignof(T), OLs...>;
```

## Experiments on the undocument template

To test how the undocumented template work, I write the following codes:

```cpp
    struct foo {
        uint64_t secret1[5];       // offset = 0
    };

    typedef sgx::custom_alignment<foo, __builtin_offsetof(foo, secret1), sizeof(((foo*)0)->secret1)> AFOO;

    printf("=== Size of foo = %u ===\n", sizeof(foo));                               // 40
    printf("=== Size of bar = %u ===\n", sizeof(AFOO));                              // 64
    printf("=== offset of AROO.v = %u ===\n", __builtin_offsetof(AFOO, v));          // 8
    printf("=== offset of secret1 = %u ===\n", __builtin_offsetof(AFOO, v.secret1)); // 8
```

So we can see that the structure of AROO is:

```cpp
struct AROO {
    uint64_t _padding_head[1]         // offset = 0, len = 8
    struct {
        uint64_t secret1[5];          // offset = 8, len = 40
    } v;
    uint64_t _padding_tail[2];        // offset = 40, len = 16
```

It seems the undocumented C++ template aligns `AROO` to the next level, and add 8-byte headings into it. If we add the second secret in `foo` like:

```cpp
    struct foo {
        uint64_t secret1[5];       // offset = 0
        uint64_t secret2[1];       // offset = 40
    };

    typedef sgx::custom_alignment<foo,
                __builtin_offsetof(foo, secret1), sizeof(((foo*)0)->secret1),
                __builtin_offsetof(foo, secret2), sizeof(((foo*)0)->secret2)
            > AFOO;

    printf("=== Size of foo = %u ===\n", sizeof(foo));            // 48
    printf("=== Size of bar = %u ===\n", sizeof(AFOO));           // 64
    printf("=== offset of AROO.v = %u ===\n", __builtin_offsetof(AFOO, v));           // 8
    printf("=== offset of AROO.v.secret1 = %u ===\n", __builtin_offsetof(AFOO, v.secret1));           // 8
    printf("=== offset of AROO.v.secret2 = %u ===\n", __builtin_offsetof(AFOO, v.secret2));           // 48
```

we can see that the structure of AROO is:

```cpp
struct AROO {
    uint64_t _padding_head[1]         // offset = 0, len = 8
    struct {
        uint64_t secret1[5];          // offset = 8, len = 40
        uint64_t secret2[1];          // offset = 48, len = 8
    } v;
    uint64_t _padding_tail[1];        // offset = 56, len = 8
```

If we increase `secret2` to 16-bytes, it works well as usual. And the `_padding_tail` will have **zero length**. So does it means that *only extra heading is required for mitigation*? But it'll not compile if we make `secret2` 24-bytes, like:

```c++
    struct foo {
        uint64_t secret1[5];       // offset = 0
        uint64_t secret2[3];       // offset = 40
    };

    typedef sgx::custom_alignment<foo,
                __builtin_offsetof(foo, secret1), sizeof(((foo*)0)->secret1),
                __builtin_offsetof(foo, secret2), sizeof(((foo*)0)->secret2)
            > AFOO;
```

GCC would terminate on:

```
make[1]: Entering directory '/root/linux-sgx/SampleCode/Cxx11SGXDemo'
In file included from Enclave/TrustedLibrary/Libcxx.cpp:47:0:
/opt/sgxsdk/include/sgx_secure_align.h: In instantiation of 'struct sgx::__custom_alignment_internal::custom_alignment<ecall_lambdas_demo()::foo, 8ul, -1>':
Enclave/TrustedLibrary/Libcxx.cpp:125:53:   required from here
/opt/sgxsdk/include/sgx_secure_align.h:123:13: error: static assertion failed: No viable offset
             static_assert(LZ > 0, "No viable offset");
             ^
/opt/sgxsdk/include/sgx_secure_align.h:125:48: error: size of array is negative
             char __no_secret_allowed_in_here[LZ];
                                                ^
Makefile:255: recipe for target 'Enclave/TrustedLibrary/Libcxx.o' failed
```

Nothing changes if we switch to the original template `sgx::custom_alignment_aligned`. So I guess the template does not support structures:

- contains secrets consecutively,  and
- the consecutive secrets' size is larger than a certain number (not sure yet)

If we break down `secret1` and `secret2` by inserting something in the middle, the template works:

```c++
struct foo {
  uint64_t secret1[5];       // offset = 0
  char     dumb;             // offset = 40
  uint64_t secret2[3];       // offset = 48
};

typedef sgx::custom_alignment<foo,
__builtin_offsetof(foo, secret1), sizeof(((foo*)0)->secret1),
__builtin_offsetof(foo, secret2), sizeof(((foo*)0)->secret2)
  > AFOO;

printf("=== Size of foo = %u ===\n", sizeof(foo));            // 72
printf("=== Size of bar = %u ===\n", sizeof(AFOO));           // 128
printf("=== offset of AROO.v = %u ===\n", __builtin_offsetof(AFOO, v));           // 24
printf("=== offset of AROO.v.secret1 = %u ===\n", __builtin_offsetof(AFOO, v.secret1));           // 24
printf("=== offset of AROO.v.secret2 = %u ===\n", __builtin_offsetof(AFOO, v.secret2));           // 72
```

## Changes/Actions required

From Intel's usage, we can learn that:

**Don't construct a sensitive data structure directly. Always allocate an aligned structure and fill it up later **

It means:

* if you allocate something sensitive (e.g. keys in `sgx_key_128bit_t`) on stack/heap, you probably need to allocate another guarded structure first, and get a mutable reference to its inner data.
* if you want to make `sgx_key_128bit_t` as the type of return value, you can choose between (1) return a guarded structure, or (2) takes an additional argument of caller-allocated, mutuable reference of `sgx_key_128bit_t` and fill it.

## Rust SGX provided primitive

* We provided `AlignBox` as a replacement of `Box`

  * `Box` is somewhat tricky because it always "initialize on stack first and copy to heap later". [copyless](https://github.com/kvark/copyless) provides a novel primitive to solve [it but it does not always effective](https://github.com/dingelish/realbox). To this end, we created `AlignBox` which guarantees "on-heap initialization" without copying any bits. Usage:

    ```rust
    let heap_align_obj = AlignBox::<struct_align_t>::heap_init_with_req(|mut t| {
      t.key1 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
      t.pad1 = [0x00; 16];
      t.key2 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
      t.pad2 = [0x00; 16];
      t.key3 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
      t.pad3 = [0x00; 16];
      t.key4 = [0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff];
      }, 16, &str_slice);
    assert!(heap_align_obj.is_some());
    ```

* We provided aligned key type for each built-in key type. The layout are calculated by Intel's template.

  * `sgx_align_key_128bit_t`
  * `sgx_align_mac_128bit_t`
  * `sgx_align_key_256bit_t`
  * `sgx_align_mac_256bit_t`
  * `sgx_align_ec256_dh_shared_t`
  * `sgx_align_ec256_private_t`

We modified `sgx_tcrypto`, `sgx_tse`, and `sgx_tdh` and use the above primitives for enhancement, following the above required changes. One sample is from `sgx_tcrypto`:

```rust
+    let mut align_mac = sgx_align_mac_128bit_t::default();
+    let ret = unsafe {
+        sgx_rijndael128_cmac_msg(key as * const sgx_cmac_128bit_key_t,
+                                 src.as_ptr() as * const u8,
+                                 size as u32,
+                                 &mut align_mac.mac as * mut sgx_cmac_128bit_tag_t)
+    };
```

We allocate an aligned structure first, and then fill it up using Intel's crypto primitive later.
