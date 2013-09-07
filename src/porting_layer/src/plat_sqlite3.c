/**
 * plat_sqlite3.c
 *
 * Copyright (c) 2013
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#ifdef HAVE_CONFIG_H
#  include <config.h>
#endif

#include <malloc.h>
#include <stdlib.h>

#include "plat_sqlite3.h"
#include "plat_types.h"

#define CHEWING_MAX_DB_PATH	(1024)

#if defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE)

#include <Shlobj.h>
#define CHEWING_DB_PATH		L"chewing"
#define CHEWING_DB_NAME		L"chewing.db"

static int SetSQLiteTemp( char *buf, size_t len, wchar_t *wbuf, size_t wlen )
{
	/*
	 * Set temporary directory is necessary for Windows platform.
	 * http://www.sqlite.org/capi3ref.html#sqlite3_temp_directory
	 */

	int ret;

	ret = GetTempPathW( wlen, wbuf );
	if ( ret == 0 || ret >= wlen ) return -1;

	ret = WideCharToMultiByte( CP_UTF8, 0, wbuf, -1, buf, len, NULL, NULL );
	if ( ret == 0 ) return -1;

	// FIXME: When to free sqlite3_temp_directory?
	// FIXME: thread safe?
	sqlite3_temp_directory = sqlite3_mprintf( "%s", buf );
	if ( sqlite3_temp_directory == 0 ) exit( -1 );

	return 0;
}

static int GetSQLitePath( wchar_t *wbuf, size_t wlen )
{
	int ret;

	ret = GetEnvironmentVariableW( L"CHEWING_USER_PATH", wbuf, wlen );
	if ( ret ) {
		wcscat_s( wbuf, wlen, L"\\" CHEWING_DB_NAME );
		return 0;
	}

	// FIXME: Use SHGetKnownFolderPath instead?
	ret = GetEnvironmentVariableW( L"APPDATA", wbuf, wlen );
	if ( ret ) {
		wcscat_s( wbuf, wlen, L"\\" CHEWING_DB_PATH );

		ret = CreateDirectoryW( wbuf, 0 );
		if ( ret != 0 || GetLastError() == ERROR_ALREADY_EXISTS ) {
			wcscat_s( wbuf, wlen, L"\\" CHEWING_DB_NAME );
			return 0;
		}

	}
	return -1;
}

sqlite3 *GetSQLiteInstance()
{
	wchar_t *wbuf = NULL;
	char *buf = NULL;
	int ret;
	sqlite3 *db = NULL;

	wbuf = (wchar_t *) calloc( CHEWING_MAX_DB_PATH, sizeof( *wbuf ) );
	if ( !wbuf ) exit( -1 );

	buf = (char *) calloc( CHEWING_MAX_DB_PATH, sizeof( *buf ) );
	if ( !buf ) exit( -1 );

	ret = SetSQLiteTemp( buf, CHEWING_MAX_DB_PATH, wbuf, CHEWING_MAX_DB_PATH );
	if ( ret ) goto end;

	ret = GetSQLitePath( wbuf, CHEWING_MAX_DB_PATH );
	if ( ret ) goto end;

	ret = sqlite3_open16( wbuf, &db );
	if ( ret != SQLITE_OK ) goto end;

end:
	free( buf );
	free( wbuf );
	return db;
}

#else

#include <string.h>
#include <unistd.h>

#define CHEWING_DB_PATH		"chewing"
#define CHEWING_DB_NAME		"chewing.db"

static int GetSQLitePath( char *buf, size_t len )
{
	char *path;

	path = getenv( "CHEWING_USER_PATH" );
	if ( path && access( path, W_OK ) == 0 ) {
		snprintf( buf, len, "%s" PLAT_SEPARATOR "%s", path, CHEWING_DB_NAME );
		return 0;
	}

	path = getenv( "HOME" );
	if ( !path ) {
		path = PLAT_TMPDIR;
	}

	snprintf( buf, len, "%s" PLAT_SEPARATOR "%s", path, CHEWING_DB_PATH );
	PLAT_MKDIR( buf );
	strncat( buf, PLAT_SEPARATOR CHEWING_DB_NAME, len - strlen( buf ) );
	return 0;
}

sqlite3 * GetSQLiteInstance()
{
	char *buf = NULL;
	int ret;
	sqlite3 *db = NULL;

	buf = (char *) calloc( CHEWING_MAX_DB_PATH, sizeof( *buf ) );
	if ( !buf ) exit( -1 );

	ret = GetSQLitePath( buf, CHEWING_MAX_DB_PATH );
	if ( ret ) goto end;

	ret = sqlite3_open( buf, &db );
	if ( ret != SQLITE_OK ) goto end;

end:
	free( buf );
	return db;
}

#endif
