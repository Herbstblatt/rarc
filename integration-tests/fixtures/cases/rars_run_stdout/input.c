#include <rars.h>

int main(void) {
    char* out = "Hello world!";
    syscall(RARS_PrintString, out);
    return 0;
}
