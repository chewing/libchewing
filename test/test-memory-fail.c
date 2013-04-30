/**
 * test-memory-fail.c
 *
 * Copyright (c) 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 *
 * This file is used to test malloc() fail secnarios. The fail_countdown
 * control when malloc() fails. For example, fail_countdown = 1 means the
 * second malloc() will fail.
 */
#ifdef HAVE_CONFIG_H
#include <config.h>
#endif

#include <dlfcn.h>
#include <stdlib.h>

#include "chewing.h"

int fail_countdown;

void *__real_malloc( size_t size );
void *__real_calloc( size_t nmemb, size_t size );

void *__wrap_malloc( size_t size )
{
	if ( fail_countdown ) {
		--fail_countdown;
		return __real_malloc( size );
	}
	return NULL;
}

void *__wrap_calloc( size_t nmemb, size_t size )
{
	if ( fail_countdown ) {
		--fail_countdown;
		return __real_calloc( nmemb, size );
	}
	return NULL;
}

void test_chewing_new()
{
	ChewingContext *ctx = NULL;
	int i;

	chewing_Init( NULL, NULL );

	for ( i = 0; ctx == NULL; ++i ) {
		fail_countdown = i;
		ctx = chewing_new();
	}
	chewing_delete( ctx );


	chewing_Terminate();
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_chewing_new();

	return 0;
}
