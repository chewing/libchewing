/**
 * test-logger.c
 *
 * Copyright (c) 2013
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#include <config.h>
#endif

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>

#include "chewing.h"
#include "testhelper.h"

static const char *LOG_PATH = TEST_HASH_DIR "/logger.log";

void logger( void *data UNUSED, int level UNUSED, const char *fmt, ... )
{
	va_list ap;
	FILE *fd = (FILE *) data;

	va_start( ap, fmt );
	vfprintf( fd, fmt, ap );
	va_end( ap );
}

void test_set_logger()
{
	ChewingContext *ctx;
	FILE *fd;

	ctx = chewing_new();
	fd = fopen( LOG_PATH,  "w" );

	chewing_set_logger( ctx, logger, fd );
	type_keystroke_by_string( ctx, "hk4g4" );

	chewing_set_logger( ctx, NULL, 0 );
	type_keystroke_by_string( ctx, "hk4g4" );

	fclose( fd );
	chewing_delete( ctx );
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_set_logger();

	return exit_status();
}
