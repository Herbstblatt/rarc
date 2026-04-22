#include <stdarg.h>

unsigned long syscall(long number, ...) {
    register long callnum asm("a7") = number;

    va_list args;
    va_start (args, number);
    register unsigned long a0 asm("a0") = va_arg (args, unsigned long);
    register unsigned long a1 asm("a1") = va_arg (args, unsigned long);
    register unsigned long a2 asm("a2") = va_arg (args, unsigned long);
    register unsigned long a3 asm("a3") = va_arg (args, unsigned long);
    va_end (args);

    __asm__ volatile (
        "ecall"
        : "=r" (a0)
        : "r" (callnum),
        "r" (a0), "r" (a1), "r" (a2)
    );

    return a0;
}