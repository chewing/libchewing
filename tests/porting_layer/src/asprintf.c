#ifndef HAVE_ASPRINTF
#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>

int asprintf(char **strp, const char *fmt, ...)
{
    char *buf;
    size_t len;
    va_list ap;

    va_start(ap, fmt);
    len = vsnprintf(NULL, 0, fmt, ap);
    va_end(ap);

    buf = (char *) malloc(len + 1);
    if (!buf)
        return -1;

    va_start(ap, fmt);
    len = vsnprintf(buf, len + 1, fmt, ap);
    va_end(ap);

    *strp = buf;

    return len;
}
#endif