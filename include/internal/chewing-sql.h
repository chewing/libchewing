/**
 * chewing-sql.c
 *
 * Copyright (c)
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2013
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#ifndef CHEWING_SQL_H
#define CHEWING_SQL_H

#include "chewing-private.h"

enum {
	BIND_LEN,
	BIND_PHRASE,
	BIND_PHONE_0,
	BIND_PHONE_1,
	BIND_PHONE_2,
	BIND_PHONE_3,
	BIND_PHONE_4,
	BIND_PHONE_5,
	BIND_PHONE_6,
	BIND_PHONE_7,
	BIND_PHONE_8,
	BIND_PHONE_9,
	BIND_PHONE_10,
	BIND_NUM_OF_BIND,
};

enum {
	COLUMN_TIME,
	COLUMN_ORIG_FREQ,
	COLUMN_MAX_FREQ,
	COLUMN_USER_FREQ,
	COLUMN_LEN,
	COLUMN_PHRASE,
	COLUMN_PHONE_0,
	COLUMN_PHONE_1,
	COLUMN_PHONE_2,
	COLUMN_PHONE_3,
	COLUMN_PHONE_4,
	COLUMN_PHONE_5,
	COLUMN_PHONE_6,
	COLUMN_PHONE_7,
	COLUMN_PHONE_8,
	COLUMN_PHONE_9,
	COLUMN_PHONE_10,
	COLUMN_NUM_OF_BIND,
};

enum {
	STMT_SELECT_USERPHRASE,
	STMT_SELECT_USERPHRASE_BY_PHONE,
	STMT_SELECT_USERPHRASE_BY_PHONE_PHRASE,
	STMT_UPSERT_USERPHRASE,
	STMT_DELETE_USERPHRASE,
	STMT_NUM_OF_STMT,
};

typedef struct SqlStmtDesc_ {
	const char *stmt;
	const int bind[BIND_NUM_OF_BIND];
	const int column[COLUMN_NUM_OF_BIND];
} SqlStmtDesc;

extern const SqlStmtDesc SQL_STMT_DESC[STMT_NUM_OF_STMT];

int InitSql(ChewingData *pgdata);
void TerminateHash(ChewingData *pgdata);

#endif
