/**
 * hash.c
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004, 2005, 2006, 2007, 2008, 2011, 2012, 2013
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include <assert.h>
#include <string.h>
#include <sys/stat.h>
/* ISO C99 Standard: 7.10/5.2.4.2.1 Sizes of integer types */
#include <limits.h>
#include <stdlib.h>
#include <stdio.h>

#include "plat_sqlite3.h"

#include "chewing-private.h"
#include "chewing-utf8-util.h"
#include "hash-private.h"
#include "private.h"
#include "memory-private.h"

static void UpdateLiftTime( ChewingData *pgdata )
{
	int ret;
	sqlite3_stmt *stmt = NULL;

	ret = sqlite3_prepare_v2( pgdata->static_data.db, CHEWING_DB_CONFIG_INCREASE, -1, &stmt, NULL );
	if ( ret != SQLITE_OK ) goto error;

	ret = sqlite3_bind_int( stmt, CHEWING_DB_CONFIG_INS_ID, CHEWING_DB_CONFIG_ID_LIFETIME );
	if ( ret != SQLITE_OK ) goto error;

	ret = sqlite3_bind_int( stmt, CHEWING_DB_CONFIG_INS_VALUE_INC,
		pgdata->static_data.new_lifttime - pgdata->static_data.original_lifttime );
	if ( ret != SQLITE_OK ) goto error;

	ret = sqlite3_step( stmt );
	if ( ret != SQLITE_DONE ) goto error;

error:
	sqlite3_finalize( stmt );
}

void TerminateHash( ChewingData *pgdata )
{
	int ret;

	UpdateLiftTime( pgdata );

	ret = sqlite3_close( pgdata->static_data.db );
	assert( SQLITE_OK == ret );
}

static int CreateTable( ChewingData *pgdata )
{
	int ret;

	ret = sqlite3_exec( pgdata->static_data.db,
		"CREATE TABLE IF NOT EXISTS " TABLE_USERPHRASE " ("
		"time INTEGER,"
		"user_freq INTEGER,"
		"max_freq INTEGER,"
		"orig_freq INTEGER,"
		"phone BLOB,"
		"phrase TEXT,"
		"PRIMARY KEY (phone, phrase)"
		")",
		NULL, NULL, NULL );
	if ( ret != SQLITE_OK ) return -1;

	ret = sqlite3_exec( pgdata->static_data.db,
		"CREATE TABLE IF NOT EXISTS " TABLE_CONFIG " ("
		"id INTEGER,"
		"value INTEGER,"
		"PRIMARY KEY (id)"
		")",
		NULL, NULL, NULL );
	if ( ret != SQLITE_OK ) return -1;

	return 0;
}

static int SetupUserphraseLiftTime( ChewingData *pgdata )
{
	int ret;
	sqlite3_stmt *stmt = NULL;

	ret = sqlite3_prepare_v2( pgdata->static_data.db, CHEWING_DB_CONFIG_INSERT, -1, &stmt, NULL );
	if ( ret != SQLITE_OK ) goto error;

	ret = sqlite3_bind_int( stmt, CHEWING_DB_CONFIG_INS_ID, CHEWING_DB_CONFIG_ID_LIFETIME );
	if ( ret != SQLITE_OK ) goto error;

	ret = sqlite3_bind_int( stmt, CHEWING_DB_CONFIG_INS_VALUE, 0 );
	if ( ret != SQLITE_OK ) goto error;

	ret = sqlite3_step( stmt );
	if ( ret != SQLITE_DONE ) goto error;

	ret = sqlite3_finalize( stmt );
	if ( ret != SQLITE_OK ) goto error;


	ret = sqlite3_prepare_v2( pgdata->static_data.db, CHEWING_DB_CONFIG_SELECT, -1, &stmt, NULL );
	if ( ret != SQLITE_OK ) goto error;

	ret = sqlite3_bind_int( stmt, CHEWING_DB_CONFIG_SEL_ID, CHEWING_DB_CONFIG_ID_LIFETIME );
	if ( ret != SQLITE_OK ) goto error;

	ret = sqlite3_step( stmt );
	if ( ret != SQLITE_ROW ) goto error;

	pgdata->static_data.original_lifttime = sqlite3_column_int( stmt, CHEWING_DB_CONFIG_SEL_VALUE );
	pgdata->static_data.new_lifttime = pgdata->static_data.original_lifttime;

	ret = sqlite3_finalize( stmt );
	if ( ret != SQLITE_OK ) goto error;

	return 0;

error:
	sqlite3_finalize( stmt );
	return -1;
}

int InitHash( ChewingData *pgdata )
{
	int ret;

	pgdata->static_data.db = GetSQLiteInstance();
	if ( !pgdata->static_data.db ) return -1;

	ret = CreateTable( pgdata );
	if ( ret ) return -1;

	ret = SetupUserphraseLiftTime( pgdata );
	if ( ret ) return -1;

	// FIXME: Normalize lifttime when necessary.
	// FIXME: Migrate old uhash.dat here.

	return 0;
}

