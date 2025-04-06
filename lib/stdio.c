#include <stdint.h>
#include <sys/syscall.h>

int write(int fd, const char *buf, int length) {
    int ret;

    asm("mov %1, %%rax\n\t"
        "mov %2, %%rdi\n\t"
        "mov %3, %%rsi\n\t"
        "mov %4, %%rdx\n\t"
        "syscall\n\t"
        "mov %%eax, %0"
        : "=r" (ret)
        : "r" ((uint64_t) SYS_write), // #define SYS_write 1
          "r" ((uint64_t) fd),
          "r" ((uint64_t) buf),
          "r" ((uint64_t) length)
        : "%rax", "%rdi", "%rsi", "%rdx");

    return ret;
}

void print(int x) {
    char buff[4];
    buff[0] = (x / 10) % 10 + '0';
    buff[1] = x % 10 + '0';
    buff[2] = '\n';
    buff[3] = '\0';
    write(0, buff, 3);
}
