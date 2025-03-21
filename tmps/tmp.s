.global main
main:
pushq %rbp
movq %rsp, %rbp
movq $2, %rax

movq %rax, -8(%rbp)  # Assign value to a
movq -8(%rbp), %rax  # Load variable a

movq %rbp, %rsp
popq %rbp
ret

