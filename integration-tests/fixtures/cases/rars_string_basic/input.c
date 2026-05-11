#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int fail(const char *msg) {
    puts(msg);
    return 1;
}

int main(void) {
    {
        char a[] = "abc";
        char b[] = "abc";
        char c[] = "abd";
        if (memcmp(a, b, 3) != 0)
            return fail("fail-memcmp-eq");
        if (memcmp(a, c, 3) >= 0)
            return fail("fail-memcmp-lt");
    }

    {
        char buf[8] = "abcdef";
        memmove(buf + 2, buf, 4);
        if (strcmp(buf, "ababcd") != 0)
            return fail("fail-memmove-overlap");
    }

    {
        char dst[8];
        memcpy(dst, "xyz", 4);
        if (strcmp(dst, "xyz") != 0)
            return fail("fail-memcpy");
    }

    {
        char buf[5] = { 0 };
        memset(buf, 'Q', 3);
        buf[3] = '\0';
        if (strcmp(buf, "QQQ") != 0)
            return fail("fail-memset");
    }

    {
        const char *s = "abcde";
        if ((char *)memchr(s, 'c', 5) != s + 2)
            return fail("fail-memchr-hit");
        if (memchr(s, 'x', 5) != NULL)
            return fail("fail-memchr-miss");
    }

    {
        const char *s = "abc";
        if (strchr(s, 'b') != s + 1)
            return fail("fail-strchr");
    }

    {
        if (strcmp("abc", "abc") != 0)
            return fail("fail-strcmp-eq");
        if (strcmp("abc", "abd") >= 0)
            return fail("fail-strcmp-lt");
    }

    {
        char dst[8];
        strcpy(dst, "hi");
        if (strcmp(dst, "hi") != 0)
            return fail("fail-strcpy");
    }

    {
        if (strlen("hello") != 5)
            return fail("fail-strlen");
        if (strnlen("hello", 3) != 3)
            return fail("fail-strnlen");
    }

    {
        char *dup = strdup("dup");
        if (!dup || strcmp(dup, "dup") != 0)
            return fail("fail-strdup");
    }

    {
        char *dup = strndup("dup", 2);
        if (!dup || strcmp(dup, "du") != 0)
            return fail("fail-strndup");
    }

    {
        char dst[6] = "ab";
        size_t n = strlcat(dst, "cd", sizeof(dst));
        if (strcmp(dst, "abcd") != 0 || n != 4)
            return fail("fail-strlcat");
    }

    {
        char dst[4];
        size_t n = strlcpy(dst, "wxyz", sizeof(dst));
        if (strcmp(dst, "wxy") != 0 || n != 4)
            return fail("fail-strlcpy");
    }

    {
        char dst[8] = "ab";
        strncat(dst, "cd", 2);
        if (strcmp(dst, "abcd") != 0)
            return fail("fail-strncat");
    }

    {
        if (strncmp("abc", "abd", 2) != 0)
            return fail("fail-strncmp-prefix");
        if (strncmp("abc", "abd", 3) >= 0)
            return fail("fail-strncmp-order");
    }

    {
        char dst[5] = { 'x', 'x', 'x', 'x', 'x' };
        strncpy(dst, "hi", 5);
        if (dst[0] != 'h' || dst[1] != 'i' || dst[2] != '\0')
            return fail("fail-strncpy");
    }

    {
        const char *s = "bananas";
        if (strrchr(s, 'a') != s + 5)
            return fail("fail-strrchr");
    }

    {
        const char *s = "hello";
        if (strstr(s, "ell") != s + 1)
            return fail("fail-strstr");
    }

    {
        if (tolower('A') != 'a')
            return fail("fail-tolower");
        if (toupper('z') != 'Z')
            return fail("fail-toupper");
    }

    puts("string-ok");
    return 0;
}
