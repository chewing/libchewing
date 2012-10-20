/**
 * path.c
 *
 * Copyright (c) 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
  #include <config.h>
#endif
#include "path-private.h"

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "plat_types.h"

#ifdef UNDER_POSIX
int get_search_path( char * path, size_t path_len )
{
	char *chewing_path;
	char *home;

	chewing_path = getenv( "CHEWING_PATH" );
	if ( chewing_path ) {
		strncpy( path, chewing_path, path_len );
	} else {
		home = getenv( "HOME" );
		if ( home ) {
			snprintf( path, path_len, "%s/.chewing" SEARCH_PATH_SEP
				LIBDIR "/chewing", home );
		} else {
			// No HOME ?
			strncpy( path, SEARCH_PATH_SEP LIBDIR "/chewing", path_len );
		}
	}

	return 0;
}

#elif defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE)
static char * strtok_r (char *s, const char *delim, char **save_ptr)
{
	char *token;

	if ( s == NULL )
		s = *save_ptr;

	/* Scan leading delimiters.  */
	s += strspn( s, delim );
	if ( *s == '\0' ) {
		*save_ptr = s;
		return NULL;
	}

	/* Find the end of the token.  */
	token = s;
	s = strpbrk( token, delim );
	if ( s == NULL )
		/* This token finishes the string.  */
		*save_ptr = token + strlen( token );
	else {
		/* Terminate the token and make *SAVE_PTR point past it.  */
		*s = '\0';
		*save_ptr = s + 1;
	}
	return token;
}

int get_search_path( char * path, size_t path_len )
{
	char *chewing_path;
	char *appdata;

	chewing_path = getenv( "CHEWING_PATH" );
	if ( chewing_path ) {
		strncpy( path, chewing_path, path_len );
	} else {
		appdata = getenv( "APPDATA" );
		if ( appdata ) {
			snprintf( path, path_len, "%s", appdata );
		} else {
			snprintf( path, path_len, "" );
			return -1;
		}
	}

	return 0;
}
#else
#error please implement get_search_path
#endif

static int are_all_files_readable(
	const char *path,
	const char * const *files,
	char *output,
	size_t output_len )
{
	int i;

	assert( path );
	assert( files );

	for ( i = 0; files[i] != NULL; ++i ) {
		snprintf( output, output_len, "%s" PLAT_SEPARATOR "%s", path,
				files[i] );
		if ( access( output, R_OK ) != 0 ) {
			return 0;
		}
	}

	return 1;
}

int find_path_by_files(
	const char *search_path,
	const char * const *files,
	char *output,
	size_t output_len )
{
	char buffer[PATH_MAX];
	char *path;
	char *saveptr;
	int ret;

	assert( search_path );
	assert( files );
	assert( output );
	assert( output_len );

	// strtok_r will modify its first parameter.
	strncpy( buffer, search_path, sizeof( buffer ) );

	for ( path = strtok_r( buffer, SEARCH_PATH_SEP, &saveptr );
		path; path = strtok_r( NULL, SEARCH_PATH_SEP, &saveptr ) ) {

		ret = are_all_files_readable( path, files, output, output_len );
		if ( ret ) {
			snprintf( output, output_len, "%s", path );
			return 0;
		}
	}
	return -1;
}
