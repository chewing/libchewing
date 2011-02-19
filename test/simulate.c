/**
 * simulate.c
 *
 * Copyright (c) 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#define USED_IN_SIMULATION
#include "testchewing.c"
#include <unistd.h>

#define FN_MATERIALS "materials.txt"

static FILE *fp = NULL;

int init_sim()
{
	if ( 0 == access( FN_MATERIALS "-random", R_OK ))
		fp = fopen( FN_MATERIALS "-random", "r" );
	else
		fp = fopen( FN_MATERIALS, "r" );
	return (fp != NULL);
}

int fini_sim()
{
	if ( fp )
		fclose( fp );
	fflush( stdout );
	return 0;
}

static char linebuf[ MAXLEN ];
int fake_getchar()
{
	static int remainder = 0;
	static int idx = 0;
	char *pos;

	if ( feof( fp ) )
		return EOF;

	if ( remainder == 0 ) {
start:
		if ( fgets( linebuf, MAXLEN, fp ) == NULL )
			return EOF;
		if ( linebuf[ 0 ] == '#' || linebuf[ 0 ] == ' ' )
			goto start;

		pos = strstr( linebuf, "<E>" );
		if ( ! pos )
			return EOF;
		*(pos + 3) = '\0';
		remainder = pos - linebuf + 3;
		idx = 0;

		pos += 4;
		while ( *pos == '\t' || *pos == ' ' )
			pos++;
		strcpy( expect_string_buf, pos );
	}
	remainder--;
	return linebuf[ idx++ ];
}

int main()
{
	if ( ! init_sim() )
		return 1;

	chewing_test_Main();

	{
		printf(
"_________________________________________________________________________\n"
		        "[ Report ]\n");
		printf( "Checks: %d words,  Failures: %d words\n",
		        tested_word_count, failed_word_count );
		printf( "Ratio: %.2f%%\n",
		        (float) (tested_word_count - failed_word_count ) /
			        tested_word_count * 100 );
	}

	fini_sim();
	return 0;
}
