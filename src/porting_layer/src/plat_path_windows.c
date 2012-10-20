#ifdef HAVE_CONFIG_H
	#include <config.h>
#endif

#if defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE)

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// steal the code from gnulib and modify for Windows.
char * strtok_r (char *s, const char *delim, char **save_ptr)
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
#endif /* defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE) */
