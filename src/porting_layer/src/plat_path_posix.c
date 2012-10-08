#ifdef HAVE_CONFIG_H
        #include <config.h>
#endif

#ifdef UNDER_POSIX

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <wordexp.h>
#include <string.h>

#include "plat_types.h"

static int are_all_files_readable( const char *path, const char **files, char *output, size_t output_len )
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

int find_path_by_files( const char *search_path, const char **files, char *output, size_t output_len )
{
	char buffer[PATH_MAX];
	char *path;
	char *saveptr;
	int ret;
	wordexp_t word;

	assert( search_path );
	assert( files );
	assert( output );
	assert( output_len );

	// strtok_r will modify its first parameter.
	strncpy( buffer, search_path, sizeof( buffer ) );

	for ( path = strtok_r( buffer, ":", &saveptr ); path; path = strtok_r( NULL, ":", &saveptr )) {
		// expand shell variable like $HOME
		ret = wordexp( path, &word, 0 );
		if ( ret == 0 && word.we_wordc == 1 ) {
			ret = are_all_files_readable( word.we_wordv[0], files, output, output_len );
			if ( ret ) {
				snprintf( output, output_len, "%s", word.we_wordv[0] );
				wordfree( &word );
				return 0;
			}
		}
		wordfree( &word );
	}
	return -1;
}

#endif /* UNDER_POSIX */
