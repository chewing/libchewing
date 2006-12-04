/**
 * testchewing.c
 *
 * Copyright (c) 2004, 2005
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include "chewing.h"

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
#define KEY_ESC	994
#define KEY_DELETE 995
#define KEY_HOME 996
#define KEY_END 997
#define KEY_TAB 998
#define KEY_CAPSLOCK 999
#define KEY_CTRL_BASE 1000
#define END 2000

static char selKey_define[ 11 ] = "1234567890\0"; /* Default */

int get_keystroke()
{
	char ch;
	int result;
	int flag = 0;
	while ( ( ch = getchar() ) != EOF ) {
		if ( ( ch != '<' ) && ( flag != 1 ) )
			return (int) ch;
		else if ( ch == '>' ) {
			flag = 0;
			return result;
		}
		else {
			flag = 1;
			ch = getchar();
			switch ( ch ) {
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
					if ( ( ch = getchar() ) == '>' )
						return result = KEY_DOWN;
					else {
						getchar();
						return result = KEY_DELETE;
					}
					break;
				case 'E':
					if ( ( ch = getchar() ) == '>' )
						return result = KEY_ENTER;
					else if ( ch == 'E' )
						result = KEY_ESC;
					else
						result = KEY_END;
					break;
				case 'C':
					if ( ( ch = getchar() ) != '>' ) {
						if ( ( ch == 'B' ))
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
					if ( ( ch = getchar() ) == 'L' )
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

void commit_string( ChewingContext *ctx )
{
	int i;
	char *s;
	if ( chewing_commit_Check( ctx ) ) {
		s = chewing_commit_String( ctx );
		printf( "%s", s );
		free( s );
	}
}

int main( int argc, char *argv[] )
{
	ChewingConfigData config;
	ChewingContext *ctx;
	char *prefix = CHEWING_DATA_PREFIX;
	int i;
	int ctrl_shifted;

	/* Initialize libchewing */
	/* for the sake of testing, we should not change existing hash data */
	chewing_Init( prefix, TEST_HASH_DIR );

	/* Request handle to ChewingContext */
	ctx = chewing_new();

	/* Set keyboard type */ 
	chewing_set_KBType( ctx, chewing_KBStr2Num( "KB_DEFAULT" ) );

	/* Fill the configuration values */
	config.candPerPage = 20;
	config.maxChiSymbolLen = 16;

	for ( i = 0; i < 10; i++ )
		config.selKey[ i ] = selKey_define[ i ];
	/* Enable configurations */
	chewing_Configure( ctx, &config );

	while ( 1 ) {
		i = get_keystroke();
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
	}
end:
	/* Free Chewing IM handle */
	chewing_free( ctx );
	
	/* Termate Chewing services */
	chewing_Terminate();
	printf( "\n" );

	return 0;
}

