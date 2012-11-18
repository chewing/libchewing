/**
 * test.c
 *
 * Copyright (c) 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#include "test.h"

#include <assert.h>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

#include "chewing-utf8-util.h"

static unsigned int test_run;
static unsigned int test_ok;

BufferType COMMIT_BUFFER = {
	.check = chewing_commit_Check,
	.get_string = chewing_commit_String,
};

BufferType PREEDIT_BUFFER = {
	.check = chewing_buffer_Check,
	.get_length = chewing_buffer_Len,
	.get_string = chewing_buffer_String,
};

BufferType ZUIN_BUFFER = {
	.check = chewing_zuin_Check,
	.get_string_alt = chewing_zuin_String,
};

BufferType AUX_BUFFER = {
	.check = chewing_aux_Check,
	.get_length = chewing_aux_Length,
	.get_string = chewing_aux_String,
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
					else
						result = KEY_SRIGHT;
					break;
				case 'T':
					result = KEY_TAB;
					break;
			}
		}
	}
	return result = END;
}

static void type_keystoke( ChewingContext *ctx, get_char_func get_char, void *param )
{
	int ch;
	int ctrl_shifted;

	while ( ( ch = get_keystroke( get_char, param ) ) != END ) {
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
			default:
				ctrl_shifted = ( ch - KEY_CTRL_BASE );
				if ( ( ctrl_shifted >= '0' ) && ( ctrl_shifted <= '9' ) ) {
					chewing_handle_CtrlNum( ctx, ctrl_shifted );
				} else {
					chewing_handle_Default( ctx, (char) ch );
				}
				break;
		}
	}
}

static int get_char_by_string( void * param )
{
	assert( param );

	char **ptr = param;

	if ( **ptr == 0 ) {
		return END;
	}

	char ch = **ptr;
	++*ptr;
	return ch;
}

void internal_ok( const char *file, int line, int test, const char * test_txt,
	const char *fmt, ...)
{
	++test_run;
	if ( test ) {
		++test_ok;
		printf( "ok %d ", test_run);

		va_list ap;
		va_start( ap, fmt );
		vprintf( fmt, ap );
		va_end( ap );

		printf("\n");
	} else {
		printf( "not ok %d ", test_run);

		va_list ap;
		va_start( ap, fmt );
		vprintf( fmt, ap );
		va_end( ap );

		printf( "\n# %s failed in %s:%d\n", test_txt, file, line );
	}
}

void type_keystoke_by_string( ChewingContext *ctx, const char* keystoke )
{
	return type_keystoke( ctx, get_char_by_string, &keystoke );
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
	int i;
	char *buf;

	assert( ctx );
	assert( cand );

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

int exit_status()
{
	return test_run == test_ok ? 0 : -1;
}
