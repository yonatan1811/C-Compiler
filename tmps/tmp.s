.global main
main:
pushq %rbp
movq %rsp, %rbp
subq $8, %rsp  # Allocate space for a
movq $2, %rax

movq %rax, -8(%rbp)  # Store value in a
movq -8(%rbp), %rax  # Load variable a

movq %rbp, %rsp
popq %rbp
ret

