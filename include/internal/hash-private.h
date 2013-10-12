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

/*
 * userphrase_v1
 */

#define DB_USERPHRASE_COLUMN_NO_PHONE_PHRASE \
	"time,user_freq,max_freq,orig_freq,length"

#define DB_USERPHRASE_COLUMN_NO_PHONE \
	DB_USERPHRASE_COLUMN_NO_PHONE_PHRASE ",phrase" \

#define DB_USERPHRASE_COLUMN \
	DB_USERPHRASE_COLUMN_NO_PHONE ","\
	"phone_0,phone_1,phone_2,phone_3,phone_4,phone_5," \
	"phone_6,phone_7,phone_8,phone_9,phone_10"

#define DB_INDEX_TIME		(90)
#define DB_INDEX_USER_FREQ	(91)
#define DB_INDEX_MAX_FREQ	(92)
#define DB_INDEX_ORIG_FREQ	(93)
#define DB_INDEX_LENGTH		(94)
#define DB_INDEX_PHRASE		(95)
#define DB_INDEX_PHONE_0	(100)

#define DB_SELECT_INDEX_TIME		(0)
#define DB_SELECT_INDEX_USER_FREQ	(1)
#define DB_SELECT_INDEX_MAX_FREQ	(2)
#define DB_SELECT_INDEX_ORIG_FREQ	(3)
#define DB_SELECT_INDEX_LENGTH		(4)
#define DB_SELECT_INDEX_PHRASE		(5)
#define DB_SELECT_INDEX_PHONE_0		(6)

#define DB_PHONE_STMT \
	"length = ?94 AND " \
	"phone_0 = ?100 AND " \
	"phone_1 = ?101 AND " \
	"phone_2 = ?102 AND " \
	"phone_3 = ?103 AND " \
	"phone_4 = ?104 AND " \
	"phone_5 = ?105 AND " \
	"phone_6 = ?106 AND " \
	"phone_7 = ?107 AND " \
	"phone_8 = ?108 AND " \
	"phone_9 = ?109 AND " \
	"phone_10 = ?110"

#define DB_SELECT_USERPHRASE_BY_PHONE \
	"SELECT " DB_USERPHRASE_COLUMN_NO_PHONE " FROM userphrase_v1 WHERE " \
	DB_PHONE_STMT

#define DB_SELECT_USERPHRASE_BY_PHONE_PHRASE \
	"SELECT " DB_USERPHRASE_COLUMN_NO_PHONE_PHRASE " FROM userphrase_v1 WHERE " \
	DB_PHONE_STMT " AND phrase = ?95"

#define DB_UPSERT_USERPHRASE \
	"INSERT OR REPLACE INTO userphrase_v1 (" DB_USERPHRASE_COLUMN ") " \
	"VALUES (?90,?91,?92,?93,?94,?95,?100,?101,?102,?103,?104,?105,?106,?107,?108,?109,?110)"

#define DB_DELETE_USERPHRASE \
	"DELETE FROM userphrase_v1 WHERE " DB_PHONE_STMT " AND phrase = ?95"

/*
 * config_v1
 */

#define CHEWING_DB_CONFIG_COLUMN	"value, id"

#define CHEWING_DB_CONFIG_ID_LIFETIME	(0)

#define CHEWING_DB_CONFIG_SEL_VALUE	(0)
#define CHEWING_DB_CONFIG_SEL_ID	(1)

#define CHEWING_DB_CONFIG_INS_VALUE	(1)
#define CHEWING_DB_CONFIG_INS_ID	(2)
#define CHEWING_DB_CONFIG_INS_VALUE_INC	(3)

#define CHEWING_DB_CONFIG_SELECT "SELECT " CHEWING_DB_CONFIG_COLUMN " FROM " \
	"config_v1  WHERE id = ?1"

#define CHEWING_DB_CONFIG_INSERT "INSERT OR IGNORE INTO config_v1 " \
	" (" CHEWING_DB_CONFIG_COLUMN ") VALUES (?1, ?2)"

#define CHEWING_DB_CONFIG_INCREASE "UPDATE config_v1 " \
	" SET value = value + ?3 WHERE id = ?2"

int InitHash( struct tag_ChewingData *ctx );
void TerminateHash( struct tag_ChewingData *pgdata );

#endif
