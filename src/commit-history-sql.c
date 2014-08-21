/**
 * commit-history-sql.c
 *
 * Copyright (c) 2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include <assert.h>

#include "commit-history-private.h"
#include "chewing-private.h"
#include "chewing-utf8-util.h"
#include "private.h"
#include "key2pho-private.h"
#include "json.h"

static int CommitHistoryBindPhone(ChewingData *pgdata, int index, const uint16_t phoneSeq[], int len)
{
    int i;
    int ret;

    assert(pgdata);
    assert(phoneSeq);

    if (len > MAX_PHRASE_LEN) {
        LOG_WARN("phoneSeq length %d > MAX_PHRASE_LEN(%d)", len, MAX_PHRASE_LEN);
        return -1;
    }

    ret = sqlite3_bind_int(pgdata->static_data.stmt_commit_history[index], BIND_COMMIT_HISTORY_LENGTH, len);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_bind_int returns %d", ret);
        return ret;
    }

    for (i = 0; i < len; ++i) {
        ret = sqlite3_bind_int(pgdata->static_data.stmt_commit_history[index], BIND_COMMIT_HISTORY_PHONE_0 + i, phoneSeq[i]);
        if (ret != SQLITE_OK) {
            LOG_ERROR("sqlite3_bind_int returns %d", ret);
            return ret;
        }
    }

    for (i = len; i < MAX_PHRASE_LEN; ++i) {
        ret = sqlite3_bind_int(pgdata->static_data.stmt_commit_history[index], BIND_COMMIT_HISTORY_PHONE_0 + i, 0);
        if (ret != SQLITE_OK) {
            LOG_ERROR("sqlite3_bind_int returns %d", ret);
            return ret;
        }
    }

    return SQLITE_OK;
}

static int WriteCommit(void *commits_obj, int column_num,
                       char **text, char **column_name)
{
    int i;
    json_object *row_obj;

    row_obj = json_object_new_array();

    json_object_array_add(row_obj, json_object_new_int(atoi(text[COLUMN_COMMIT_HISTORY_LENGTH])));

    json_object_array_add(row_obj, json_object_new_string(text[COLUMN_COMMIT_HISTORY_PHRASE]));

    for (i = COLUMN_COMMIT_HISTORY_PHONE_0; i <= COLUMN_COMMIT_HISTORY_PHONE_10; ++i) {
        json_object_array_add(row_obj, json_object_new_int(atoi(text[i])));
    }

    json_object_array_add(commits_obj, row_obj);

    return 0;
}

int CommitHistoryInsert(ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[])
{
    int ret;
    int action;
    int phone_len;
    int word_len;

    assert(pgdata);
    assert(phoneSeq);
    assert(wordSeq);

    phone_len = GetPhoneLen(phoneSeq);
    word_len = ueStrLen(wordSeq);

    if (phone_len != word_len) {
        LOG_WARN("Do not insert commit history because phoneSeq length %d != wordSeq length %d", phone_len, word_len);
        return COMMIT_INSERT_FAIL;
    }

    if (word_len > MAX_PHRASE_LEN) {
        LOG_WARN("wordSeq length %d > MAX_PHRASE_LEN (%d)", word_len, MAX_PHRASE_LEN);
        return COMMIT_INSERT_FAIL;
    }

    assert(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_INSERT]);

    /* bind phrase */
    ret = sqlite3_bind_text(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_INSERT],
                            BIND_COMMIT_HISTORY_PHRASE, wordSeq, -1, SQLITE_STATIC);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_bind_text returns %d", ret);
        action = COMMIT_INSERT_FAIL;
        goto end;
    }

    /* bind length and phones */
    ret = CommitHistoryBindPhone(pgdata, STMT_COMMIT_HISTORY_INSERT, phoneSeq, phone_len);
    if (ret != SQLITE_OK) {
        LOG_ERROR("CommitHistoryBindPhone returns %d", ret);
        action = COMMIT_INSERT_FAIL;
        goto end;
    }

    ret = sqlite3_step(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_INSERT]);
    if (ret != SQLITE_DONE) {
        LOG_ERROR("sqlite3_step returns %d", ret);
        action = COMMIT_INSERT_FAIL;
        goto end;
    }

    action = COMMIT_INSERT_SUCCESS;

  end:
    ret = sqlite3_reset(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_INSERT]);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_reset returns %d", ret);
    }

    return action;
}

CommitHistoryData *GetCommitHistoryByPhraseFirst(ChewingData *pgdata, const char wordSeq[])
{
    int ret;

    assert(pgdata);
    assert(wordSeq);

    assert(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT_BY_PHRASE]);

    ret = sqlite3_reset(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT_BY_PHRASE]);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_reset returns %d", ret);
        return NULL;
    }

    /* bind phrase */
    ret = sqlite3_bind_text(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT_BY_PHRASE],
                            BIND_COMMIT_HISTORY_PHRASE, wordSeq, -1, SQLITE_STATIC);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_bind_text returns %d", ret);
        return NULL;
    }

    return GetCommitHistoryByPhraseNext(pgdata, wordSeq);
}

CommitHistoryData *GetCommitHistoryByPhraseNext(ChewingData *pgdata, const char wordSeq[])
{
    int i;
    int ret;
    int word_len;

    assert(pgdata);
    assert(wordSeq);

    ret = sqlite3_step(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT_BY_PHRASE]);
    if (ret != SQLITE_ROW)
        return NULL;

    /* get the length */
    pgdata->commit_history_data.length =
        sqlite3_column_int(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT_BY_PHRASE],
                           SQL_STMT_COMMIT_HISTORY[STMT_COMMIT_HISTORY_SELECT_BY_PHRASE]
                               .column[COLUMN_COMMIT_HISTORY_LENGTH]);

    /* get the phrase */
    pgdata->commit_history_data.wordSeq =
        (char *) sqlite3_column_text(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT_BY_PHRASE],
                                     SQL_STMT_COMMIT_HISTORY[STMT_COMMIT_HISTORY_SELECT_BY_PHRASE]
                                         .column[COLUMN_COMMIT_HISTORY_PHRASE]);

    /* get the phones */
    word_len = ueStrLen(wordSeq);
    for (i = 0; i < word_len; ++i) {
        pgdata->commit_history_data.phoneSeq[i] =
            sqlite3_column_int(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT_BY_PHRASE],
                               SQL_STMT_COMMIT_HISTORY[STMT_COMMIT_HISTORY_SELECT_BY_PHRASE]
                                   .column[COLUMN_COMMIT_HISTORY_PHONE_0 + i]);
    }

    return &pgdata->commit_history_data;
}

int CommitHistoryRemove(ChewingData *pgdata, const char wordSeq[])
{
    int ret;
    int affected = 0;

    assert(pgdata);
    assert(wordSeq);

    assert(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_DELETE]);

    ret = sqlite3_bind_text(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_DELETE],
                            BIND_COMMIT_HISTORY_PHRASE, wordSeq, -1, SQLITE_STATIC);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_bind_text returns %d", ret);
        goto end;
    }

    ret = sqlite3_step(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_DELETE]);
    if (ret != SQLITE_DONE) {
        LOG_ERROR("sqlite3_step returns %d", ret);
        goto end;
    }

    affected = sqlite3_changes(pgdata->static_data.db);

  end:
    ret = sqlite3_reset(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_DELETE]);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_reset returns %d", ret);
    }

    return affected;
}

int ExportCommitHistory(ChewingData *pgdata, FILE *fp)
{
    int ret;
    const char *col_name;
    sqlite3_stmt *stmt;
    struct json_object *json_obj;
    struct json_object *headings_obj;
    struct json_object *commits_obj;

    assert(pgdata);
    assert(fp);

    json_obj = json_object_new_object();
    headings_obj = json_object_new_array();
    commits_obj = json_object_new_array();

    /* add column name to json */
    ret = sqlite3_prepare_v2(pgdata->static_data.db,
                             "pragma table_info('commit_history')",
                             -1, &stmt, NULL);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_exec returns %d, ret");
        return COMMIT_EXPORT_FAIL;
    }
    while (sqlite3_step(stmt) == SQLITE_ROW) {
        col_name = (const char*) sqlite3_column_text(stmt, 1);
        json_object_array_add(headings_obj,
                              json_object_new_string(col_name));
    }

    ret = sqlite3_finalize(stmt);

    /* add commit history to json */
    ret = sqlite3_exec(pgdata->static_data.db,
                       "SELECT length, phrase, "
                       "phone_0, phone_1, phone_2, phone_3, phone_4, phone_5, "
                       "phone_6, phone_7, phone_8, phone_9, phone_10 "
                       "FROM commit_history",
                       WriteCommit, commits_obj, NULL);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_exec returns %d, ret");
        return COMMIT_EXPORT_FAIL;
    }

    json_object_object_add(json_obj, "headings", headings_obj);
    json_object_object_add(json_obj, "commits", commits_obj);

    fprintf(fp, "%s\n", json_object_to_json_string_ext(json_obj, JSON_C_TO_STRING_PRETTY));
    /* free the json_obj */
    json_object_put(json_obj);

    return COMMIT_EXPORT_SUCCESS;
}
