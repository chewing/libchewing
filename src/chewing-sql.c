/**
 * chewing-sql.c
 *
 * Copyright (c) 2013, 2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include "chewing-sql.h"
#include "chewing-private.h"

#include <assert.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "memory-private.h"
#include "plat_types.h"
#include "private.h"
#include "sqlite3.h"
#include "userphrase-private.h"

const SqlStmtUserphrase SQL_STMT_USERPHRASE[STMT_USERPHRASE_COUNT] = {
    {
     "SELECT length, phrase, "
     "phone_0, phone_1, phone_2, phone_3, phone_4, phone_5, "
     "phone_6, phone_7, phone_8, phone_9, phone_10 " "FROM userphrase_v1",
     {-1, -1, -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12},
     },
    {
     "SELECT time, orig_freq, max_freq, user_freq, phrase "
     "FROM userphrase_v1 WHERE length = ?5 AND "
     "phone_0 = ?10 AND phone_1 = ?11 AND phone_2 = ?12 AND "
     "phone_3 = ?13 AND phone_4 = ?14 AND phone_5 = ?15 AND "
     "phone_6 = ?16 AND phone_7 = ?17 AND phone_8 = ?18 AND " "phone_9 = ?19 AND phone_10 = ?20",
     {0, 1, 2, 3, -1, 4, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1},
     },
    {
     "SELECT time, orig_freq, max_freq, user_freq "
     "FROM userphrase_v1 WHERE length = ?5 AND phrase = ?6 AND "
     "phone_0 = ?10 AND phone_1 = ?11 AND phone_2 = ?12 AND "
     "phone_3 = ?13 AND phone_4 = ?14 AND phone_5 = ?15 AND "
     "phone_6 = ?16 AND phone_7 = ?17 AND phone_8 = ?18 AND " "phone_9 = ?19 AND phone_10 = ?20",
     {0, 1, 2, 3, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1},
     },
    {
     "INSERT OR REPLACE INTO userphrase_v1 ("
     "time, orig_freq, max_freq, user_freq, length, phrase, "
     "phone_0, phone_1, phone_2, phone_3, phone_4, phone_5, "
     "phone_6, phone_7, phone_8, phone_9, phone_10) "
     "VALUES (?1, ?2, ?3, ?4, ?5, ?6, " "?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
     {-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1},
     },
    {
     "DELETE FROM userphrase_v1 WHERE length = ?5 AND phrase = ?6 AND "
     "phone_0 = ?10 AND phone_1 = ?11 AND phone_2 = ?12 AND "
     "phone_3 = ?13 AND phone_4 = ?14 AND phone_5 = ?15 AND "
     "phone_6 = ?16 AND phone_7 = ?17 AND phone_8 = ?18 AND " "phone_9 = ?19 AND phone_10 = ?20",
     {-1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1},
     },
    {
     "SELECT MAX(user_freq) FROM userphrase_v1 WHERE length = ?5 AND "
     "phone_0 = ?10 AND phone_1 = ?11 AND phone_2 = ?12 AND "
     "phone_3 = ?13 AND phone_4 = ?14 AND phone_5 = ?15 AND "
     "phone_6 = ?16 AND phone_7 = ?17 AND phone_8 = ?18 AND " "phone_9 = ?19 AND phone_10 = ?20",
     {-1, -1, -1, 0, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1},
     },
};

const SqlStmtConfig SQL_STMT_CONFIG[STMT_CONFIG_COUNT] = {
    {
     "SELECT value FROM config_v1 WHERE id = ?1",
     {-1, 0},
     },
    {
     "INSERT OR IGNORE INTO config_v1 (id, value) VALUES (?1, ?2)",
     {-1, -1},
     },
    {
     "UPDATE config_v1 SET value = value + ?2 WHERE id = ?1",
     {-1, -1},
     },
};

#define HASH_FIELD_SIZE		(125)
#define HASH_FIELD_START	(8)
#define HASH_LENGTH_OFFSET	(16)
#define HASH_NAME		"uhash.dat"
#define HASH_OLD_NAME		"uhash.old"
#define HASH_SIGS		"CBiH"

static sqlite3 *GetSQLiteInstance(ChewingData *pgdata, const char *path)
{
    int ret;
    sqlite3 *db = NULL;

    assert(pgdata);
    assert(path);

    ret = sqlite3_open(path, &db);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_open returns %d", ret);
        goto end;
    }

  end:
    return db;
}


static int CreateTable(ChewingData *pgdata)
{
    int ret;

    STATIC_ASSERT(MAX_PHRASE_LEN == 11);

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
                       "phrase)" ")", NULL, NULL, NULL);
    if (ret != SQLITE_OK) {
        LOG_ERROR("Cannot create table userphrase_v1, error = %d", ret);
        return -1;
    }

    ret = sqlite3_exec(pgdata->static_data.db,
                       "CREATE TABLE IF NOT EXISTS config_v1 ("
                       "id INTEGER,"
                       "value INTEGER,"
                       "PRIMARY KEY (id)" ")", NULL, NULL, NULL);
    if (ret != SQLITE_OK) {
        LOG_ERROR("Cannot create table config_v1, error = %d", ret);
        return -1;
    }

    return 0;
}

static int SetupUserphraseLifeTime(ChewingData *pgdata)
{
    int ret;

    assert(pgdata->static_data.stmt_config[STMT_CONFIG_INSERT]);

    ret = sqlite3_bind_int(pgdata->static_data.stmt_config[STMT_CONFIG_INSERT], BIND_CONFIG_ID, CONFIG_ID_LIFETIME);
    if (ret != SQLITE_OK) {
        LOG_ERROR("Cannot bind ?%d to %d in stmt %s, error = %d",
                  BIND_CONFIG_ID, CONFIG_ID_LIFETIME, SQL_STMT_CONFIG[STMT_CONFIG_INSERT].stmt, ret);
        return -1;
    }

    ret = sqlite3_bind_int(pgdata->static_data.stmt_config[STMT_CONFIG_INSERT], BIND_CONFIG_VALUE, 0);
    if (ret != SQLITE_OK) {
        LOG_ERROR("Cannot bind ?%d to %d in stmt %s, error = %d",
                  BIND_CONFIG_VALUE, 0, SQL_STMT_CONFIG[STMT_CONFIG_INSERT].stmt, ret);
        return -1;
    }

    ret = sqlite3_step(pgdata->static_data.stmt_config[STMT_CONFIG_INSERT]);
    if (ret != SQLITE_DONE) {
        LOG_ERROR("sqlite3_step returns %d", ret);
        return -1;
    }

    ret = sqlite3_reset(pgdata->static_data.stmt_config[STMT_CONFIG_INSERT]);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_reset returns %d", ret);
        return -1;
    }

    assert(pgdata->static_data.stmt_config[STMT_CONFIG_SELECT]);

    ret = sqlite3_bind_int(pgdata->static_data.stmt_config[STMT_CONFIG_SELECT], BIND_CONFIG_ID, CONFIG_ID_LIFETIME);
    if (ret != SQLITE_OK) {
        LOG_ERROR("Cannot bind ?%d to %d in stmt %s, error = %d",
                  BIND_CONFIG_ID, CONFIG_ID_LIFETIME, SQL_STMT_CONFIG[STMT_CONFIG_SELECT].stmt, ret);
        return -1;
    }

    ret = sqlite3_step(pgdata->static_data.stmt_config[STMT_CONFIG_SELECT]);
    if (ret != SQLITE_ROW) {
        LOG_ERROR("sqlite3_step returns %d", ret);
        return -1;
    }

    pgdata->static_data.original_lifetime = sqlite3_column_int(pgdata->static_data.stmt_config[STMT_CONFIG_SELECT],
                                                               SQL_STMT_CONFIG[STMT_CONFIG_SELECT].column
                                                               [COLUMN_CONFIG_VALUE]);
    pgdata->static_data.new_lifetime = pgdata->static_data.original_lifetime;

    ret = sqlite3_reset(pgdata->static_data.stmt_config[STMT_CONFIG_SELECT]);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_reset returns %d", ret);
        return -1;
    }

    return 0;
}

static int UpdateLifeTime(ChewingData *pgdata)
{
    int ret;
    int result = 0;

    if (!pgdata->static_data.stmt_config[STMT_CONFIG_INCREASE]) {
        LOG_ERROR("pgdata->static_data.stmt_config[STMT_CONFIG_INCREASE] is NULL");
        result = -1;
        goto end;
    }

    ret = sqlite3_clear_bindings(pgdata->static_data.stmt_config[STMT_CONFIG_INCREASE]);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_clear_bindings returns %d", ret);
        result = -1;
        goto end;
    }

    ret = sqlite3_bind_int(pgdata->static_data.stmt_config[STMT_CONFIG_INCREASE], BIND_CONFIG_ID, CONFIG_ID_LIFETIME);
    if (ret != SQLITE_OK) {
        LOG_ERROR("Cannot bind ?%d to %d in stmt %s, error = %d",
                  BIND_CONFIG_ID, CONFIG_ID_LIFETIME, SQL_STMT_CONFIG[STMT_CONFIG_INCREASE].stmt, ret);
        result = -1;
        goto end;
    }

    ret = sqlite3_bind_int(pgdata->static_data.stmt_config[STMT_CONFIG_INCREASE],
                           BIND_CONFIG_VALUE, pgdata->static_data.new_lifetime - pgdata->static_data.original_lifetime);
    if (ret != SQLITE_OK) {
        LOG_ERROR("Cannot bind ?%d to %d in stmt %s, error = %d",
                  BIND_CONFIG_VALUE,
                  pgdata->static_data.new_lifetime - pgdata->static_data.original_lifetime,
                  SQL_STMT_CONFIG[STMT_CONFIG_INCREASE].stmt, ret);
        result = -1;
        goto end;
    }

    ret = sqlite3_step(pgdata->static_data.stmt_config[STMT_CONFIG_INCREASE]);
    if (ret != SQLITE_DONE) {
        LOG_ERROR("sqlite3_step returns %d", ret);
        result = -1;
        goto end;
    }

  end:
    ret = sqlite3_reset(pgdata->static_data.stmt_config[STMT_CONFIG_INCREASE]);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_reset returns %d", ret);
        result = -1;
    }

    return result;
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

    STATIC_ASSERT(ARRAY_SIZE(SQL_STMT_CONFIG) == ARRAY_SIZE(pgdata->static_data.stmt_config));
    STATIC_ASSERT(ARRAY_SIZE(SQL_STMT_USERPHRASE) == ARRAY_SIZE(pgdata->static_data.stmt_userphrase));

    for (i = 0; i < ARRAY_SIZE(SQL_STMT_CONFIG); ++i) {
        ret = sqlite3_prepare_v2(pgdata->static_data.db,
                                 SQL_STMT_CONFIG[i].stmt, -1, &pgdata->static_data.stmt_config[i], NULL);
        if (ret != SQLITE_OK) {
            LOG_ERROR("Cannot create stmt %s", SQL_STMT_CONFIG[i].stmt);
            return -1;
        }
    }

    for (i = 0; i < ARRAY_SIZE(SQL_STMT_USERPHRASE); ++i) {
        ret = sqlite3_prepare_v2(pgdata->static_data.db,
                                 SQL_STMT_USERPHRASE[i].stmt, -1, &pgdata->static_data.stmt_userphrase[i], NULL);
        if (ret != SQLITE_OK) {
            LOG_ERROR("Cannot create stmt %s", SQL_STMT_USERPHRASE[i].stmt);
            return -1;
        }
    }

    return 0;
}

static void MigrateOldFormat(ChewingData *pgdata, const char *path)
{
    char *uhash;
    char *old_uhash;
    FILE *fd = NULL;
    char buf[HASH_FIELD_SIZE];
    uint16_t phoneSeq[MAX_PHRASE_LEN + 1];
    char *pos;
    int len;
    int i;
    int ret;

    assert(pgdata);
    assert(path);

    len = strlen(path) + 1 + strlen(HASH_NAME) + 1;
    uhash = calloc(sizeof(*uhash), len);
    if (!uhash) {
        LOG_ERROR("calloc returns %#p", uhash);
        exit(-1);
    }
    snprintf(uhash, len, "%s" PLAT_SEPARATOR "%s", path, HASH_NAME);

    len = strlen(path) + 1 + strlen(HASH_OLD_NAME) + 1;
    old_uhash = calloc(sizeof(*old_uhash), len);
    if (!old_uhash) {
        LOG_ERROR("calloc returns %#p", old_uhash);
        exit(-1);
    }
    snprintf(old_uhash, len, "%s" PLAT_SEPARATOR "%s", path, HASH_OLD_NAME);

    /*
     * The binary format is described as following:
     *
     * 0 ~ 3                signature (CBiH)
     * 4 ~ 7                lifttime, platform endianness
     * 8 ~ 8 + 125 * n      array of hash item, 125 bytes each
     *
     * 0 ~ 3                user frequency, platform endianness
     * 4 ~ 7                recent time, platform endianness
     * 8 ~ 11               max frequency, platform endianness
     * 12 ~ 15              original frequency, platform endianness
     * 16                   phone length
     * 17 ~ 17 + 2 * n      phone sequence, uint16_t, platform endianness
     * 17 + 2 * n + 1       phrase length in bytes
     * 17 + 2 * n + 2 ~ y   phrase in UTF-8
     *
     */

    fd = fopen(uhash, "r");
    if (!fd)
        goto end;

    LOG_INFO("Migrate old format from %s", uhash);
    ret = fread(buf, 4, 1, fd);
    if (ret != 1) {
        LOG_WARN("fread returns %d", ret);
        goto end_remove_hash;
    }

    if (memcmp(buf, HASH_SIGS, 4) != 0) {
        LOG_WARN("signature is not %d", HASH_SIGS);
        goto end_remove_hash;
    }

    ret = fseek(fd, 8, SEEK_SET);
    if (ret) {
        LOG_WARN("fseek returns %d", ret);
        goto end_remove_hash;
    }

    while (fread(buf, HASH_FIELD_SIZE, 1, fd) == 1) {
        pos = &buf[HASH_LENGTH_OFFSET];
        len = *pos;
        ++pos;

        if (len > MAX_PHRASE_LEN || len < 1) {
            LOG_WARN("skip field due to len = %d", len);
            continue;
        }

        for (i = 0; i < len; ++i) {
            phoneSeq[i] = GetUint16PreservedEndian(pos);
            pos += 2;
        }
        phoneSeq[len] = 0;

        ++pos;
        UserUpdatePhrase(pgdata, phoneSeq, pos);
    }

  end_remove_hash:
    if (fd)
        fclose(fd);
    PLAT_RENAME(uhash, old_uhash);
  end:
    free(old_uhash);
    free(uhash);
}

int InitUserphrase(ChewingData *pgdata, const char *path)
{
    int ret;

    assert(!pgdata->static_data.db);
    assert(path);

    pgdata->static_data.db = GetSQLiteInstance(pgdata, path);
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

    /* FIXME: Normalize lifttime when necessary. */

    MigrateOldFormat(pgdata, path);

    return 0;

  error:
    TerminateUserphrase(pgdata);
    return -1;
}

void TerminateUserphrase(ChewingData *pgdata)
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
