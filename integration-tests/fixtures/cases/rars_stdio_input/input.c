#include <stdio.h>

int main(void) {
    int c1 = getchar();
    int c2 = getchar();
    char line[16];
    int num = 0;
    int matches = 0;

    if (!fgets(line, sizeof(line), stdin)) {
        puts("fgets-fail");
        return 1;
    }

    matches = sscanf(line, "%d", &num);

    printf("c1=%c\n", c1);
    printf("c2=%c\n", c2);
    printf("line=%s", line);
    printf("matches=%d\n", matches);
    printf("num=%d\n", num);

    return 0;
}
