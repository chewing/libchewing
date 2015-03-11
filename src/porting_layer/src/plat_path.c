/**
 * plat_path.c
 *
 * Copyright (c) 2012-2014
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif
#include "plat_path.h"

#ifndef HAVE_ASPRINTF
#    include <stdarg.h>
#endif
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "plat_types.h"

#ifdef UNDER_POSIX
#    define SEARCH_PATH_SEP ":"
int get_search_path(char *path, size_t path_len)
{
    char *chewing_path;
    char *home;

    chewing_path = getenv("CHEWING_PATH");
    if (chewing_path) {
        strncpy(path, chewing_path, path_len);
    } else {
        home = getenv("HOME");
        if (home) {
            snprintf(path, path_len,
                     "%s/.chewing" SEARCH_PATH_SEP CHEWING_DATADIR, home);
        } else {
            /* No HOME ? */
            strncpy(path, SEARCH_PATH_SEP CHEWING_DATADIR, path_len);
        }
    }

    return 0;
}

#elif defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE)
#include <Shlobj.h>

#    define SEARCH_PATH_SEP ";"

int get_search_path(char *path, size_t path_len)
{
    char *chewing_path;
    size_t len;
    HRESULT result;

    chewing_path = getenv("CHEWING_PATH");
    if (chewing_path) {
        /* FIXME: Check for truncated. */
        strncpy(path, chewing_path, path_len);
    } else {

        /*
         * Try to search dictionary location at the following path
         *
         * - %CSIDL_PROGRAM_FILESX86%/ChewingTextService/Dictionary
         * - %CSIDL_PROGRAM_FILES%/ChewingTextService/Dictionary
         */
        if (path_len < MAX_PATH)
            return -1;

        result = SHGetFolderPathA(NULL, CSIDL_PROGRAM_FILESX86, NULL, 0, path);
        if (result != S_OK)
            result = SHGetFolderPathA(NULL, CSIDL_PROGRAM_FILES, NULL, 0, path);

        if (result != S_OK)
            return -1;

        len = strlen(path);
        path += len;
        path_len -= len;

        /* FIXME: Check for truncated. */
        snprintf(path, path_len,
                 "\\%s\\%s", "ChewingTextService", "Dictionary");
    }

    return 0;
}
#else
#    error please implement get_search_path
#endif

#ifndef HAVE_STRTOK_R
char *strtok_r(char *s, const char *delim, char **save_ptr)
{
    char *token;

    if (s == NULL)
        s = *save_ptr;

    /* Scan leading delimiters.  */
    s += strspn(s, delim);
    if (*s == '\0') {
        *save_ptr = s;
        return NULL;
    }

    /* Find the end of the token.  */
    token = s;
    s = strpbrk(token, delim);
    if (s == NULL) {
        /* This token finishes the string.  */
        *save_ptr = token + strlen(token);
    } else {
        /* Terminate the token and make *SAVE_PTR point past it.  */
        *s = '\0';
        *save_ptr = s + 1;
    }
    return token;
}
#endif

#ifndef HAVE_ASPRINTF
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

static int are_all_files_readable(const char *path,
                                  const char *const *files,
                                  char *output, size_t output_len)
{
    int i;

    assert(path);
    assert(files);

    for (i = 0; files[i] != NULL; ++i) {
        snprintf(output, output_len,
                 "%s" PLAT_SEPARATOR "%s", path, files[i]);
        if (access(output, R_OK) != 0)
            return 0;
    }

    return 1;
}

int find_path_by_files(const char *search_path,
                       const char *const *files,
                       char *output, size_t output_len)
{
    char buffer[PATH_MAX + 1] = {0};
    char *path;
    char *saveptr;
    int ret;

    assert(search_path);
    assert(files);
    assert(output);
    assert(output_len);

    /* strtok_r will modify its first parameter. */
    strncpy(buffer, search_path, sizeof(buffer) - 1);

    for (path = strtok_r(buffer, SEARCH_PATH_SEP, &saveptr); path;
         path = strtok_r(NULL, SEARCH_PATH_SEP, &saveptr)) {
        ret = are_all_files_readable(path, files, output, output_len);
        if (ret) {
            snprintf(output, output_len, "%s", path);
            return 0;
        }
    }
    return -1;
}
