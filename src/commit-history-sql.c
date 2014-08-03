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

CommitHistoryData *GetCommitHistoryFirst(ChewingData *pgdata, const char wordSeq[])
{
    int ret;

    assert(pgdata);
    assert(wordSeq);

    assert(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT]);

    ret = sqlite3_reset(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT]);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_reset returns %d", ret);
        return NULL;
    }

    /* bind phrase */
    ret = sqlite3_bind_text(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT],
                            BIND_COMMIT_HISTORY_PHRASE, wordSeq, -1, SQLITE_STATIC);
    if (ret != SQLITE_OK) {
        LOG_ERROR("sqlite3_bind_text returns %d", ret);
        return NULL;
    }

    return GetCommitHistoryNext(pgdata, wordSeq);
}

CommitHistoryData *GetCommitHistoryNext(ChewingData *pgdata, const char wordSeq[])
{
    int i;
    int ret;
    int word_len;

    assert(pgdata);
    assert(wordSeq);

    ret = sqlite3_step(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT]);
    if (ret != SQLITE_ROW)
        return NULL;

    /* get the length */
    pgdata->commit_history_data.length =
        sqlite3_column_int(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT],
                           SQL_STMT_COMMIT_HISTORY[STMT_COMMIT_HISTORY_SELECT]
                               .column[COLUMN_COMMIT_HISTORY_LENGTH]);

    /* get the phrase */
    pgdata->commit_history_data.wordSeq =
        (char *) sqlite3_column_text(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT],
                                     SQL_STMT_COMMIT_HISTORY[STMT_COMMIT_HISTORY_SELECT]
                                         .column[COLUMN_COMMIT_HISTORY_PHRASE]);

    /* get the phones */
    word_len = ueStrLen(wordSeq);
    for (i = 0; i < word_len; ++i) {
        pgdata->commit_history_data.phoneSeq[i] =
            sqlite3_column_int(pgdata->static_data.stmt_commit_history[STMT_COMMIT_HISTORY_SELECT],
                               SQL_STMT_COMMIT_HISTORY[STMT_COMMIT_HISTORY_SELECT]
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
