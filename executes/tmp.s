.global main
main:
movl $1 , %eax
push %eax
movl $2 , %eax
pop %ecx
addl %ecx,%eax
ret