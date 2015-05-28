/**
 * chewing-utf8-util.c
 *
 * Copyright (c) 2005, 2006, 2012-2014
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include <stdio.h>
#include <string.h>
#include "chewing-utf8-util.h"

/* Table of UTF-8 length */
static const char utf8len_tab[256] = {
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,     /*bogus */
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,     /*bogus */
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 1, 1,
};

/* Return length of UTF-8 string */
int ueStrLen(const char *str)
{
    int length = 0;
    const char *strptr = str;

    while (strptr[0] != '\0') {
        strptr += ueBytesFromChar(strptr[0]);
        ++length;
    }
    return length;
}

/* Return bytes of a UTF-8 character */
int ueBytesFromChar(unsigned char b)
{
    return utf8len_tab[b];
}

/* Return bytes of a UTF-8 string until n position */
int ueStrNBytes(const char *str, int n)
{
    int i = 0, len = 0;
    const char *iter = str;

    for (i = 0; i < n; i++) {
        len += ueBytesFromChar(iter[len]);
    }
    return len;
}

/* Return how many bytes was copied */
int ueStrNCpy(char dest[], const char *src, size_t n, int end)
{
    int len = 0;

    len = ueStrNBytes(src, n);
    memcpy(dest, src, len);
    if (end == STRNCPY_CLOSE)
        dest[len] = '\0';
    return len;
}

const char *ueConstStrSeek(const char *src, size_t n)
{
    size_t i = 0;
    const char *iter = src;

    for (i = 0; i < n; i++) {
        iter += ueBytesFromChar(iter[0]);
    }
    return iter;
}

char *ueStrSeek(char *src, size_t n)
{
    size_t i = 0;
    char *iter = src;

    for (i = 0; i < n; i++) {
        iter += ueBytesFromChar(iter[0]);
    }
    return iter;
}

/* Locate a UTF-8 substring from UTF-8 string */
const char *ueStrStr(const char *str, size_t lstr, const char *substr, size_t lsub)
{
    const char *p = str;
    size_t ub;

    if (lstr < lsub)
        return NULL;
    ub = lstr - lsub;
    for (; (size_t) (p - str) <= ub; p++) {
        if (!strncmp(p, substr, lsub))
            return p;
    }
    return NULL;
}
