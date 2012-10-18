#ifdef HAVE_CONFIG_H
        #include <config.h>
#endif

#ifdef UNDER_POSIX

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "plat_types.h"

#define PATH_SEP ":"

static int are_all_files_readable( const char *path, const char * const *files, char *output, size_t output_len )
{
	int i;

	assert( path );
	assert( files );

	for ( i = 0; files[i] != NULL; ++i ) {
		snprintf( output, output_len, "%s" PLAT_SEPARATOR "%s", path, files[i] );
		if ( access( output, R_OK ) != 0 ) {
			return 0;
		}
	}

	return 1;
}

int find_path_by_files( const char *search_path, const char * const *files, char *output, size_t output_len )
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

	for ( path = strtok_r( buffer, PATH_SEP, &saveptr ); path; path = strtok_r( NULL, PATH_SEP, &saveptr )) {
		ret = are_all_files_readable( path, files, output, output_len );
		if ( ret ) {
			snprintf( output, output_len, "%s", path );
			return 0;
		}
	}
	return -1;
}

void get_search_path( char * path, size_t path_len )
{
	char *chewing_path;
	char *home;

	chewing_path = getenv( "CHEWING_PATH" );
	if ( chewing_path ) {
		strncpy( path, chewing_path, path_len );
	} else {
		home = getenv( "HOME" );
		if ( home ) {
			snprintf( path, path_len, "%s/.chewing" PATH_SEP LIBDIR "/chewing", home );
		} else {
			// No HOME ?
			strncpy( path, PATH_SEP LIBDIR "/chewing", path_len );
		}
	}

	return;
}

#endif /* UNDER_POSIX */
