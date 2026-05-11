#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    char *p = (char *)malloc(16);
    if (!p) {
        puts("malloc1=null");
        return 1;
    }
    for (int i = 0; i < 15; i++) {
        p[i] = (char)('A' + i);
    }
    p[15] = '\0';
    printf("buf1=%s\n", p);

    free(p);

    char *q = (char *)malloc(8);
    printf("malloc2=%s\n", q ? "ok" : "null");
    if (!q) {
        return 1;
    }
    strcpy(q, "hi");
    printf("buf2=%s\n", q);

    char *z = (char *)calloc(4, 4);
    if (!z) {
        puts("calloc=null");
        return 1;
    }
    int zeros = 1;
    for (int i = 0; i < 16; i++) {
        if (z[i] != 0) {
            zeros = 0;
            break;
        }
    }
    printf("calloc_zero=%d\n", zeros);

    char *r = (char *)realloc(q, 32);
    if (!r) {
        puts("realloc=null");
        return 1;
    }
    {
        size_t len = strlen(r);
        if (len + 1 < 32) {
            r[len] = '!';
            r[len + 1] = '\0';
        }
    }
    printf("buf3=%s\n", r);

    char *s = (char *)realloc(r, 4);
    if (!s) {
        puts("realloc2=null");
        return 1;
    }
    s[3] = '\0';
    printf("buf4=%s\n", s);

    free(s);
    free(z);
    return 0;
}
