/**
 * userphrase.c
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004, 2006
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include <assert.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#include "chewing-utf8-util.h"
#include "hash-private.h"
#include "dict-private.h"
#include "tree-private.h"
#include "userphrase-private.h"
#include "private.h"
#include "key2pho-private.h"

static int UserBindPhone(
	ChewingData *pgdata,
	int index,
	const uint16_t phoneSeq[])
{
	int i;
	int len;
	int ret;

	assert(pgdata);
	assert(phoneSeq);

	len = GetPhoneLen(phoneSeq);

	ret = sqlite3_bind_int(
		pgdata->static_data.stmt_userphrase[index],
		SQL_STMT_USERPHRASE[index].bind[BIND_USERPHRASE_LENGTH], len);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_bind_int returns %d", ret);
		return ret;
	}

	for (i = 0; i < len; ++i) {
		ret = sqlite3_bind_int(
			pgdata->static_data.stmt_userphrase[index],
			SQL_STMT_USERPHRASE[index].bind[BIND_USERPHRASE_PHONE_0 + i],
			phoneSeq[i]);
		if (ret != SQLITE_OK) {
			LOG_ERROR("sqlite3_bind_int returns %d", ret);
			return ret;
		}
	}

	for (i = len; i < MAX_PHRASE_LEN; ++i) {
		ret = sqlite3_bind_int(
			pgdata->static_data.stmt_userphrase[index],
			SQL_STMT_USERPHRASE[index].bind[BIND_USERPHRASE_PHONE_0 + i],
			0);
		if (ret != SQLITE_OK) {
			LOG_ERROR("sqlite3_bind_int returns %d", ret);
			return ret;
		}
	}

	return SQLITE_OK;
}


/* load the orginal frequency from the static dict */
static int LoadOriginalFreq( ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[], int len )
{
	const TreeType *tree_pos;
	int retval;
	Phrase *phrase = ALC( Phrase, 1 );

	tree_pos = TreeFindPhrase( pgdata, 0, len - 1, phoneSeq );
	if ( tree_pos ) {
		GetPhraseFirst( pgdata, phrase, tree_pos );
		do {
			/* find the same phrase */
			if ( ! strcmp(
				phrase->phrase,
				wordSeq ) ) {
				retval = phrase->freq;
				free( phrase );
				return retval;
			}
		} while ( GetVocabNext( pgdata, phrase ) );
	}

	free( phrase );
	return FREQ_INIT_VALUE;
}

/* find the maximum frequency of the same phrase */
static int LoadMaxFreq(ChewingData *pgdata, const uint16_t phoneSeq[], int len)
{
	const TreeType *tree_pos;
	Phrase *phrase = ALC(Phrase, 1);
	int maxFreq = FREQ_INIT_VALUE;
	int ret;

	tree_pos = TreeFindPhrase(pgdata, 0, len - 1, phoneSeq);
	if (tree_pos) {
		GetPhraseFirst(pgdata, phrase, tree_pos);
		do {
			if (phrase->freq > maxFreq)
				maxFreq = phrase->freq;
		} while(GetVocabNext(pgdata, phrase));
	}
	free(phrase);

	ret = sqlite3_reset(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_GET_MAX_FREQ]);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_reset returns %d", ret);
		return maxFreq;
	}

	ret = UserBindPhone(pgdata, STMT_USERPHRASE_GET_MAX_FREQ, phoneSeq);
	if (ret != SQLITE_OK) {
		LOG_ERROR("UserBindPhone returns %d", ret);
		return maxFreq;
	}

	ret = sqlite3_step(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_GET_MAX_FREQ]);
	if (ret !=  SQLITE_ROW)
		return maxFreq;

	maxFreq = sqlite3_column_int(
		pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_GET_MAX_FREQ],
		SQL_STMT_USERPHRASE[STMT_USERPHRASE_GET_MAX_FREQ].column[COLUMN_USERPHRASE_USER_FREQ]);

	return maxFreq;
}

/* compute the new updated freqency */
static int UpdateFreq( int freq, int maxfreq, int origfreq, int deltatime )
{
	int delta;

	/* Short interval */
	if ( deltatime < 4000 ) {
		delta = ( freq >= maxfreq ) ?
			min(
				( maxfreq - origfreq ) / 5 + 1,
				SHORT_INCREASE_FREQ ) :
			max(
				( maxfreq - origfreq ) / 5 + 1,
				SHORT_INCREASE_FREQ );
		return min( freq + delta, MAX_ALLOW_FREQ );
	}
	/* Medium interval */
	else if ( deltatime < 50000 ) {
		delta = ( freq >= maxfreq ) ?
			min(
				( maxfreq - origfreq ) / 10 + 1,
				MEDIUM_INCREASE_FREQ ) :
			max(
				( maxfreq - origfreq ) / 10 + 1,
				MEDIUM_INCREASE_FREQ );
		return min( freq + delta, MAX_ALLOW_FREQ );
	}
	/* long interval */
	else {
		delta = max( ( freq - origfreq ) / 5, LONG_DECREASE_FREQ );
		return max( freq - delta, origfreq );
	}
}

static int GetCurrentLifeTime( ChewingData *pgdata )
{
	return pgdata->static_data.new_lifetime;
}

static void LogUserPhrase(
	ChewingData *pgdata,
	const uint16_t phoneSeq[],
	const char wordSeq[],
	int orig_freq,
	int max_freq,
	int user_freq,
	int recent_time)
{
	/* Size of each phone is len("0x1234 ") = 7 */
	char buf[7 * MAX_PHRASE_LEN + 1] = { 0 };
	int i;

	for (i = 0; i < MAX_PHRASE_LEN; ++i) {
		if (phoneSeq[i] == 0)
			break;
		snprintf(buf + 7 * i, 7 + 1, "%#06x ", phoneSeq[i]);
	}

	LOG_INFO( "userphrase %s, phone = %s, orig_freq = %d, max_freq = %d, user_freq = %d, recent_time = %d\n",
		wordSeq, buf, orig_freq, max_freq, user_freq, recent_time);
}

void UserUpdatePhraseBegin( ChewingData *pgdata )
{
	sqlite3_exec( pgdata->static_data.db, "BEGIN", 0, 0, 0 );
}

int UserUpdatePhrase(ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[])
{
	int ret;
	int action;
	int phone_len;
	int word_len;

	int orig_freq;
	int max_freq;
	int user_freq;
	int recent_time;
	int orig_time;

	assert(pgdata);
	assert(phoneSeq);
	assert(wordSeq);

	phone_len = GetPhoneLen(phoneSeq);
	word_len = ueStrLen(wordSeq);

	if (phone_len != word_len) {
		LOG_WARN("Do not update userphrase because phoneSeq length %d != wordSeq length %d", phone_len, word_len);
		return USER_UPDATE_FAIL;
	}

	ret = sqlite3_reset(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT_BY_PHONE_PHRASE]);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_reset returns %d", ret);
		return USER_UPDATE_FAIL;
	}

	ret = UserBindPhone(pgdata, STMT_USERPHRASE_SELECT_BY_PHONE_PHRASE, phoneSeq);
	if (ret != SQLITE_OK) {
		LOG_ERROR("UserBindPhone returns %d", ret);
		return USER_UPDATE_FAIL;
	}

	ret = sqlite3_bind_text(
		pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT_BY_PHONE_PHRASE],
		SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT_BY_PHONE_PHRASE].bind[BIND_USERPHRASE_PHRASE],
		wordSeq, -1, SQLITE_STATIC);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_bind_text returns %d", ret);
		return USER_UPDATE_FAIL;
	}

	recent_time = GetCurrentLifeTime(pgdata);

	ret = sqlite3_step(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT_BY_PHONE_PHRASE]);
	if (ret == SQLITE_ROW) {
		action = USER_UPDATE_MODIFY;

		orig_freq = sqlite3_column_int(
			pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT_BY_PHONE_PHRASE],
			SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT_BY_PHONE_PHRASE].column[COLUMN_USERPHRASE_ORIG_FREQ]);

		max_freq = LoadMaxFreq(pgdata, phoneSeq, phone_len);

		user_freq = sqlite3_column_int(
			pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT_BY_PHONE_PHRASE],
			SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT_BY_PHONE_PHRASE].column[COLUMN_USERPHRASE_USER_FREQ]);

		orig_time = sqlite3_column_int(
			pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT_BY_PHONE_PHRASE],
			SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT_BY_PHONE_PHRASE].column[COLUMN_USERPHRASE_TIME]);

		user_freq = UpdateFreq(user_freq, max_freq, orig_freq, recent_time - orig_time);
	} else {
		action = USER_UPDATE_INSERT;

		orig_freq = LoadOriginalFreq(pgdata, phoneSeq, wordSeq, word_len);
		max_freq = LoadMaxFreq(pgdata, phoneSeq, phone_len);
		user_freq = orig_freq;
	}

	ret = sqlite3_reset(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_UPSERT]);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_reset returns %d", ret);
		return USER_UPDATE_FAIL;
	}

	ret = sqlite3_bind_int(
		pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_UPSERT],
		SQL_STMT_USERPHRASE[STMT_USERPHRASE_UPSERT].bind[BIND_USERPHRASE_TIME],
		recent_time);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_bind_int returns %d", ret);
		return USER_UPDATE_FAIL;
	}

	ret = sqlite3_bind_int(
		pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_UPSERT],
		SQL_STMT_USERPHRASE[STMT_USERPHRASE_UPSERT].bind[BIND_USERPHRASE_USER_FREQ],
		user_freq);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_bind_int returns %d", ret);
		return USER_UPDATE_FAIL;
	}

	ret = sqlite3_bind_int(
		pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_UPSERT],
		SQL_STMT_USERPHRASE[STMT_USERPHRASE_UPSERT].bind[BIND_USERPHRASE_MAX_FREQ],
		max_freq);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_bind_int returns %d", ret);
		return USER_UPDATE_FAIL;
	}

	ret = sqlite3_bind_int(
		pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_UPSERT],
		SQL_STMT_USERPHRASE[STMT_USERPHRASE_UPSERT].bind[BIND_USERPHRASE_ORIG_FREQ],
		orig_freq);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_bind_int returns %d", ret);
		return USER_UPDATE_FAIL;
	}

	ret = UserBindPhone(pgdata, STMT_USERPHRASE_UPSERT, phoneSeq);
	if (ret != SQLITE_OK) {
		LOG_ERROR("UserBindPhone returns %d", ret);
		return USER_UPDATE_FAIL;
	}

	ret = sqlite3_bind_text(
		pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_UPSERT],
		SQL_STMT_USERPHRASE[STMT_USERPHRASE_UPSERT].bind[BIND_USERPHRASE_PHRASE],
		wordSeq, -1, SQLITE_STATIC);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_bind_text returns %d", ret);
		return USER_UPDATE_FAIL;
	}

	ret = sqlite3_step(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_UPSERT]);
	if (ret != SQLITE_DONE) {
		LOG_ERROR("sqlite3_step returns %d", ret);
		return USER_UPDATE_FAIL;
	}

	LogUserPhrase(pgdata, phoneSeq, wordSeq, orig_freq, max_freq, user_freq, recent_time);

	return action;
}

void UserUpdatePhraseEnd( ChewingData *pgdata )
{
	sqlite3_exec( pgdata->static_data.db, "END", 0, 0, 0 );
}

void UserRemovePhrase(ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[])
{
	int ret;

	assert(pgdata);
	assert(phoneSeq);
	assert(wordSeq);

	ret = sqlite3_reset(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_DELETE]);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_reset returns %d", ret);
		return;
	}

	ret = UserBindPhone(
		pgdata, STMT_USERPHRASE_DELETE, phoneSeq);
	if (ret != SQLITE_OK) {
		LOG_ERROR("UserBindPhone returns %d", ret);
		return;
	}

	ret = sqlite3_bind_text(
		pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_DELETE],
		SQL_STMT_USERPHRASE[STMT_USERPHRASE_DELETE].bind[BIND_USERPHRASE_PHRASE],
		wordSeq, -1, SQLITE_STATIC);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_bind_text returns %d", ret);
		return;
	}

	ret = sqlite3_step(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_DELETE]);
	if (ret != SQLITE_DONE) {
		LOG_ERROR("sqlite3_step returns %d", ret);
		return;
	}
}


UserPhraseData *UserGetPhraseFirst(ChewingData *pgdata, const uint16_t phoneSeq[])
{
	int ret;

	assert(pgdata);
	assert(phoneSeq);

	ret = sqlite3_reset(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT_BY_PHONE]);
	if (ret != SQLITE_OK) {
		LOG_ERROR("sqlite3_reset returns %d", ret);
		return NULL;
	}

	ret = UserBindPhone(pgdata, STMT_USERPHRASE_SELECT_BY_PHONE, phoneSeq);
	if (ret != SQLITE_OK) {
		LOG_ERROR("UserBindPhone returns %d", ret);
		return NULL;
	}

	return UserGetPhraseNext(pgdata, phoneSeq);
}

UserPhraseData *UserGetPhraseNext(ChewingData *pgdata, const uint16_t phoneSeq[])
{
	int ret;

	assert(pgdata);
	assert(phoneSeq);

	ret = sqlite3_step(pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT_BY_PHONE]);
	if (ret !=  SQLITE_ROW) return NULL;

	/* FIXME: shall not remove const here. */
	pgdata->userphrase_data.wordSeq = (char *) sqlite3_column_text(
		pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT_BY_PHONE],
		SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT_BY_PHONE].column[COLUMN_USERPHRASE_PHRASE]);
	pgdata->userphrase_data.phoneSeq = (uint16_t *) phoneSeq;

	pgdata->userphrase_data.recentTime = sqlite3_column_int(
		pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT_BY_PHONE],
		SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT_BY_PHONE].column[COLUMN_USERPHRASE_TIME]);

	pgdata->userphrase_data.userfreq = sqlite3_column_int(
		pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT_BY_PHONE],
		SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT_BY_PHONE].column[COLUMN_USERPHRASE_USER_FREQ]);

	pgdata->userphrase_data.maxfreq = sqlite3_column_int(
		pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT_BY_PHONE],
		SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT_BY_PHONE].column[COLUMN_USERPHRASE_MAX_FREQ]);

	pgdata->userphrase_data.origfreq = sqlite3_column_int(
		pgdata->static_data.stmt_userphrase[STMT_USERPHRASE_SELECT_BY_PHONE],
		SQL_STMT_USERPHRASE[STMT_USERPHRASE_SELECT_BY_PHONE].column[COLUMN_USERPHRASE_ORIG_FREQ]);

	return &pgdata->userphrase_data;
}

void UserGetPhraseEnd(ChewingData *pgdata, const uint16_t phoneSeq[])
{
	/* FIXME: Remove this */
}

void IncreaseLifeTime( ChewingData *pgdata )
{
	++pgdata->static_data.new_lifetime;
}
