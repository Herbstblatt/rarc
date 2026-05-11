#include <stdio.h>
#include <string.h>
#include <time.h>

static int fail(const char *msg) {
    puts(msg);
    return 1;
}

int main(void) {
    time_t t0 = 0;
    struct tm *tm0 = gmtime(&t0);
    if (!tm0)
        return fail("fail-gmtime-null");
    if (tm0->tm_year != 70 || tm0->tm_mon != 0 || tm0->tm_mday != 1 ||
        tm0->tm_hour != 0 || tm0->tm_min != 0 || tm0->tm_sec != 0 ||
        tm0->tm_wday != 4 || tm0->tm_yday != 0)
        return fail("fail-gmtime-fields");

    struct tm *tm1 = localtime(&t0);
    if (!tm1)
        return fail("fail-localtime-null");
    if (tm1->tm_year != tm0->tm_year || tm1->tm_mon != tm0->tm_mon ||
        tm1->tm_mday != tm0->tm_mday || tm1->tm_hour != tm0->tm_hour ||
        tm1->tm_min != tm0->tm_min || tm1->tm_sec != tm0->tm_sec)
        return fail("fail-localtime-fields");

    {
        char expected[] = {
            'T','h','u',' ','J','a','n',' ',' ','1',' ','0','0',':','0','0',':','0','0',
            ' ','1','9','7','0','\n','\0'
        };
        char *asc = asctime(tm0);
        if (!asc || strcmp(asc, expected) != 0)
            return fail("fail-asctime");
    }

    {
        char expected[] = {
            'T','h','u',' ','J','a','n',' ',' ','1',' ','0','0',':','0','0',':','0','0',
            ' ','1','9','7','0','\n','\0'
        };
        char *ct = ctime(&t0);
        if (!ct || strcmp(ct, expected) != 0)
            return fail("fail-ctime");
    }

    {
        char fmt[] = {
            '%','Y','-','%','m','-','%','d',' ','%','H',':','%','M',':','%','S','\0'
        };
        char expected[] = {
            '1','9','7','0','-','0','1','-','0','1',' ','0','0',':','0','0',':','0','0','\0'
        };
        char buf[64];
        size_t n = strftime(buf, sizeof(buf), fmt, tm0);
        if (n != 19 || strcmp(buf, expected) != 0)
            return fail("fail-strftime-ymd");
    }

    {
        char fmt[] = { '%','c','\0' };
        char expected[] = {
            'T','h','u',' ','J','a','n',' ',' ','1',' ','0','0',':','0','0',':','0','0',
            ' ','1','9','7','0','\0'
        };
        char buf[64];
        size_t n = strftime(buf, sizeof(buf), fmt, tm0);
        if (n != 24 || strcmp(buf, expected) != 0)
            return fail("fail-strftime-c");
    }

    {
        struct tm tm2;
        tm2.tm_year = 70;
        tm2.tm_mon = 0;
        tm2.tm_mday = 2;
        tm2.tm_hour = 0;
        tm2.tm_min = 0;
        tm2.tm_sec = 0;
        tm2.tm_isdst = 0;

        time_t t2 = mktime(&tm2);
        if (t2 != 86400)
            return fail("fail-mktime-seconds");
        if (tm2.tm_wday != 5 || tm2.tm_yday != 1)
            return fail("fail-mktime-fields");
    }

    puts("time-funcs-ok");
    return 0;
}
