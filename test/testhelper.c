/**
 * testhelper.c
 *
 * Copyright (c) 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#include "testhelper.h"

#include <assert.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "chewing-private.h"
#include "chewing-utf8-util.h"
#include "hash-private.h"
#include "key2pho-private.h"

static unsigned int test_run;
static unsigned int test_ok;

/* We cannot use designated initializer here due to Visual Studio */
BufferType COMMIT_BUFFER = {
	chewing_commit_Check,
	0,
	0,
	chewing_commit_String,
	0,
};

BufferType PREEDIT_BUFFER = {
	chewing_buffer_Check,
	0,
	chewing_buffer_Len,
	chewing_buffer_String,
	0,
};

BufferType ZUIN_BUFFER = {
	0,
	chewing_zuin_Check,
	0,
	0,
	chewing_zuin_String,
};

BufferType AUX_BUFFER = {
	chewing_aux_Check,
	0,
	chewing_aux_Length,
	chewing_aux_String,
	0,
};

int get_keystroke( get_char_func get_char, void * param )
{
	int ch;
	int result = END;
	int flag = 0;

	assert( get_char );

	while ( ( ch = get_char( param ) ) != END ) {
		if ( ( ch != '<' ) && ( flag != 1 ) )
			return (int) ch;
		else if ( ch == '>' ) {
			flag = 0;
			return result;
		}
		else {
			flag = 1;
			ch = get_char( param );
			switch ( ch ) {
				case '<':
				case '>':
					if ( get_char( param ) == '>' )
						return result = ch;
					break;
				case 'L':
					result = KEY_LEFT;
					break;
				case 'R':
					result = KEY_RIGHT;
					break;
				case 'U':
					result = KEY_UP;
					break;
				case 'D':
					if ( ( ch = get_char( param ) ) == '>' )
						return result = KEY_DOWN;
					else {
						get_char( param );
						return result = KEY_DELETE;
					}
					break;
				case 'E':
					if ( ( ch = get_char( param ) ) == '>' )
						return result = KEY_ENTER;
					else if ( ch == 'E' )
						result = KEY_ESC;
					else
						result = KEY_END;
					break;
				case 'C':
					if ( ( ch = get_char( param ) ) != '>' ) {
						if ( ch == 'B' )
							result = ( KEY_CAPSLOCK );
						else
							result = ( KEY_CTRL_BASE + ch );
					}
					break;
				case 'B':
					result = KEY_BACKSPACE;
					break;
				case 'H':
					result = KEY_HOME;
					break;
				case 'S':
					if ( ( ch = get_char( param ) ) == 'L' )
						result = KEY_SLEFT;
					else if ( ch == 'R' )
						result = KEY_SRIGHT;
					else
						result = KEY_SSPACE;
					break;
				case 'T':
					if ( ( ch = get_char( param ) ) == '>' )
						return result = KEY_TAB;
					else
						result = KEY_DBLTAB;
					break;
				case 'P':
					if ( ( ch = get_char( param ) ) == 'D' )
						result = KEY_NPAGE;
					else
						result = KEY_PPAGE;
					break;
				case 'N':
					ch = get_char( param );
					result = KEY_NUMPAD_BASE + ch;
					break;
			}
		}
	}
	return result = END;
}

void type_single_keystroke( ChewingContext *ctx, int ch )
{
	switch ( ch ) {
		case KEY_LEFT:
			chewing_handle_Left( ctx );
			break;
		case KEY_SLEFT:
			chewing_handle_ShiftLeft( ctx );
			break;
		case KEY_RIGHT:
			chewing_handle_Right( ctx );
			break;
		case KEY_SRIGHT:
			chewing_handle_ShiftRight( ctx );
			break;
		case KEY_UP:
			chewing_handle_Up( ctx );
			break;
		case KEY_DOWN:
			chewing_handle_Down( ctx );
			break;
		case KEY_SPACE:
			chewing_handle_Space( ctx );
			break;
		case KEY_ENTER:
			chewing_handle_Enter( ctx );
			break;
		case KEY_BACKSPACE:
			chewing_handle_Backspace( ctx );
			break;
		case KEY_ESC:
			chewing_handle_Esc( ctx );
			break;
		case KEY_DELETE:
			chewing_handle_Del( ctx );
			break;
		case KEY_HOME:
			chewing_handle_Home( ctx );
			break;
		case KEY_END:
			chewing_handle_End( ctx );
			break;
		case KEY_TAB:
			chewing_handle_Tab( ctx );
			break;
		case KEY_CAPSLOCK:
			chewing_handle_Capslock( ctx );
			break;
		case KEY_NPAGE:
			chewing_handle_PageDown( ctx );
			break;
		case KEY_PPAGE:
			chewing_handle_PageUp( ctx );
			break;
		case KEY_SSPACE:
			chewing_handle_ShiftSpace( ctx );
			break;
		case KEY_DBLTAB:
			chewing_handle_DblTab( ctx );
			break;
		default:
			if ( KEY_CTRL_BASE <= ch && ch < KEY_NUMPAD_BASE)
				chewing_handle_CtrlNum( ctx, ch - KEY_CTRL_BASE );
			else if ( KEY_NUMPAD_BASE <= ch )
				chewing_handle_Numlock( ctx, ch - KEY_NUMPAD_BASE );
			else
				chewing_handle_Default( ctx, (char) ch );
			break;
	}
}

static void type_keystroke( ChewingContext *ctx, get_char_func get_char, void *param )
{
	int ch;

	while ( ( ch = get_keystroke( get_char, param ) ) != END )
		type_single_keystroke( ctx, ch );
}

static int get_char_by_string( void * param )
{
	char **ptr = param;
	char ch;

	assert( param );

	if ( **ptr == 0 ) {
		return END;
	}

	ch = **ptr;
	++*ptr;
	return ch;
}

void internal_ok( const char *file, int line, int test, const char * test_txt,
	const char *fmt, ...)
{
	va_list ap;

	++test_run;
	if ( test ) {
		++test_ok;
		printf( "ok %d ", test_run);

		va_start( ap, fmt );
		vprintf( fmt, ap );
		va_end( ap );

		printf("\n");
	} else {
		printf( "not ok %d ", test_run);

		va_start( ap, fmt );
		vprintf( fmt, ap );
		va_end( ap );

		printf( "\n# %s failed in %s:%d\n", test_txt, file, line );
	}
}

void type_keystroke_by_string( ChewingContext *ctx, char* keystroke )
{
	type_keystroke( ctx, get_char_by_string, &keystroke );
}

void internal_ok_buffer( const char *file, int line, ChewingContext *ctx,
	const char *expected, const BufferType *buffer )
{
	char *buf;
	int actual_ret;
	int expected_ret;
	int expected_len;

	assert( ctx );
	assert( expected );
	assert( buffer );

	expected_len = ueStrLen( expected );

	if ( buffer->check ) {
		actual_ret = buffer->check( ctx );
		expected_ret = !!expected_len;
		internal_ok( file, line, actual_ret == expected_ret,
			"actual_ret == expected_ret",
			"check function returned `%d' shall be `%d'", actual_ret, expected_ret );
	}

	if ( buffer->check_alt ) {
		actual_ret = buffer->check_alt( ctx );
		expected_ret = !expected_len;
		internal_ok( file, line, actual_ret == expected_ret,
			"actual_ret == expected_ret",
			"check function returned `%d' shall be `%d'", actual_ret, expected_ret );
	}

	if ( buffer->get_length ) {
		actual_ret = buffer->get_length( ctx );
		expected_ret = expected_len;
		internal_ok( file, line, actual_ret == expected_ret,
			"actual_ret == expected_ret",
			"get length function returned `%d' shall be `%d'", actual_ret, expected_ret );
	}

	if ( buffer->get_string ) {
		buf = buffer->get_string( ctx );
		internal_ok( file, line, !strcmp( buf, expected ), "!strcmp( buf, expected )",
			"string function returned `%s' shall be `%s'", buf, expected );
		chewing_free( buf );
	}

	if ( buffer->get_string_alt ) {
		buf = buffer->get_string_alt( ctx, &actual_ret );
		expected_ret = expected_len;
		internal_ok( file, line, actual_ret == expected_ret,
			"actual_ret == expected_ret",
			"string function returned parameter `%d' shall be `%d'", actual_ret, expected_ret );
		internal_ok( file, line, !strcmp( buf, expected ), "!strcmp( buf, expected )",
			"string function returned `%s' shall be `%s'", buf, expected );
		chewing_free( buf );
	}
}

void internal_ok_candidate( const char *file, int line,
	ChewingContext *ctx, const char *cand[], size_t cand_len )
{
	size_t i;
	char *buf;

	assert( ctx );

	chewing_cand_Enumerate( ctx );
	for ( i = 0; i < cand_len; ++i ) {
		internal_ok( file, line, chewing_cand_hasNext( ctx ), __func__,
			"shall has next candidate" );
		buf = chewing_cand_String( ctx );
		internal_ok( file, line, strcmp( buf, cand[i] ) == 0, __func__,
			"candndate `%s' shall be `%s'", buf, cand[i] );
		chewing_free( buf );
	}

	internal_ok( file, line , !chewing_cand_hasNext( ctx ), __func__,
			"shall not have next candidate" );
	buf = chewing_cand_String( ctx );

	internal_ok( file, line, strcmp( buf, "" ) == 0, __func__,
		"candndate `%s' shall be `%s'", buf, "" );

	chewing_free( buf );
}

void internal_ok_keystroke_rtn( const char *file, int line,
	ChewingContext *ctx, int rtn )
{
	const struct {
		int rtn;
		int (*func)(ChewingContext* ctx);
	} TABLE[] = {
		{ KEYSTROKE_IGNORE, chewing_keystroke_CheckIgnore },
		{ KEYSTROKE_COMMIT, chewing_commit_Check },
		// No function to check KEYSTROKE_BELL
		{ KEYSTROKE_ABSORB, chewing_keystroke_CheckAbsorb },
	};
	size_t i;
	int actual;
	int expected;

	assert( ctx );

	for ( i = 0; i < ARRAY_SIZE( TABLE ); ++i ) {
		actual = TABLE[i].func( ctx );
		expected = !!( rtn & TABLE[i].rtn );

		internal_ok( file, line, actual == expected,
			__func__, "keystroke rtn `%d' shall be `%d'", actual, expected );
	}
}

int internal_has_userphrase( const char *file UNUSED, int line UNUSED,
	ChewingContext *ctx, const char *bopomofo, const char *phrase )
{
	uint16_t *phone = NULL;
	char *bopomofo_buf = NULL;
	int i;
	char *p;
	char *save_ptr = NULL;
	HASH_ITEM *item = NULL;
	int ret = 0;

	phone = calloc( MAX_PHONE_SEQ_LEN, sizeof (*phone) );
	if ( !phone ) {
		fprintf( stderr, "calloc fails at %s:%d", __FILE__, __LINE__ );
		goto end;
	}

	bopomofo_buf = strdup( bopomofo );
	if ( !bopomofo_buf ) {
		fprintf( stderr, "strdup fails at %s:%d", __FILE__, __LINE__ );
		goto end;
	}

	for ( i = 0, p = strtok_r( bopomofo_buf, " ", &save_ptr );
		i < MAX_PHONE_SEQ_LEN && p;
		++i, p = strtok_r( NULL, " ", &save_ptr) ) {
		phone[i] = UintFromPhone( p );
	}

	while ( ( item = HashFindPhonePhrase( ctx->data, phone, item ) ) != NULL ) {
		if ( phrase == NULL || strcmp( item->data.wordSeq, phrase ) == 0 ) {
			ret = 1;
			goto end;
		}
	}

end:
	free( bopomofo_buf );
	free( phone );

	return ret;
}

int exit_status()
{
	return test_run == test_ok ? 0 : -1;
}
