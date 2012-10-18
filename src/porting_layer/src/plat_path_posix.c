#ifdef HAVE_CONFIG_H
        #include <config.h>
#endif

#ifdef UNDER_POSIX

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "plat_types.h"

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

	for ( path = strtok_r( buffer, ":", &saveptr ); path; path = strtok_r( NULL, ":", &saveptr )) {
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
	char *tmp;

	tmp = getenv( "CHEWING_PATH" );
	if ( tmp ) {
		strncpy( path, tmp, path_len );
	} else {
		strncpy( path, "$HOME/.chewing:" LIBDIR "/chewing", path_len );
	}

	return;
}

#endif /* UNDER_POSIX */
