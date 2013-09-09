/**
 * hash-private.h
 *
 * Copyright (c) 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_HASH_PRIVATE_H
#define _CHEWING_HASH_PRIVATE_H

#include "global.h"
#include "userphrase-private.h"

#ifdef __MacOSX__
#define CHEWING_HASH_PATH "/Library/ChewingOSX"
#else
#define CHEWING_HASH_PATH "/.chewing"
#endif

#define FIELD_SIZE (125)
#define BIN_HASH_SIG "CBiH"
#define HASH_FILE  "uhash.dat"

#define CHEWING_TABLE_USERPHRASE	"userphrase_v1"
#define CHEWING_CREATE_TABLE_USERPHRASE	"CREATE TABLE IF NOT EXISTS " CHEWING_TABLE_USERPHRASE " (" \
	"time INTEGER," \
	"user_freq INTEGER," \
	"max_freq INTEGER," \
	"orig_freq INTEGER," \
	"phone BLOB," \
	"phrase TEXT," \
	"PRIMARY KEY (phone, phrase)" \
	");"

#define CHEWING_DB_COLUMN "time, user_freq, max_freq, orig_freq, phone, phrase"
/*
 * The SELECT index starts from 0, but the INSERT/REPLACE index starts from 1,
 * so we cannot use the same index for both SELECT & INSERT/REPLACE.
 */
#define CHEWING_DB_SEL_INDEX_TIME	(0)
#define CHEWING_DB_SEL_INDEX_USER_FREQ	(1)
#define CHEWING_DB_SEL_INDEX_MAX_FREQ	(2)
#define CHEWING_DB_SEL_INDEX_ORIG_FREQ	(3)
#define CHEWING_DB_SEL_INDEX_PHONE	(4)
#define CHEWING_DB_SEL_INDEX_PHRASE	(5)

#define CHEWING_DB_INS_INDEX_TIME	(CHEWING_DB_SEL_INDEX_TIME + 1)
#define CHEWING_DB_INS_INDEX_USER_FREQ	(CHEWING_DB_SEL_INDEX_USER_FREQ + 1)
#define CHEWING_DB_INS_INDEX_MAX_FREQ	(CHEWING_DB_SEL_INDEX_MAX_FREQ + 1)
#define CHEWING_DB_INS_INDEX_ORIG_FREQ	(CHEWING_DB_SEL_INDEX_ORIG_FREQ + 1)
#define CHEWING_DB_INS_INDEX_PHONE	(CHEWING_DB_SEL_INDEX_PHONE + 1)
#define CHEWING_DB_INS_INDEX_PHRASE	(CHEWING_DB_SEL_INDEX_PHRASE + 1)

#define CHEWING_DB_SELECT_BY_PHONE "SELECT " CHEWING_DB_COLUMN " FROM " \
	CHEWING_TABLE_USERPHRASE " WHERE phone = ?4"

#define CHEWING_DB_SELECT_BY_PHONE_PHRASE "SELECT " CHEWING_DB_COLUMN " FROM " \
	CHEWING_TABLE_USERPHRASE " WHERE phone = ?4 AND phrase = ?5"

#define CHEWING_DB_UPSERT "INSERT OR REPLACE INTO " CHEWING_TABLE_USERPHRASE \
	"(" CHEWING_DB_COLUMN ") VALUES (?1,?2,?3,?4,?5,?6)"


#define CHEWING_TABLE_CONFIG	"config_v1"
#define CHEWING_DB_CONFIG_CREATE_TABLE "CREATE TABLE IF NOT EXISTS " CHEWING_TABLE_CONFIG " ("\
	"id INTEGER," \
	"value INTEGER," \
	"PRIMARY KEY (id)" \
	");"

#define CHEWING_DB_CONFIG_COLUMN	"value, id"

#define CHEWING_DB_CONFIG_ID_LIFETIME	(0)

#define CHEWING_DB_CONFIG_SEL_VALUE	(0)
#define CHEWING_DB_CONFIG_SEL_ID	(1)

#define CHEWING_DB_CONFIG_INS_VALUE	(1)
#define CHEWING_DB_CONFIG_INS_ID	(2)
#define CHEWING_DB_CONFIG_INS_VALUE_INC	(3)

#define CHEWING_DB_CONFIG_SELECT "SELECT " CHEWING_DB_CONFIG_COLUMN " FROM " CHEWING_TABLE_CONFIG \
	" WHERE id = ?1"

#define CHEWING_DB_CONFIG_INSERT "INSERT OR IGNORE INTO " CHEWING_TABLE_CONFIG \
	" (" CHEWING_DB_CONFIG_COLUMN ") VALUES (?1, ?2)"

#define CHEWING_DB_CONFIG_INCREASE "UPDATE " CHEWING_TABLE_CONFIG \
	" SET value = value + ?3 WHERE id = ?2"

int InitHash( struct tag_ChewingData *ctx );
void TerminateHash( struct tag_ChewingData *pgdata );

#endif
