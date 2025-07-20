/* Licensed to the Apache Software Foundation (ASF) under one
   or more contributor license agreements.  See the NOTICE file
   distributed with this work for additional information
   regarding copyright ownership.  The ASF licenses this file
   to you under the Apache License, Version 2.0 (the
   "License"); you may not use this file except in compliance
   with the License.  You may obtain a copy of the License at

      http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing,
   software distributed under the License is distributed on an
   "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
   KIND, either express or implied.  See the License for the
   specific language governing permissions and limitations
   under the License. */

.file "pic.S"
.section absolute

.global __ImageBase
__ImageBase:

.equ SE_GUARD_PAGE_SHIFT, 16
.equ SE_GUARD_PAGE_SIZE,  (1 << SE_GUARD_PAGE_SHIFT)
.equ RED_ZONE_SIZE,       128
.equ STATIC_STACK_SIZE,   2656

.equ OCMD_ERET,           -1

.equ last_sp,             (SE_WORDSIZE * 1)
.equ stack_base_addr,     (SE_WORDSIZE * 2)
.equ stack_limit_addr,    (SE_WORDSIZE * 3)
.equ first_ssa_gpr,       (SE_WORDSIZE * 4)
.equ xsave_size,          (SE_WORDSIZE * 7)
.equ self_addr,           0
.equ stack_guard,         (SE_WORDSIZE * 5)

/* SSA GPR */
.equ ssa_sp_t,            32
.equ ssa_sp_u,            144
.equ ssa_bp_u,            152
.equ ssa_exit_info,       160

.equ EXIT_INFO_VALID,     0x80000000
/* OCALL command */
.equ OCALL_FLAG,          0x04F434944

.equ RDRAND_RETRY_TIMES,  10

.equ ENCLAVE_INIT_NOT_STARTED,  0
.equ ENCLAVE_INIT_IN_PROGRESS,  1
.equ ENCLAVE_INIT_DONE,         2
.equ ENCLAVE_CRASHED,           3


.section .nipx,"ax",@progbits

/* 
 * ---------------------------------------------------------------------
 *     Get the address of Tds
 * ---------------------------------------------------------------------
 */
DECLARE_LOCAL_FUNC get_tds
    READ_TD_DATA self_addr 
    ret

/* 
 * ---------------------------------------------------------------------
 *     Get the value of stack_guard
 * ---------------------------------------------------------------------
 */
DECLARE_LOCAL_FUNC get_stack_guard 
    READ_TD_DATA stack_guard 
    ret

/* 
 * ---------------------------------------------------------------------
 * Function: enclave_entry
 *      The entry point of the enclave.
 *
 * Registers:
 *      XAX - TCS.CSSA
 *      XBX - the address of a TCS
 *      XCX - the address of the instruction following the EENTER
 *      XDI - the reason of entering the enclave
 *      XSI - the pointer to the marshalling structure
 */
.global _start
DECLARE_GLOBAL_FUNC enclave_entry
/* 
 * ----------------------------------------------------------------------
 * Dispatch code according to CSSA and the reason of EENTER
 *      eax >   0 - exception handler
 *      edi >=  0 - ecall
 *      edi == -1 - do_init_enclave
 *      edi == -2 - oret
 * Registers
 *      No need to use any register during the dispatch
 * ----------------------------------------------------------------------
 */
_start:
    .cfi_startproc

    /* Clear unused general registers */
    xor     %rdx, %rdx
    add     %rdx, %rdx          /* OF = SF = AF = CF = 0; ZF = PF = 1 */
    cld                         /* DF = 0 */
    xor     %r8, %r8
    xor     %r9, %r9
    xor     %r10, %r10
    xor     %r11, %r11
    xor     %r12, %r12
    xor     %r13, %r13
    xor     %r14, %r14
    xor     %r15, %r15

    /* switch to trusted stack */
    cmp     $0, %rax
    jne     .Ldo_handler                /* handle exception state */
    /* xor     %rdx, %rdx               rdx is cssa, make sure it is 0 */
    READ_TD_DATA last_sp
    cmp     $0, %rax
    jne .Lswitch_stack
    GET_STACK_BASE  %rbx
    sub     $STATIC_STACK_SIZE, %rax    /* give space for static stack */
.Lswitch_stack:
    xchg    %rsp, %rax
    push    %rcx
    push    %rbp

    .cfi_def_cfa_offset   2 * SE_WORDSIZE
    .cfi_offset           rbp, -2 * SE_WORDSIZE
    mov     %rsp, %rbp
    .cfi_def_cfa_register rbp

    CLEAN_XFLAGS

    /* Save the registers */
    sub     $(6*SE_WORDSIZE), %rsp
    mov     %rax, -1*SE_WORDSIZE(%rbp)  /* rsp_u */
    mov     %rdx, -3*SE_WORDSIZE(%rbp)  /* cssa */
    mov     %rbx, -4*SE_WORDSIZE(%rbp)  /* TCS */
    mov     %rsi, -5*SE_WORDSIZE(%rbp)  /* rsi */
    mov     %rdi, -6*SE_WORDSIZE(%rbp)  /* rdi */

    /* clean extended feature registers */
    sub     $(4*SE_WORDSIZE), %rsp

    call    get_synthetic_state_ptr     /* lea_pic SYNTHETIC_STATE, %rdi */
    mov     %rax, %rdi
    call    restore_xregs
    lfence
    add     $(4*SE_WORDSIZE), %rsp

    /* switch to C code */
    mov     -6*SE_WORDSIZE(%rbp), %rdi  /* index */
    mov     -5*SE_WORDSIZE(%rbp), %rsi  /* ms */
    mov     -4*SE_WORDSIZE(%rbp), %rdx  /* TCS */
    mov     -3*SE_WORDSIZE(%rbp), %rcx  /* cssa */
    call    enter_enclave
    mov     %rax, %rbx

.Lexit_enclave:
/* clean extended feature registers */
    call    get_synthetic_state_ptr     /* lea_pic SYNTHETIC_STATE, %rdi */
    mov     %rax, %rdi
    call    restore_xregs
    lfence

/* set xdi and xsi */
    mov     $OCMD_ERET, %rdi
    mov     %rbx, %rsi

/* restore stack */
    mov     -1*SE_WORDSIZE(%rbp), %rdx  /* rdx: rsp_u  */
    mov     %rbp, %rsp
    pop     %rbp                        /* rbp_u */
    pop     %rbx                        /* ret_u */
    mov     %rdx, %rsp                  /* rsp_u */

.Lclear_and_exit_enclave:
    /* Clear all GPRs, except rax, rbx, rdi and rsi */
    xor     %rcx, %rcx
    xor     %rdx, %rdx
    xor     %r8, %r8
    xor     %r9, %r9
    xor     %r10, %r10
    xor     %r11, %r11
    xor     %r12, %r12
    xor     %r13, %r13
    xor     %r14, %r14
    xor     %r15, %r15

    /* Set status flags to pre-defined values */
    add     %rdx, %rdx          /* OF = SF = AF = CF = 0; ZF = PF = 1 */
    cld                         /* DF = 0 */

    /* EEXIT */
    mov     $SE_EEXIT, %rax     /* EEXIT leaf */
    ENCLU

    /* Should not come here */
    ud2

.Ldo_handler:
    mov     %rax, %rdx          /* rdx: cssa */
    GET_STACK_BASE %rbx
    jmp     .Lswitch_stack   
 
    /* Should not come here */
    ud2
    
    .cfi_endproc

/* 
 * -------------------------------------------------------------------------
 *  SgxStatus __do_ocall(unsigned int index, void *ms);
 *
 *  Function: __do_ocall
 *        The entry point of the enclave
 *  Parameters:
 *        func_addr - target function address
 *        ms - marshalling structure
 *
 *  Stack: (same as do_oret)
 *        bottom of stack ->
 *                            -----------------
 *                           | ECALL/OCALL     |
 *    previous TD.last_sp -> |   frames        |
 *                            -----------------
 *                           |   ECALL frame   |
 *                           | do_ocall param 2| 3
 *                           | do_ocall param 1| 2
 *                           |do_ocall ret_addr| 1
 *                           |     xbp         | 0 + xbp
 *                           |     ....        |
 *                           |   xsave buffer  |
 *                           |     ....        |
 *                           |  xsave pointer  | 19
 *                           |   ocall_depth   | 18
 *                           |   reserved      | 17
 *                           |   reserved      | 16
 *                           |   reserved      | 15
 *                           |     rbx         | 14
 *                           |     rsi         | 13
 *                           |     rdi         | 12
 *                           |     rbp         | 11
 *                           |     r12         | 10
 *                           |     r13         | 9
 *                           |     r14         | 8
 *                           |     r15         | 7
 *                           | prev TD.last_sp | 6
 *                           |  ocall_index    | 5
 *                           |   OCALL FLAG    | 4
 *                           |    shadow       | 3
 *                           |    shadow       | 2
 *                           |    shadow       | 1
 *             TD.last_sp -> |    shadow       | 0 + xsp
 *                            -----------------
 * -------------------------------------------------------------------------
 */
DECLARE_LOCAL_FUNC __do_ocall

/* 
 * 8 for GPR, 1 for TD.last_sp, 1 for ocall_index
 * 1 for OCALL_FLAG, 4 for shadow space.
 * Stack Pointer is 16-byte aligned under x86_64.
 */
    push    %rbp
    mov     %rsp, %rbp

/* save parameters in stack */
    mov     %rdi, 2*SE_WORDSIZE(%rbp)
    mov     %rsi, 3*SE_WORDSIZE(%rbp)

/* save and clean extended feature registers */
    READ_TD_DATA xsave_size
    sub     %rax, %rsp                 /* allocate buffer to save xregs */
    mov     $0x3f, %rax
    not     %rax
    and     %rax, %rsp                 /* xsave requires 64 byte aligned */
    mov     %rsp, %rcx                 # xsave pointer

    sub     $(20*SE_WORDSIZE), %rsp    /* 20 slots for GPRs and other info */
    mov     %rcx, SE_WORDSIZE*19(%rsp) /* addr for xsave */
/* save non-volatile registers, except xsp */
    mov     %rbx, SE_WORDSIZE*14(%rsp)
    mov     %rsi, SE_WORDSIZE*13(%rsp)
    mov     %rdi, SE_WORDSIZE*12(%rsp)
    mov     %rbp, SE_WORDSIZE*11(%rsp)
    mov     %r12, SE_WORDSIZE*10(%rsp)
    mov     %r13, SE_WORDSIZE* 9(%rsp)
    mov     %r14, SE_WORDSIZE* 8(%rsp)
    mov     %r15, SE_WORDSIZE* 7(%rsp)

/* save and clean extended feature registers */
    mov     SE_WORDSIZE*19(%rsp), %rdi /* xsave pointer */
    READ_TD_DATA xsave_size
    mov     %rax, %rcx
    shr     $2, %rcx                   /* xsave size in dword */
    xor     %rax, %rax
    cld
    rep stos %eax, %es:(%rdi)

    mov     SE_WORDSIZE*19(%rsp), %rdi # xsave pointer
    mov     %rdi, (%rsp)
    call    save_xregs
    call    get_synthetic_state_ptr    /* lea_pic SYNTHETIC_STATE, %rdi */
    mov     %rax, %rdi
    mov     %rdi, (%rsp)
    call    restore_xregs
    lfence

    /* set rdi and rsi using the input parameters */
    mov     SE_WORDSIZE*12(%rsp), %rdi
    mov     SE_WORDSIZE*13(%rsp), %rsi

    /* save ocall index to the stack */
    mov     $OCALL_FLAG, %rax
    mov     %rax, SE_WORDSIZE*4(%rsp)   /* save OCALL_FLAG */
    mov     %rdi, SE_WORDSIZE*5(%rsp)   /* save ocall_index */

    /*
     * save the inside stack context
     *     push TD.last_sp
     *     set TD.last_sp = xsp
     */
    READ_TD_DATA self_addr
    mov     %rax, %rbx 

    /* call update_ocall_lastsp */
    mov     %rsp, %rdi
    
    call    update_ocall_lastsp          /* rax: tds.last_sp */

    mov     SE_WORDSIZE*12(%rsp), %rdi   /* restore rdi */
    mov     SE_WORDSIZE*13(%rsp), %rsi   /* restore rdi */

    /* restore outside stack context */
    mov     first_ssa_gpr(%rbx), %rdx
    mov     ssa_bp_u(%rdx), %rbp
    mov     ssa_sp_u(%rdx), %rsp
    /*
     * set EEXIT registers
     * return address can be read from the ECALL frame:
     *       TD.last_sp ->
     *                     -------------
     *                    | ret_addr    |
     *                    | xbp_u       |
     *                    | xsp_u       |
     *                    | ...         |
     */
    mov     -1*SE_WORDSIZE(%rax), %rbx  /* return address */
    mov     $SE_EEXIT, %rax             /* EEXIT leaf */

    /* Clear all GPRs, except rax, rbx, rdi, and rsi*/
    xor     %rcx, %rcx
    xor     %rdx, %rdx
    xor     %r8,  %r8
    xor     %r9,  %r9
    xor     %r10, %r10
    xor     %r11, %r11
    xor     %r12, %r12
    xor     %r13, %r13
    xor     %r14, %r14
    xor     %r15, %r15

    /* Set status flags to pre-defined values */
    add     %rdx, %rdx          /* OF = SF = AF = CF = 0; ZF = PF = 1 */
    cld                         /* DF = 0 */

    ENCLU

/*
 * ------------------------------------------------------------------
 * this function is the wrapper of do_ocall, which is used to
 * stick ocall bridge and proxy frame together
 * ------------------------------------------------------------------
 */
DECLARE_LOCAL_FUNC do_ocall
    .cfi_startproc
    push %rbp
    .cfi_def_cfa_offset     2*SE_WORDSIZE
    .cfi_offset             rbp,-2*SE_WORDSIZE
    mov %rsp, %rbp
    .cfi_def_cfa_register   rbp
    sub $(4*SE_WORDSIZE), %rsp

    call        __do_ocall
    leave
    ret
    .cfi_endproc

DECLARE_GLOBAL_FUNC asm_oret
    mov     %rsp, %rbx
    mov     %rdi, SE_WORDSIZE(%rsp)
    mov     %rsi, 2*SE_WORDSIZE(%rsp)
    mov     SE_WORDSIZE(%rbx), %rsp    /* restore thread_data.last_sp */

/* restore extended feature registers */
    mov     19*SE_WORDSIZE(%rsp), %rdi

    call    restore_xregs

/* memset_s */
    xor     %rax, %rax
    mov     11*SE_WORDSIZE(%rsp), %rcx
    sub     %rdi, %rcx
    sub     $SE_WORDSIZE, %rcx
    shr     $2, %rcx
    cld
    rep stos %eax,%es:(%rdi)

    mov     2*SE_WORDSIZE(%rbx), %rax  /* ocall return value */

    mov     7*SE_WORDSIZE(%rsp), %r15
    mov     8*SE_WORDSIZE(%rsp), %r14
    mov     9*SE_WORDSIZE(%rsp), %r13
    mov    10*SE_WORDSIZE(%rsp), %r12
    mov    11*SE_WORDSIZE(%rsp), %rbp
    mov    12*SE_WORDSIZE(%rsp), %rdi
    mov    13*SE_WORDSIZE(%rsp), %rsi
    mov    14*SE_WORDSIZE(%rsp), %rbx

    mov     %rbp, %rsp
    pop     %rbp

    ret
    /* should not come here */
    ud2

/* 
 * -------------------------------------------------------------------------
 * do_ereport(target_info, report_data, report);
 * EREPORT: rbx - the address of TARGETINFO;
 *          rcx - the address of REPORTDATA;
 *          rdx - the address where REPORT is outputted
 * return value:
 *          zero: success
 *          non-zero: failure
 * -------------------------------------------------------------------------
 */
 /*
.global Lereport_inst
DECLARE_LOCAL_FUNC do_ereport
    SE_PROLOG
    mov       $SE_EREPORT, %rax
    clc
Lereport_inst:
    ENCLU
    setc      %al
    SE_EPILOG
*/

/*
 * -------------------------------------------------------------------------
 * extern "C" void abort(void) __attribute__(__noreturn__);
 * -------------------------------------------------------------------------
 */
/*
DECLARE_GLOBAL_FUNC abort
    lea_pic ENCLAVE_STATE, %rax
    movl    $ENCLAVE_CRASHED, (%rax)
    ud2
*/
/* 
 * -------------------------------------------------------------------------
 * extern "C" __attribute__((regparm(1))) void continue_execution(sgx_exception_info_t *info);
 * -------------------------------------------------------------------------
 */
DECLARE_LOCAL_FUNC continue_execution
    mov     %rdi, %rcx
    mov     SE_WORDSIZE*0(%rcx), %rax
    push    %rax                       /* push rax */
    mov     SE_WORDSIZE*1(%rcx), %rax
    push    %rax                       /* push xcx */
    mov     SE_WORDSIZE*4(%rcx), %rax  /* xax: xsp */
/* x86_64 requires a 128-bytes red zone. We need to allocate buffer to avoid touching the red zone. */
    sub     $(SE_WORDSIZE + RED_ZONE_SIZE), %rax   /* allocate buffer to skip red zone and save xip */

/* restore registers except xax, xcx, xsp */
    mov     SE_WORDSIZE*2(%rcx), %rdx
    mov     SE_WORDSIZE*3(%rcx), %rbx
    mov     SE_WORDSIZE*5(%rcx), %rbp
    mov     SE_WORDSIZE*6(%rcx), %rsi
    mov     SE_WORDSIZE*7(%rcx), %rdi
    mov     SE_WORDSIZE*8(%rcx), %r8
    mov     SE_WORDSIZE*9(%rcx), %r9
    mov     SE_WORDSIZE*10(%rcx), %r10
    mov     SE_WORDSIZE*11(%rcx), %r11
    mov     SE_WORDSIZE*12(%rcx), %r12
    mov     SE_WORDSIZE*13(%rcx), %r13
    mov     SE_WORDSIZE*14(%rcx), %r14
    mov     SE_WORDSIZE*15(%rcx), %r15
    push    SE_WORDSIZE*16(%rcx)
    popf    /* make sure the following instructions do not affect flags */

    mov     SE_WORDSIZE*17(%rcx), %rcx

/* do not setup the new stack until info is not needed any more
 * otherwise, info will be overwritten
 */
    mov     %rcx, (%rax)               /* save xip to the new stack */
    pop     %rcx                       /* restore xcx */
    pop     %rsp                       /* xsp: xax */
    xchg    %rax, %rsp
    ret     $(RED_ZONE_SIZE)           /* pop xip and red zone (if any) */

/*
DECLARE_GLOBAL_FUNC lfence
    pop %r11
    lfence
    jmp *%r11

DECLARE_GLOBAL_FUNC mfence
    pop %r11
    mfence
    jmp *%r11
*/
