#ifdef HAVE_CONFIG_H
        #include <config.h>
#endif

#ifdef UNDER_POSIX

#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "plat_types.h"
#include "plat_path.h"

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
			snprintf( path, path_len, "%s/.chewing" PATH_SEP
				LIBDIR "/chewing", home );
		} else {
			// No HOME ?
			strncpy( path, PATH_SEP LIBDIR "/chewing", path_len );
		}
	}

	return 0;
}

#endif /* UNDER_POSIX */
