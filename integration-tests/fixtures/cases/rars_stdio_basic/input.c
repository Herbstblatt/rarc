#include <stdio.h>
#include <stdarg.h>
#include <errno.h>

static void test_vprintf(const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    vprintf(fmt, args);
    va_end(args);
}

static void test_vfprintf(FILE *stream, const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    vfprintf(stream, fmt, args);
    va_end(args);
}

static int test_vsnprintf(char *buf, size_t size, const char *fmt, ...) {
    va_list args;
    int ret;

    va_start(args, fmt);
    ret = vsnprintf(buf, size, fmt, args);
    va_end(args);
    return ret;
}

static int test_vsprintf(char *buf, const char *fmt, ...) {
    va_list args;
    int ret;

    va_start(args, fmt);
    ret = vsprintf(buf, fmt, args);
    va_end(args);
    return ret;
}

static int test_vdprintf(int fd, const char *fmt, ...) {
    va_list args;
    int ret;

    va_start(args, fmt);
    ret = vdprintf(fd, fmt, args);
    va_end(args);
    return ret;
}

static int test_vsscanf(const char *str, const char *fmt, ...) {
    va_list args;
    int ret;

    va_start(args, fmt);
    ret = vsscanf(str, fmt, args);
    va_end(args);
    return ret;
}

int main(void) {
    char buf[64];

    puts("puts-ok");
    fputs("fputs-ok", stdout);
    putchar('\n');
    fputs("fputc=", stdout);
    fputc('Z', stdout);
    putchar('\n');

    printf("printf-int=");
    printf("%d\n", 123);
    test_vprintf("vprintf-int=%d\n", 7);
    fprintf(stdout, "fprintf-int=%d\n", 8);
    test_vfprintf(stdout, "vfprintf-int=%d\n", 9);
    dprintf(1, "dprintf-hex=%x\n", 0x2a);
    test_vdprintf(1, "vdprintf-str=%s\n", "ok");

    snprintf(buf, sizeof(buf), "snprintf-int=%d", 5);
    puts(buf);
    test_vsnprintf(buf, sizeof(buf), "vsnprintf-str=%s", "ok");
    puts(buf);
    sprintf(buf, "sprintf-hex=%x", 0x3b);
    puts(buf);
    test_vsprintf(buf, "vsprintf-int=%d", 11);
    puts(buf);

    {
        int out_i = 0;
        unsigned out_u = 0;
        void *out_p = 0;
        int matches = sscanf("12 34 0x10", "%d %u %p", &out_i, &out_u, &out_p);
        printf("sscanf-count=%d\n", matches);
        printf("sscanf-int=%d\n", out_i);
        printf("sscanf-uint=%u\n", out_u);
        printf("sscanf-ptr=%p\n", out_p);
    }
    {
        int out_i = 0;
        unsigned out_u = 0;
        void *out_p = 0;
        int matches = test_vsscanf("7 8 0x20", "%d %u %p", &out_i, &out_u, &out_p);
        printf("vsscanf-count=%d\n", matches);
        printf("vsscanf-int=%d\n", out_i);
        printf("vsscanf-uint=%u\n", out_u);
        printf("vsscanf-ptr=%p\n", out_p);
    }

    printf("setvbuf=%d\n", setvbuf(stdout, NULL, _IONBF, 0));
    printf("strerror=%s\n", strerror(5));
    errno = 7;
    perror("perr");

    {
        FILE *file = fopen("stdio_test.txt", "w");
        if (!file) {
            puts("fopen-w-fail");
            return 1;
        }
        printf("fwrite=%u\n", (unsigned)fwrite("abc\n123\n", 1, 8, file));
        printf("fflush=%d\n", fflush(file));
        printf("fclose=%d\n", fclose(file));

        file = fopen("stdio_test.txt", "r");
        if (!file) {
            puts("fopen-r-fail");
            return 1;
        }
        (void)fileno(file);
        printf("fgetc=%c\n", fgetc(file));
        {
            char line[8];
            fgets(line, sizeof(line), file);
            printf("fgets=%s", line);
        }
        printf("fgetc2=%c\n", fgetc(file));
        printf("fclose2=%d\n", fclose(file));
    }

    {
        FILE *out = fdopen(1, "w");
        fputs("fdopen-ok\n", out);
        printf("fflush2=%d\n", fflush(out));
    }

    return 0;
}
