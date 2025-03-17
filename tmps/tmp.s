.global main
main:
movl $1 , %eax
push %rax
movl $1 , %eax
pop %rcx
addq %rcx, %rax

not %eax
ret

