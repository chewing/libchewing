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

void TerminateHash( ChewingData *pgdata )
{
	int ret;
	ret = sqlite3_close( pgdata->static_data.db );
	assert( SQLITE_OK == ret );
}

int InitHash( ChewingData *pgdata )
{
	int ret;
	sqlite3_stmt *stmt = NULL;

	// FIXME: Normalize lifttime when necessary.

	pgdata->static_data.db = GetSQLiteInstance();
	if ( !pgdata->static_data.db ) goto error;

	ret = sqlite3_prepare_v2( pgdata->static_data.db, CHEWING_CREATE_TABLE_USERPHRASE, -1, &stmt, NULL );
	if ( ret != SQLITE_OK ) goto error;

	ret = sqlite3_step( stmt );
	if ( ret != SQLITE_DONE ) goto error;

	ret = sqlite3_finalize( stmt );
	if ( ret != SQLITE_OK ) goto error;

	ret = sqlite3_prepare_v2( pgdata->static_data.db, CHEWING_DB_CONFIG_CREATE_TABLE, -1, &stmt, NULL );
	if ( ret != SQLITE_OK ) goto error;

	ret = sqlite3_step( stmt );
	if ( ret != SQLITE_DONE ) goto error;

	ret = sqlite3_finalize( stmt );
	if ( ret != SQLITE_OK ) goto error;


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

	// FIXME: Migrate old uhash.dat here.

	return 1;

error:
	sqlite3_finalize( stmt );
	// FIXME: Use -1 as error
	return 0;
}

