.global main
main:
movl $0 , %eax
cmp $0, %eax
sete %al
movzbl %al, %eax
ret

