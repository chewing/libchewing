/**
 * userphrase.c
 *
 * Copyright (c) 2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include "userphrase-private.h"

#include <assert.h>

#include "chewing-private.h"
#include "chewing-sql.h"
#include "private.h"

#include "plat_path.h"

#if defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE)

#    include <Shlobj.h>
#    define USERPHRASE_DIR	"ChewingTextService"

char *GetDefaultChewingUserPath(ChewingData *pgdata)
{
    wchar_t *tmp;
    char *path;
    int path_len;
    int len;

    assert(pgdata);

    len = GetEnvironmentVariableW(L"CHEWING_USER_PATH", NULL, 0);
    if (len) {
        tmp = calloc(sizeof(*tmp), len);
        if (!tmp) {
            LOG_ERROR("calloc returns %#p", tmp);
            exit(-1);
        }

        GetEnvironmentVariableW(L"CHEWING_USER_PATH", tmp, len);

        len = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, tmp, -1, NULL, 0, NULL, NULL);
        path_len = len + 1;
        path = calloc(sizeof(*path), path_len);
        if (!path) {
            free(tmp);
            LOG_ERROR("calloc returns %#p", path);
            exit(-1);
        }
        WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, tmp, -1, path, len, NULL, NULL);

        LOG_INFO("chewing user path is at %s", path);

        free(tmp);
        return path;
    }

    len = GetEnvironmentVariableW(L"USERPROFILE", NULL, 0);
    if (len) {
        tmp = calloc(sizeof(*tmp), len);
        if (!tmp) {
            LOG_ERROR("calloc returns %#p", tmp);
            exit(-1);
        }

        GetEnvironmentVariableW(L"USERPROFILE", tmp, len);

        len = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, tmp, -1, NULL, 0, NULL, NULL);
        path = calloc(sizeof(*path), len + 1 + strlen(USERPHRASE_DIR) + 1);
        if (!path) {
            free(tmp);
            LOG_ERROR("calloc returns %#p", path);
            exit(-1);
        }
        WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, tmp, -1, path, len, NULL, NULL);

        strcpy(path + len - 1, "\\" USERPHRASE_DIR);
        LOG_INFO("chewing user path is at %s", path);

        free(tmp);
        return path;
    }

    return NULL;
}

char *GetDefaultUserPhrasePath(ChewingData *pgdata)
{
    char *tmp;
    char *path;
    int ret;

    assert(pgdata);

    tmp = GetDefaultChewingUserPath(pgdata);
    if (tmp) {
        ret = asprintf(&path, "%s\\%s", tmp, DB_NAME);
        if (ret == -1) {
            free(tmp);
            LOG_ERROR("asprintf returns %d", ret);
            exit(-1);
        }

        LOG_INFO("userphrase is at %s", path);

        free(tmp);
        return path;
    }

    return NULL;
}

#else

#    ifdef __MaxOSX__
/* FIXME: Shall this path pre user? */
#        define USERPHRASE_DIR	"/Library/ChewingOSX"
#    else
#        define USERPHRASE_DIR	".chewing"
#    endif

#    include <stdio.h>
#    include <stdlib.h>
#    include <string.h>
#    include <unistd.h>

char *GetDefaultChewingUserPath(ChewingData *pgdata)
{
    char *tmp;
    char *path;
    int ret;

    assert(pgdata);

    tmp = getenv("CHEWING_USER_PATH");
    if (tmp) {
        ret = asprintf(&path, "%s", tmp);
        if (ret == -1) {
            LOG_ERROR("asprintf returns %d", ret);
            exit(-1);
        }
        return path;
    }

    tmp = getenv("HOME");
    if (!tmp) {
        tmp = PLAT_TMPDIR;
    }

    ret = asprintf(&path, "%s/%s", tmp, USERPHRASE_DIR);
    if (ret == -1) {
        LOG_ERROR("asprintf returns %d", ret);
        exit(-1);
    }

    PLAT_MKDIR(path);

    return path;
}

char *GetDefaultUserPhrasePath(ChewingData *pgdata)
{
    char *tmp;
    char *path;
    int ret;

    assert(pgdata);

    tmp = GetDefaultChewingUserPath(pgdata);
    if (tmp && access(tmp, W_OK) == 0) {
        ret = asprintf(&path, "%s/%s", tmp, DB_NAME);
        if (ret == -1) {
            free(tmp);
            LOG_ERROR("asprintf returns %d", ret);
            exit(-1);
        }
        free(tmp);
        return path;
    }
    free(tmp);

    return NULL;
}

#endif
