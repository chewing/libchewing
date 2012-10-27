/**
 * testchewing.c
 *
 * Copyright (c) 2004, 2005, 2008, 2011
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include "chewing.h"
#include "test.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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

#ifdef USED_IN_SIMULATION
#define MAXLEN 149
char commit_string_buf[ MAXLEN ];
char expect_string_buf[ MAXLEN ];
#define getchar fake_getchar
int fake_getchar();
#include "internal/chewing-utf8-util.h"
int tested_word_count = 0;
int failed_word_count = 0;
#endif

static int selKey_define[ 11 ] = {'1','2','3','4','5','6','7','8','9','0',0}; /* Default */

static int get_char( void *param )
{
	int ch = getchar();
	if ( ch == EOF )
		return END;
	return ch;
}

void commit_string( ChewingContext *ctx )
{
	char *s;
	if ( chewing_commit_Check( ctx ) ) {
		s = chewing_commit_String( ctx );
#ifdef USED_IN_SIMULATION
		strcat( commit_string_buf, s );
#else
		printf( "%s", s );
#endif
		free( s );
	}
}

#ifdef USED_IN_SIMULATION
void compare_per_run()
{
	int i, len;
	char utf8buf_expect[16];
	char utf8buf_commit[16];
	printf( "Expected:  %s", expect_string_buf );
	printf( "Committed: ");

	tested_word_count += (len = ueStrLen( expect_string_buf ) - 1);
		/* omit the suffix character */
	for ( i = 0; i < len; i++ ) {
		ueStrNCpy( utf8buf_expect,
		           ueStrSeek( expect_string_buf, i ),
			   1, STRNCPY_CLOSE );
		ueStrNCpy( utf8buf_commit,
		           ueStrSeek( commit_string_buf, i ),
			   1, STRNCPY_CLOSE );
		if ( ! strcmp( utf8buf_expect, utf8buf_commit ) )
			printf( "%s", utf8buf_commit );
		else {
			printf( "\033[44;37m%s\033[m", utf8buf_commit );
			failed_word_count++;
		}
	}
	memset( commit_string_buf, 0, MAXLEN );
	printf( "\n\n" );
}

/* entry point for simulation */
int chewing_test_Main()
#else
int main( int argc, char *argv[] )
#endif
{
	ChewingContext *ctx;
	char *prefix = CHEWING_DATA_PREFIX;
	int i;
	int ctrl_shifted;

	/* Initialize libchewing */
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	/* for the sake of testing, we should not change existing hash data */
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );
	chewing_Init( prefix, TEST_HASH_DIR );

	/* Request handle to ChewingContext */
	ctx = chewing_new();

	/* Set keyboard type */ 
	chewing_set_KBType( ctx, chewing_KBStr2Num( "KB_DEFAULT" ) );

	chewing_set_candPerPage( ctx, 9 );
	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_addPhraseDirection( ctx, 1 );
	chewing_set_selKey( ctx, selKey_define, 10 );
	chewing_set_spaceAsSelection( ctx, 1 );

	while ( 1 ) {
		i = get_keystroke( get_char, NULL );
		switch ( i ) {
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
			case END:
				goto end;
			default:
				ctrl_shifted = ( i - KEY_CTRL_BASE );
				if ( ( ctrl_shifted >= '0' ) && ( ctrl_shifted <= '9' ) ) {
					chewing_handle_CtrlNum( ctx, ctrl_shifted );
				} else {
					chewing_handle_Default( ctx, (char) i );
				}
				break;
		}
		commit_string( ctx );
#ifdef USED_IN_SIMULATION
		if ( i == KEY_ENTER )
			compare_per_run();
#endif
	}
end:
	/* Free Chewing IM handle */
	chewing_delete( ctx );
	
	/* Termate Chewing services */
	chewing_Terminate();
#ifndef USED_IN_SIMULATION
	printf( "\n" );
#endif
	return 0;
}

