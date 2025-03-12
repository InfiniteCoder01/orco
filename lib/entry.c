#include <stdint.h>
#include <sys/syscall.h>

extern void main();
_Noreturn void exit(int code) {
    for (;;) {
        asm("mov %0, %%rax\n\t"
            "mov %1, %%rdi\n\t"
            "syscall\n\t"
            :
            : "r" ((uint64_t) SYS_exit),
              "r" ((uint64_t) code)
            : "%rax", "%rdi");
    }
}
__attribute__((force_align_arg_pointer))
void _start() {
    main();
    exit(0);
}
