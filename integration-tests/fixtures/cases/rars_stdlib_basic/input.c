#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <limits.h>

int main(void) {
    char buf1[32];
    char buf2[32];
    char buf3[32];
    char buf4[32];
    char buf5[32];
    char buf6[32];

    printf("abs=%d\n", abs(-7));
    printf("labs=%ld\n", labs(-123456L));
    {
        char tmp[32];
        i64toa_r(llabs(-9000000000LL), tmp);
        printf("llabs=%s\n", tmp);
    }

    printf("atol=%ld\n", atol("-42"));
    printf("atoi=%d\n", atoi("17"));

    utoa_r(12345UL, buf1);
    itoa_r(-678L, buf2);
    u64toa_r(1234567890123456789ULL, buf3);
    i64toa_r(-1234567890123LL, buf4);
    utoh_r(0x2aUL, buf5);
    u64toh_r(0x1234abcd5678ef00ULL, buf6);

    printf("utoa_r=%s\n", buf1);
    printf("itoa_r=%s\n", buf2);
    printf("u64toa_r=%s\n", buf3);
    printf("i64toa_r=%s\n", buf4);
    printf("utoh_r=%s\n", buf5);
    printf("u64toh_r=%s\n", buf6);

    {
        char *end = NULL;
        char tmp[32];
        char endbuf[32];
        long val = strtol(", -0x2a!", &end, 0);
        itoa_r(val, tmp);
        utoh_r((unsigned long)(unsigned char)*end, endbuf);
        printf("strtol=%s, end=0x%s\n", tmp, endbuf);
    }
    {
        char *end = NULL;
        char tmp[32];
        char endbuf[32];
        unsigned long val = strtoul("0755x", &end, 0);
        utoa_r(val, tmp);
        utoh_r((unsigned long)(unsigned char)*end, endbuf);
        printf("strtoul=%s, end=0x%s\n", tmp, endbuf);
    }
    {
        char *end = NULL;
        char tmp[32];
        char endbuf[32];
        unsigned long long val = strtoull("ffzz", &end, 16);
        u64toa_r(val, tmp);
        utoh_r((unsigned long)(unsigned char)*end, endbuf);
        printf("strtoull=%s, end=0x%s\n", tmp, endbuf);
    }
    {
        char *end = NULL;
        char tmp[32];
        char endbuf[32];
        long val = strtol("1", &end, 37);
        itoa_r(val, tmp);
        utoh_r((unsigned long)(unsigned char)*end, endbuf);
        printf("strtol_base=%s, errno=%d, end=0x%s\n", tmp, errno, endbuf);
    }
    {
        char *end = NULL;
        char tmp[32];
        long long val = strtoll("99999999999999999999", &end, 10);
        i64toa_r(val, tmp);
        printf("strtoll_overflow=%s, errno=%d\n", tmp, errno);
    }
    {
        char *end = NULL;
        uintmax_t val = strtoumax("42", &end, 10);
        u64toa_r((uint64_t)val, buf1);
        {
            char endbuf[32];
            utoh_r((unsigned long)(unsigned char)*end, endbuf);
            printf("strtoumax=%s, end=0x%s\n", buf1, endbuf);
        }
    }

    return 0;
}
