/**
 * userphrase.c
 *
 * Copyright (c) 2014
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include "userphrase-private.h"

#include <assert.h>

#include "chewing-private.h"
#include "chewing-sql.h"
#include "private.h"

#if defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE)

#include <Shlobj.h>
#define USERPHRASE_DIR	"ChewingTextService"

char *GetDefaultUserPhrasePath(ChewingData *pgdata)
{
	wchar_t *tmp;
	char *path;
	int i;
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
		path = calloc(sizeof(*path), len + 1 + strlen(DB_NAME) + 1);
		if (!path) {
			free(tmp);
			LOG_ERROR("calloc returns %#p", path);
			exit(-1);
		}
		WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, tmp, -1, path, len, NULL, NULL);
		strcat(path + len, "\\" DB_NAME);

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
		path = calloc(sizeof(*path), len + 1 + strlen(USERPHRASE_DIR) + 1 + strlen(DB_NAME) + 1);
		if (!path) {
			free(tmp);
			LOG_ERROR("calloc returns %#p", path);
			exit(-1);
		}
		WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, tmp, -1, path, len, NULL, NULL);
		strcat(path + len, "\\" USERPHRASE_DIR "\\" DB_NAME);

		free(tmp);
		return path;
	}

	return NULL;
}

#else

#ifdef __MaxOSX__
/* FIXME: Shall this path pre user? */
#define USERPHRASE_DIR	"/Library/ChewingOSX"
#else
#define USERPHRASE_DIR	".chewing"
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

char *GetDefaultUserPhrasePath(ChewingData *pgdata)
{
	char *tmp;
	char *path;
	int len;
	int ret;

	assert(pgdata);

	tmp = getenv("CHEWING_USER_PATH");
	if (tmp && access(tmp, W_OK) == 0) {
		ret = asprintf(&path, "%s/%s", tmp, DB_NAME);
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

	len = snprintf(NULL, 0, "%s/%s/%s", tmp, USERPHRASE_DIR, DB_NAME);
	++len;
	path = malloc(len);
	if (!path) {
		LOG_ERROR("malloc returns %#p", path);
		exit(-1);
	}

	snprintf(path, len, "%s/%s", tmp, USERPHRASE_DIR);
	PLAT_MKDIR(path);
	snprintf(path, len, "%s/%s/%s", tmp, USERPHRASE_DIR, DB_NAME);

	return path;
}

#endif
