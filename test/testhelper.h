/**
 * testhelper.h
 * * Copyright (c) 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#ifdef HAVE_CONFIG_H
#include <config.h>
#endif

#include <stddef.h>

#include "chewing.h"
#include "chewingio.h"

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

#ifdef __GNUC__
#define ARRAY_SIZE( array ) ( sizeof(array) / sizeof(((typeof(array)){})[0]) )
#else
#define ARRAY_SIZE( array ) ( sizeof(array) / sizeof(array[0] ) )
#endif

#define ok(test, fmt, ...) \
	internal_ok(__FILE__, __LINE__, !!(test), #test, fmt, ##__VA_ARGS__)
#define ok_commit_buffer(ctx, expected) \
	internal_ok_buffer(__FILE__, __LINE__, ctx, expected, &COMMIT_BUFFER)
#define ok_preedit_buffer(ctx, expected) \
	internal_ok_buffer(__FILE__, __LINE__, ctx, expected, &PREEDIT_BUFFER)
#define ok_zuin_buffer(ctx, expected) \
	internal_ok_buffer(__FILE__, __LINE__, ctx, expected, &ZUIN_BUFFER)
#define ok_aux_buffer(ctx, expected) \
	internal_ok_buffer(__FILE__, __LINE__, ctx, expected, &AUX_BUFFER)
#define ok_candidate(ctx, cand, cand_len) \
	internal_ok_candidate(__FILE__, __LINE__, ctx, cand, cand_len)
#define ok_keystoke_rtn(ctx, rtn) \
	internal_ok_keystoke_rtn(__FILE__, __LINE__, ctx, rtn)
#define has_userphrase(ctx, bopomofo, phrase) \
	internal_has_userphrase(__FILE__, __LINE__, ctx, bopomofo, phrase)

typedef struct {
	char * token;
	char * expected;
} TestData;

typedef struct {
	int (*check)(ChewingContext *ctx);
	int (*check_alt)(ChewingContext *ctx);
	int (*get_length)(ChewingContext *ctx);
	char * (*get_string)(ChewingContext *ctx);
	char * (*get_string_alt)(ChewingContext *ctx, int *len);
} BufferType;

extern BufferType COMMIT_BUFFER;
extern BufferType PREEDIT_BUFFER;
extern BufferType ZUIN_BUFFER;
extern BufferType AUX_BUFFER;

typedef int (*get_char_func) ( void *param );

int get_keystroke( get_char_func get_char, void *param );
void type_keystoke_by_string( ChewingContext *ctx, char* keystoke );
int exit_status();

// The internal_xxx function shall be used indirectly by macro in order to
// get correct __FILE__ and __LINE__ information.
void internal_ok_buffer( const char *file, int line, ChewingContext *ctx,
	const char *expected, const BufferType *buffer );
void internal_ok( const char *file, int line, int test, const char * test_txt,
	const char *message, ...);
void internal_ok_candidate( const char *file, int line,
	ChewingContext *ctx, const char *cand[], size_t cand_len );
void internal_ok_keystoke_rtn( const char *file, int line,
	ChewingContext *ctx, int rtn );
int internal_has_userphrase( const char *file, int line,
	ChewingContext *ctx, const char *bopomofo, const char *phrase );
