.global main
main:
movl $2 , %eax
push %rax
movl $2 , %eax
pop %rcx
cmpq %rcx, %rax
sete %al
movzbq %al, %rax

testq %rax, %rax
jnz .true_1
movl $0 , %eax
testq %rax, %rax
jnz .true_1
movq $0, %rax
jmp .end_2
.true_1:
movq $1, %rax
.end_2:ret

