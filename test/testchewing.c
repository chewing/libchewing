/**
 * testhelper.hewing.c
 *
 * Copyright (c) 2004, 2005, 2008, 2011
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include "chewing.h"
#include "testhelper.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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
			case END:
				goto end;
			default:
				if ( KEY_CTRL_BASE <= i && i < KEY_NUMPAD_BASE)
					chewing_handle_CtrlNum( ctx, i - KEY_CTRL_BASE );
				else if ( KEY_NUMPAD_BASE <= i )
					chewing_handle_Numlock( ctx, i - KEY_NUMPAD_BASE );
				else
					chewing_handle_Default( ctx, (char) i );
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

