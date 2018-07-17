#if defined(__x86_64__)
.text	
.extern	GFp_ia32cap_P
.hidden GFp_ia32cap_P


.align	64
.Lord:
.quad	0xf3b9cac2fc632551, 0xbce6faada7179e84, 0xffffffffffffffff, 0xffffffff00000000
.LordK:
.quad	0xccd1c8aaee00bc4f






.globl	GFp_p256_scalar_mul_mont
.hidden GFp_p256_scalar_mul_mont
.type	GFp_p256_scalar_mul_mont,@function
.align	32
GFp_p256_scalar_mul_mont:
	pushq	%rbp
	pushq	%rbx
	pushq	%r12
	pushq	%r13

	movq	%rdx,%rbx

	movq	0(%rbx),%rcx
	movq	0(%rsi),%rax
	mulq	%rcx
	movq	%rax,%r8
	movq	%rdx,%r9

	movq	8(%rsi),%rax
	mulq	%rcx
	addq	%rax,%r9
	adcq	$0,%rdx
	movq	%rdx,%r10

	movq	16(%rsi),%rax
	mulq	%rcx
	addq	%rax,%r10
	adcq	$0,%rdx
	movq	%rdx,%r11

	movq	24(%rsi),%rax
	mulq	%rcx
	addq	%rax,%r11
	adcq	$0,%rdx
	movq	%rdx,%r12
	xorq	%r13,%r13


	movq	%r8,%rax
	mulq	.LordK(%rip)
	movq	%rax,%rcx

	movq	0+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rax,%r8
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	8+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rbp,%r9
	adcq	$0,%rdx
	addq	%rax,%r9

	movq	%rcx,%rbp
	adcq	%rdx,%r10
	adcq	$0,%rbp
	subq	%rcx,%r10
	sbbq	$0,%rbp

	movq	24+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rbp,%r11
	adcq	$0,%rdx
	addq	%rax,%r11
	adcq	%rdx,%r12
	adcq	$0,%r13


	movq	8(%rbx),%rcx

	movq	0(%rsi),%rax
	mulq	%rcx
	addq	%rax,%r9
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	8(%rsi),%rax
	mulq	%rcx
	addq	%rbp,%r10
	adcq	$0,%rdx
	addq	%rax,%r10
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	16(%rsi),%rax
	mulq	%rcx
	addq	%rbp,%r11
	adcq	$0,%rdx
	addq	%rax,%r11
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	24(%rsi),%rax
	mulq	%rcx
	addq	%rbp,%r12
	adcq	$0,%rdx
	addq	%rax,%r12
	adcq	%rdx,%r13
	adcq	$0,%r8

	movq	%r9,%rax
	mulq	.LordK(%rip)
	movq	%rax,%rcx

	movq	0+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rax,%r9
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	8+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rbp,%r10
	adcq	$0,%rdx
	addq	%rax,%r10

	movq	%rcx,%rbp
	adcq	%rdx,%r11
	adcq	$0,%rbp
	subq	%rcx,%r11
	sbbq	$0,%rbp

	movq	24+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rbp,%r12
	adcq	$0,%rdx
	addq	%rax,%r12
	adcq	%rdx,%r13
	adcq	$0,%r8

	movq	16(%rbx),%rcx

	movq	0(%rsi),%rax
	mulq	%rcx
	addq	%rax,%r10
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	8(%rsi),%rax
	mulq	%rcx
	addq	%rbp,%r11
	adcq	$0,%rdx
	addq	%rax,%r11
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	16(%rsi),%rax
	mulq	%rcx
	addq	%rbp,%r12
	adcq	$0,%rdx
	addq	%rax,%r12
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	24(%rsi),%rax
	mulq	%rcx
	addq	%rbp,%r13
	adcq	$0,%rdx
	addq	%rax,%r13
	adcq	%rdx,%r8
	adcq	$0,%r9

	movq	%r10,%rax
	mulq	.LordK(%rip)
	movq	%rax,%rcx

	movq	0+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rax,%r10
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	8+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rbp,%r11
	adcq	$0,%rdx
	addq	%rax,%r11

	movq	%rcx,%rbp
	adcq	%rdx,%r12
	adcq	$0,%rbp
	subq	%rcx,%r12
	sbbq	$0,%rbp

	movq	24+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rbp,%r13
	adcq	$0,%rdx
	addq	%rax,%r13
	adcq	%rdx,%r8
	adcq	$0,%r9

	movq	24(%rbx),%rcx

	movq	0(%rsi),%rax
	mulq	%rcx
	addq	%rax,%r11
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	8(%rsi),%rax
	mulq	%rcx
	addq	%rbp,%r12
	adcq	$0,%rdx
	addq	%rax,%r12
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	16(%rsi),%rax
	mulq	%rcx
	addq	%rbp,%r13
	adcq	$0,%rdx
	addq	%rax,%r13
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	24(%rsi),%rax
	mulq	%rcx
	addq	%rbp,%r8
	adcq	$0,%rdx
	addq	%rax,%r8
	adcq	%rdx,%r9
	adcq	$0,%r10

	movq	%r11,%rax
	mulq	.LordK(%rip)
	movq	%rax,%rcx

	movq	0+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rax,%r11
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	8+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rbp,%r12
	adcq	$0,%rdx
	addq	%rax,%r12

	movq	%rcx,%rbp
	adcq	%rdx,%r13
	adcq	$0,%rbp
	subq	%rcx,%r13
	sbbq	$0,%rbp

	movq	24+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rbp,%r8
	adcq	$0,%rdx
	addq	%rax,%r8
	adcq	%rdx,%r9
	adcq	$0,%r10


	movq	%r12,%rsi
	movq	%r13,%r11
	movq	%r8,%rcx
	movq	%r9,%rbp

	subq	0+.Lord(%rip),%r12
	sbbq	8+.Lord(%rip),%r13
	sbbq	16+.Lord(%rip),%r8
	sbbq	24+.Lord(%rip),%r9
	sbbq	$0,%r10

	cmovcq	%rsi,%r12
	cmovcq	%r11,%r13
	cmovcq	%rcx,%r8
	cmovcq	%rbp,%r9

	movq	%r12,0(%rdi)
	movq	%r13,8(%rdi)
	movq	%r8,16(%rdi)
	movq	%r9,24(%rdi)

	popq	%r13
	popq	%r12
	popq	%rbx
	popq	%rbp
	.byte	0xf3,0xc3
.size	GFp_p256_scalar_mul_mont,.-GFp_p256_scalar_mul_mont





.globl	GFp_p256_scalar_sqr_rep_mont
.hidden GFp_p256_scalar_sqr_rep_mont
.type	GFp_p256_scalar_sqr_rep_mont,@function
.align	32
GFp_p256_scalar_sqr_rep_mont:

	pushq	%rbp
	pushq	%rbx
	pushq	%r12
	pushq	%r13
	pushq	%r14
	pushq	%r15

	movq	.LordK(%rip),%r15

	movq	%rdx,%r14

.Lord_sqr_loop:

	movq	0(%rsi),%rcx

	movq	8(%rsi),%rax
	mulq	%rcx
	movq	%rax,%r9
	movq	%rdx,%r10

	movq	16(%rsi),%rax
	mulq	%rcx
	addq	%rax,%r10
	adcq	$0,%rdx
	movq	%rdx,%r11

	movq	24(%rsi),%rax
	mulq	%rcx
	addq	%rax,%r11
	adcq	$0,%rdx
	movq	%rdx,%r12

	movq	8(%rsi),%rcx

	movq	16(%rsi),%rax
	mulq	%rcx
	addq	%rax,%r11
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	24(%rsi),%rax
	mulq	%rcx
	addq	%rbp,%r12
	adcq	$0,%rdx
	addq	%rax,%r12
	adcq	$0,%rdx
	movq	%rdx,%r13

	movq	16(%rsi),%rcx

	movq	24(%rsi),%rax
	mulq	%rcx
	addq	%rax,%r13
	adcq	$0,%rdx
	movq	%rdx,%rbx
	xorq	%rbp,%rbp

	addq	%r9,%r9
	adcq	%r10,%r10
	adcq	%r11,%r11
	adcq	%r12,%r12
	adcq	%r13,%r13
	adcq	%rbx,%rbx
	adcq	$0,%rbp

	movq	0(%rsi),%rax
	mulq	%rax
	movq	%rax,%r8
	movq	%rdx,%rcx

	movq	8(%rsi),%rax
	mulq	%rax
	addq	%rcx,%r9
	adcq	%rax,%r10
	adcq	$0,%rdx
	movq	%rdx,%rcx

	movq	16(%rsi),%rax
	mulq	%rax
	addq	%rcx,%r11
	adcq	%rax,%r12
	adcq	$0,%rdx
	movq	%rdx,%rcx

	movq	24(%rsi),%rax
	mulq	%rax
	addq	%rcx,%r13
	adcq	%rax,%rbx
	adcq	%rdx,%rbp
	movq	%rbp,%rsi


	movq	%r8,%rax
	mulq	%r15
	movq	%rax,%rcx

	movq	0+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rax,%r8
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	8+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rbp,%r9
	adcq	$0,%rdx
	addq	%rax,%r9

	movq	%rcx,%rbp
	adcq	%rdx,%r10
	adcq	$0,%rbp
	subq	%rcx,%r10
	sbbq	$0,%rbp

	movq	%rcx,%rax
	movq	%rcx,%rdx
	movq	%rcx,%r8
	shlq	$32,%rax
	shrq	$32,%rdx

	addq	%rbp,%r11
	adcq	$0,%r8
	subq	%rax,%r11
	sbbq	%rdx,%r8


	movq	%r9,%rax
	mulq	%r15
	movq	%rax,%rcx

	movq	0+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rax,%r9
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	8+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rbp,%r10
	adcq	$0,%rdx
	addq	%rax,%r10

	movq	%rcx,%rbp
	adcq	%rdx,%r11
	adcq	$0,%rbp
	subq	%rcx,%r11
	sbbq	$0,%rbp

	movq	%rcx,%rax
	movq	%rcx,%rdx
	movq	%rcx,%r9
	shlq	$32,%rax
	shrq	$32,%rdx

	addq	%rbp,%r8
	adcq	$0,%r9
	subq	%rax,%r8
	sbbq	%rdx,%r9


	movq	%r10,%rax
	mulq	%r15
	movq	%rax,%rcx

	movq	0+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rax,%r10
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	8+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rbp,%r11
	adcq	$0,%rdx
	addq	%rax,%r11

	movq	%rcx,%rbp
	adcq	%rdx,%r8
	adcq	$0,%rbp
	subq	%rcx,%r8
	sbbq	$0,%rbp

	movq	%rcx,%rax
	movq	%rcx,%rdx
	movq	%rcx,%r10
	shlq	$32,%rax
	shrq	$32,%rdx

	addq	%rbp,%r9
	adcq	$0,%r10
	subq	%rax,%r9
	sbbq	%rdx,%r10


	movq	%r11,%rax
	mulq	%r15
	movq	%rax,%rcx

	movq	0+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rax,%r11
	adcq	$0,%rdx
	movq	%rdx,%rbp

	movq	8+.Lord(%rip),%rax
	mulq	%rcx
	addq	%rbp,%r8
	adcq	$0,%rdx
	addq	%rax,%r8

	movq	%rcx,%rbp
	adcq	%rdx,%r9
	adcq	$0,%rbp
	subq	%rcx,%r9
	sbbq	$0,%rbp

	movq	%rcx,%rax
	movq	%rcx,%r11
	shlq	$32,%rax
	shrq	$32,%rcx

	addq	%rbp,%r10
	adcq	$0,%r11
	subq	%rax,%r10
	sbbq	%rcx,%r11
	xorq	%rcx,%rcx

	addq	%r12,%r8
	adcq	%r13,%r9
	adcq	%rbx,%r10
	adcq	%rsi,%r11
	adcq	$0,%rcx

	movq	%r8,%r12
	movq	%r9,%r13
	movq	%r10,%rbx
	movq	%r11,%rbp

	subq	0+.Lord(%rip),%r8
	sbbq	8+.Lord(%rip),%r9
	sbbq	16+.Lord(%rip),%r10
	sbbq	24+.Lord(%rip),%r11
	sbbq	$0,%rcx

	cmovcq	%r12,%r8
	cmovcq	%r13,%r9
	cmovcq	%rbx,%r10
	cmovcq	%rbp,%r11

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%rdi,%rsi
	decq	%r14
	jne	.Lord_sqr_loop

	popq	%r15
	popq	%r14
	popq	%r13
	popq	%r12
	popq	%rbx
	popq	%rbp
	.byte	0xf3,0xc3
.size	GFp_p256_scalar_sqr_rep_mont,.-GFp_p256_scalar_sqr_rep_mont
#endif
