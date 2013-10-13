/**
 * chewing-sql.c
 *
 * Copyright (c) 2013
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include "chewing-sql.h"
#include "chewing-private.h"

#include <assert.h>
#include <malloc.h>
#include <stdlib.h>

#include "sqlite3.h"
#include "plat_types.h"
#include "private.h"

const SqlStmtUserphrase SQL_STMT_USERPHRASE[STMT_USERPHRASE_COUNT] = {
	{
		"SELECT length, phrase, "
			"phone_0, phone_1, phone_2, phone_3, phone_4, phone_5, "
			"phone_6, phone_7, phone_8, phone_9, phone_10 "
			"FROM userphrase_v1",
		{ -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1 },
		{ -1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12 },
	},
	{
		"SELECT time, user_freq, max_freq, orig_freq, phrase "
			"FROM userphrase_v1 WHERE length = ?1 AND "
			"phone_0 = ?10 AND phone_1 = ?11 AND phone_2 = ?12 AND "
			"phone_3 = ?13 AND phone_4 = ?14 AND phone_5 = ?15 AND "
			"phone_6 = ?16 AND phone_7 = ?17 AND phone_8 = ?18 AND "
			"phone_9 = ?19 AND phone_10 = ?20",
		{ -1, -1, -1, -1, 1, -1, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20 },
		{ 0, 1, 2, 3, -1, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1 },
	},
	{
		"SELECT time, user_freq, max_freq, orig_freq "
			"FROM userphrase_v1 WHERE length = ?1 AND phrase = ?2 AND "
			"phone_0 = ?10 AND phone_1 = ?11 AND phone_2 = ?12 AND "
			"phone_3 = ?13 AND phone_4 = ?14 AND phone_5 = ?15 AND "
			"phone_6 = ?16 AND phone_7 = ?17 AND phone_8 = ?18 AND "
			"phone_9 = ?19 AND phone_10 = ?20",
		{ -1, -1, -1, -1, 1, 2, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20 },
		{ 0, 1, 2, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1 },
	},
	{
		"INSERT OR REPLACE INTO userphrase_v1 ("
			"time, user_freq, max_freq, orig_freq, length, phrase, "
			"phone_0, phone_1, phone_2, phone_3, phone_4, phone_5, "
			"phone_6, phone_7, phone_8, phone_9, phone_10) "
			"VALUES (?1, ?2, ?3, ?4, ?5, ?6, "
			"?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
		{ 1, 2, 3, 4, 5, 6, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20 },
		{ -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1 },
	},
	{
		"DELETE FROM userphrase_v1 WHERE length = ?1 AND phrase = ?2 AND "
			"phone_0 = ?10 AND phone_1 = ?11 AND phone_2 = ?12 AND "
			"phone_3 = ?13 AND phone_4 = ?14 AND phone_5 = ?15 AND "
			"phone_6 = ?16 AND phone_7 = ?17 AND phone_8 = ?18 AND "
			"phone_9 = ?19 AND phone_10 = ?20",
		{ -1, -1, -1, -1, 1, 2, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20 },
		{ -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1 },
	},
	{
		"SELECT MAX(user_freq) FROM userphrase_v1 WHERE length = ?1 AND "
			"phone_0 = ?10 AND phone_1 = ?11 AND phone_2 = ?12 AND "
			"phone_3 = ?13 AND phone_4 = ?14 AND phone_5 = ?15 AND "
			"phone_6 = ?16 AND phone_7 = ?17 AND phone_8 = ?18 AND "
			"phone_9 = ?19 AND phone_10 = ?20",
		{ -1, -1, -1, -1, 1, -1, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20 },
		{ -1, 0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1 },
	},
};

const SqlStmtConfig SQL_STMT_CONFIG[STMT_CONFIG_COUNT] = {
	{
		"SELECT value FROM config_v1 WHERE id = ?1",
		{ 1, -1 },
		{ -1, 0 },
	},
	{
		"INSERT OR IGNORE INTO config_v1 (id, value) VALUES (?1, ?2)",
		{ 1, 2 },
		{ -1, -1 },
	},
	{
		"UPDATE config_v1 SET value = value + ?1 WHERE id = ?2",
		{ 2, 1 },
		{ -1, -1 },
	},
};

#define CHEWING_MAX_DB_PATH	(1024)

#if defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE)

#include <Shlobj.h>
#define CHEWING_DB_PATH		L"chewing"
#define CHEWING_DB_NAME		L"chewing.db"

static int SetSQLiteTemp(char *buf, size_t len, wchar_t *wbuf, size_t wlen)
{
	/*
	 * Set temporary directory is necessary for Windows platform.
	 * http://www.sqlite.org/capi3ref.html#sqlite3_temp_directory
	 */

	int ret;

	ret = GetTempPathW(wlen, wbuf);
	if (ret == 0 || ret >= wlen) return -1;

	ret = WideCharToMultiByte(CP_UTF8, 0, wbuf, -1, buf, len, NULL, NULL);
	if (ret == 0) return -1;

	// FIXME: When to free sqlite3_temp_directory?
	// FIXME: thread safe?
	sqlite3_temp_directory = sqlite3_mprintf("%s", buf);
	if (sqlite3_temp_directory == 0) exit(-1);

	return 0;
}

static int GetSQLitePath(wchar_t *wbuf, size_t wlen)
{
	int ret;

	ret = GetEnvironmentVariableW(L"CHEWING_USER_PATH", wbuf, wlen);
	if (ret) {
		wcscat_s(wbuf, wlen, L"\\" CHEWING_DB_NAME);
		return 0;
	}

	// FIXME: Shall be %USERPROFILE%\ChewingTextService
	ret = GetEnvironmentVariableW(L"APPDATA", wbuf, wlen);
	if (ret) {
		wcscat_s(wbuf, wlen, L"\\" CHEWING_DB_PATH);

		ret = CreateDirectoryW(wbuf, 0);
		if (ret != 0 || GetLastError() == ERROR_ALREADY_EXISTS) {
			wcscat_s(wbuf, wlen, L"\\" CHEWING_DB_NAME);
			return 0;
		}
	}
	return -1;
}

sqlite3 *GetSQLiteInstance(ChewingData *pgdata)
{
	wchar_t *wbuf = NULL;
	char *buf = NULL;
	int ret;
	sqlite3 *db = NULL;

	wbuf = (wchar_t *) calloc(CHEWING_MAX_DB_PATH, sizeof(*wbuf));
	if (!wbuf) exit(-1);

	buf = (char *) calloc(CHEWING_MAX_DB_PATH, sizeof(*buf));
	if (!buf) exit(-1);

	ret = SetSQLiteTemp(buf, CHEWING_MAX_DB_PATH, wbuf, CHEWING_MAX_DB_PATH);
	if (ret) goto end;

	ret = GetSQLitePath(wbuf, CHEWING_MAX_DB_PATH);
	if (ret) goto end;

	ret = sqlite3_open16(wbuf, &db);
	if (ret != SQLITE_OK) goto end;

end:
	free(buf);
	free(wbuf);
	return db;
}

#else

#include <string.h>
#include <unistd.h>

#define CHEWING_DB_PATH		"chewing"
#define CHEWING_DB_NAME		"chewing.db"

static int GetSQLitePath(char *buf, size_t len)
{
	char *path;

	path = getenv("CHEWING_USER_PATH");
	if (path && access(path, W_OK) == 0) {
		snprintf(buf, len, "%s" PLAT_SEPARATOR "%s", path, CHEWING_DB_NAME);
		return 0;
	}

	path = getenv("HOME");
	if (!path) {
		path = PLAT_TMPDIR;
	}

	snprintf(buf, len, "%s" PLAT_SEPARATOR "%s", path, CHEWING_DB_PATH);
	PLAT_MKDIR(buf);
	strncat(buf, PLAT_SEPARATOR CHEWING_DB_NAME, len - strlen(buf));
	return 0;
}

sqlite3 * GetSQLiteInstance(ChewingData *pgdata)
{
	char *buf = NULL;
	int ret;
	sqlite3 *db = NULL;

	buf = (char *) calloc(CHEWING_MAX_DB_PATH, sizeof(*buf));
	if (!buf) exit(-1);

	ret = GetSQLitePath(buf, CHEWING_MAX_DB_PATH);
	if (ret) {
		LOG_ERROR("GetSQLitePath returns %d", ret);
		goto end;
	}

	ret = sqlite3_open(buf, &db);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_open returns %d", ret);
		goto end;
	}

end:
	free(buf);
	return db;
}

#endif

static int CreateTable(ChewingData *pgdata)
{
	int ret;

	STATIC_ASSERT(MAX_PHRASE_LEN == 11, update_database_schema_for_max_phrase_len);

	ret = sqlite3_exec(pgdata->static_data.db,
		"CREATE TABLE IF NOT EXISTS userphrase_v1 ("
		"time INTEGER,"
		"user_freq INTEGER,"
		"max_freq INTEGER,"
		"orig_freq INTEGER,"
		"length INTEGER,"
		"phone_0 INTEGER,"
		"phone_1 INTEGER,"
		"phone_2 INTEGER,"
		"phone_3 INTEGER,"
		"phone_4 INTEGER,"
		"phone_5 INTEGER,"
		"phone_6 INTEGER,"
		"phone_7 INTEGER,"
		"phone_8 INTEGER,"
		"phone_9 INTEGER,"
		"phone_10 INTEGER,"
		"phrase TEXT,"
		"PRIMARY KEY ("
			"phone_0,"
			"phone_1,"
			"phone_2,"
			"phone_3,"
			"phone_4,"
			"phone_5,"
			"phone_6,"
			"phone_7,"
			"phone_8,"
			"phone_9,"
			"phone_10,"
			"phrase)"
		")",
		NULL, NULL, NULL );
	if (ret != SQLITE_OK) {
		LOG_ERROR("Cannot create table userphrase_v1, error = %d", ret);
		return -1;
	}

	ret = sqlite3_exec(pgdata->static_data.db,
		"CREATE TABLE IF NOT EXISTS config_v1 ("
		"id INTEGER,"
		"value INTEGER,"
		"PRIMARY KEY (id)"
		")",
		NULL, NULL, NULL);
	if (ret != SQLITE_OK) {
		LOG_ERROR("Cannot create table config_v1, error = %d", ret);
		return -1;
	}

	return 0;
}

static int SetupUserphraseLifeTime(ChewingData *pgdata)
{
	int ret;

	ret = sqlite3_reset(pgdata->static_data.stmt_config[STMT_CONFIG_INSERT]);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_reset returns %d", ret);
		return -1;
	}

	ret = sqlite3_clear_bindings(pgdata->static_data.stmt_config[STMT_CONFIG_INSERT]);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_clear_bindings returns %d", ret);
		return -1;
	}

	ret = sqlite3_bind_int(pgdata->static_data.stmt_config[STMT_CONFIG_INSERT],
		SQL_STMT_CONFIG[STMT_CONFIG_INSERT].bind[BIND_CONFIG_ID],
		CONFIG_ID_LIFETIME);
	if (ret != SQLITE_OK) {
		LOG_ERROR("Cannot bind ?%d to %d in stmt %s, error = %d",
			SQL_STMT_CONFIG[STMT_CONFIG_INSERT].bind[BIND_CONFIG_ID],
			CONFIG_ID_LIFETIME,
			SQL_STMT_CONFIG[STMT_CONFIG_INSERT].stmt, ret);
		return -1;
	}

	ret = sqlite3_bind_int(pgdata->static_data.stmt_config[STMT_CONFIG_INSERT],
		SQL_STMT_CONFIG[STMT_CONFIG_INSERT].bind[BIND_CONFIG_VALUE], 0);
	if (ret != SQLITE_OK) {
		LOG_ERROR("Cannot bind ?%d to %d in stmt %s, error = %d",
			SQL_STMT_CONFIG[STMT_CONFIG_INSERT].bind[BIND_CONFIG_VALUE],
			0,
			SQL_STMT_CONFIG[STMT_CONFIG_INSERT].stmt, ret);
		return -1;
	}

	ret = sqlite3_step(pgdata->static_data.stmt_config[STMT_CONFIG_INSERT]);
	if (ret != SQLITE_DONE) {
		LOG_ERROR("sqlite3_step returns %d", ret);
		return -1;
	}


	ret = sqlite3_reset(pgdata->static_data.stmt_config[STMT_CONFIG_SELECT]);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_reset returns %d", ret);
		return -1;
	}

	ret = sqlite3_clear_bindings(pgdata->static_data.stmt_config[STMT_CONFIG_SELECT]);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_clear_bindings returns %d", ret);
		return -1;
	}

	ret = sqlite3_bind_int(pgdata->static_data.stmt_config[STMT_CONFIG_SELECT],
		SQL_STMT_CONFIG[STMT_CONFIG_SELECT].bind[BIND_CONFIG_ID],
		CONFIG_ID_LIFETIME);
	if (ret != SQLITE_OK) {
		LOG_ERROR("Cannot bind ?%d to %d in stmt %s, error = %d",
			SQL_STMT_CONFIG[STMT_CONFIG_SELECT].bind[BIND_CONFIG_ID],
			CONFIG_ID_LIFETIME,
			SQL_STMT_CONFIG[STMT_CONFIG_SELECT].stmt, ret);
		return -1;
	}

	ret = sqlite3_step(pgdata->static_data.stmt_config[STMT_CONFIG_SELECT]);
	if (ret != SQLITE_ROW) {
		LOG_ERROR("sqlite3_step returns %d", ret);
		return -1;
	}

	pgdata->static_data.original_lifetime = sqlite3_column_int(
		pgdata->static_data.stmt_config[STMT_CONFIG_SELECT],
		SQL_STMT_CONFIG[STMT_CONFIG_SELECT].column[COLUMN_CONFIG_VALUE]);
	pgdata->static_data.new_lifetime = pgdata->static_data.original_lifetime;

	return 0;
}

static int UpdateLifeTime(ChewingData *pgdata)
{
	int ret;

	ret = sqlite3_reset(pgdata->static_data.stmt_config[STMT_CONFIG_INCREASE]);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_reset returns %d", ret);
		return -1;
	}

	ret = sqlite3_clear_bindings(pgdata->static_data.stmt_config[STMT_CONFIG_INCREASE]);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_clear_bindings returns %d", ret);
		return -1;
	}

	ret = sqlite3_bind_int(pgdata->static_data.stmt_config[STMT_CONFIG_INCREASE],
		SQL_STMT_CONFIG[STMT_CONFIG_INCREASE].bind[BIND_CONFIG_ID],
		CONFIG_ID_LIFETIME);
	if (ret != SQLITE_OK) {
		LOG_ERROR("Cannot bind ?%d to %d in stmt %s, error = %d",
			SQL_STMT_CONFIG[STMT_CONFIG_INCREASE].bind[BIND_CONFIG_ID],
			CONFIG_ID_LIFETIME,
			SQL_STMT_CONFIG[STMT_CONFIG_INCREASE].stmt, ret);
		return -1;
	}

	ret = sqlite3_bind_int(pgdata->static_data.stmt_config[STMT_CONFIG_INCREASE],
		SQL_STMT_CONFIG[STMT_CONFIG_INCREASE].bind[BIND_CONFIG_VALUE],
		pgdata->static_data.new_lifetime - pgdata->static_data.original_lifetime);
	if (ret != SQLITE_OK) {
		LOG_ERROR("Cannot bind ?%d to %d in stmt %s, error = %d",
			SQL_STMT_CONFIG[STMT_CONFIG_INCREASE].bind[BIND_CONFIG_VALUE],
			pgdata->static_data.new_lifetime - pgdata->static_data.original_lifetime,
			SQL_STMT_CONFIG[STMT_CONFIG_INCREASE].stmt, ret);
		return -1;
	}

	ret = sqlite3_step(pgdata->static_data.stmt_config[STMT_CONFIG_INCREASE]);
	if (ret != SQLITE_ROW) {
		LOG_ERROR("sqlite3_step returns %d", ret);
		return ret;
	}

	return 0;
}

static int ConfigDatabase(ChewingData *pgdata)
{
	int ret;

	assert(pgdata);
	assert(pgdata->static_data.db);

	ret = sqlite3_exec(pgdata->static_data.db, "PRAGMA synchronous=OFF", NULL, NULL, NULL);
	if (ret != SQLITE_OK) {
		LOG_ERROR("Cannot set synchronous=OFF, error = %d", ret);
		return -1;
	}

	return 0;
}

static int CreateStmt(ChewingData *pgdata)
{
	int i;
	int ret;

	assert(pgdata);

	STATIC_ASSERT(ARRAY_SIZE(SQL_STMT_CONFIG) == ARRAY_SIZE(pgdata->static_data.stmt_config),
		stmt_config_size_mismatch);
	STATIC_ASSERT(ARRAY_SIZE(SQL_STMT_USERPHRASE) == ARRAY_SIZE(pgdata->static_data.stmt_userphrase),
		stmt_userphrase_size_mismatch);

	for (i = 0; i < ARRAY_SIZE(SQL_STMT_CONFIG); ++i) {
		ret = sqlite3_prepare_v2(pgdata->static_data.db,
			SQL_STMT_CONFIG[i].stmt, -1,
			&pgdata->static_data.stmt_config[i], NULL);
		if (ret != SQLITE_OK) {
			LOG_ERROR("Cannot create stmt %s", SQL_STMT_CONFIG[i].stmt);
			return -1;
		}
	}

	for (i = 0; i < ARRAY_SIZE(SQL_STMT_USERPHRASE); ++i) {
		ret = sqlite3_prepare_v2(pgdata->static_data.db,
			SQL_STMT_USERPHRASE[i].stmt, -1,
			&pgdata->static_data.stmt_userphrase[i], NULL);
		if (ret != SQLITE_OK) {
			LOG_ERROR("Cannot create stmt %s", SQL_STMT_USERPHRASE[i].stmt);
			return -1;
		}
	}

	return 0;
}

int InitSql(ChewingData *pgdata)
{
	int ret;

	assert(!pgdata->static_data.db);

	pgdata->static_data.db = GetSQLiteInstance(pgdata);
	if (!pgdata->static_data.db) {
		LOG_ERROR("GetSQLiteInstance fails");
		goto error;
	}

	ret = ConfigDatabase(pgdata);
	if (ret) {
		LOG_ERROR("ConfigDatabase returns %d", ret);
		goto error;
	}

	ret = CreateTable(pgdata);
	if (ret) {
		LOG_ERROR("CreateTable returns %d", ret);
		goto error;
	}

	ret = CreateStmt(pgdata);
	if (ret) {
		LOG_ERROR("CreateStmt returns %d", ret);
		goto error;
	}

	ret = SetupUserphraseLifeTime(pgdata);
	if (ret) {
		LOG_ERROR("SetupUserphraseLiftTime returns %d", ret);
		goto error;
	}

	// FIXME: Normalize lifttime when necessary.
	// FIXME: Migrate old uhash.dat here.

	return 0;

error:
	TerminateSql(pgdata);
	return -1;
}

void TerminateSql(ChewingData *pgdata)
{
	int i;
	int ret;

	UpdateLifeTime(pgdata);

	for (i = 0; i < ARRAY_SIZE(pgdata->static_data.stmt_config); ++i) {
		sqlite3_finalize(pgdata->static_data.stmt_config[i]);
		pgdata->static_data.stmt_config[i] = NULL;
	}

	for (i = 0; i < ARRAY_SIZE(pgdata->static_data.stmt_userphrase); ++i) {
		sqlite3_finalize(pgdata->static_data.stmt_userphrase[i]);
		pgdata->static_data.stmt_userphrase[i] = NULL;
	}

	ret = sqlite3_close(pgdata->static_data.db);
	assert(SQLITE_OK == ret);
	pgdata->static_data.db = NULL;
}
