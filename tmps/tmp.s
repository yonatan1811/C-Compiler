.global main
main:
pushq %rbp
movq %rsp, %rbp
movq $2, %rax

pushq %rax
movq $2, %rax

popq %rcx
addq %rcx, %rax
movq $0, %rax

movq %rbp, %rsp
popq %rbp
ret

