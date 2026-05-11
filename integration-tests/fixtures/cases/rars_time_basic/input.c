#include <stdio.h>
#include <time.h>
#include <rars.h>

int main(void) {
    time_t t1 = time(NULL);
    syscall(RARS_Sleep, 1100);
    time_t t2 = time(NULL);

    if (t2 >= t1 + 1) {
        puts("time-ok");
        return 0;
    }

    puts("time-fail");
    return 1;
}
