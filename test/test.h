/**
 * test.h
 * * Copyright (c) 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#ifdef HAVE_CONFIG_H
#include <config.h>
#endif

#include "chewing.h"

#define KEY_SLEFT 896
#define KEY_SRIGHT 897
#define KEY_LEFT 898
#define KEY_RIGHT 899
#define KEY_UP 990
#define KEY_DOWN 991
#define KEY_SPACE ' '
#define KEY_ENTER 992
#define KEY_BACKSPACE 993
#define KEY_ESC 994
#define KEY_DELETE 995
#define KEY_HOME 996
#define KEY_END 997
#define KEY_TAB 998
#define KEY_CAPSLOCK 999
#define KEY_CTRL_BASE 1000
#define END 2000

#define ARRAY_SIZE(array) ( sizeof(array) / sizeof(array[0] ) )

#define ok(test, message) \
	internal_ok(!!(test), #test, message, __FILE__, __LINE__)
#define verify_keystoke(ctx, key, expected) \
	internal_verify_keystoke( ctx, key, expected, __FILE__, __LINE__)

typedef struct {
	char * token;
	char * expected;
} TestData;

typedef int (*get_char_func) ( void *param );

int get_keystroke( get_char_func get_char, void *param );
int exit_status();

// The internal_xxx function shall be used indirectly by macro in order to
// get correct __FILE__ and __LINE__ information.
void internal_verify_keystoke( ChewingContext *ctx, char *key, char *expected,
	const char *file, int line );
void internal_ok( int test, const char * test_txt, const char *message,
	const char *file, int line );
