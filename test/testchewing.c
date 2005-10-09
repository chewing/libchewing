/**
 * testchewing.c
 *
 * Copyright (c) 2004
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

void commit_string( ChewingOutput *pgo )
{
	int i;
	if ( pgo->keystrokeRtn & KEYSTROKE_COMMIT ) {
		for ( i = 0; i < pgo->nCommitStr; i++ )	{
			printf( "%s", pgo->commitStr[ i ].s );
		}
	}
}

int main( int argc, char *argv[] )
{
	ChewingConf *cf = (ChewingConf *) malloc( sizeof( ChewingConf ) );
	ChewingData *da = (ChewingData *) malloc( sizeof( ChewingData ) );
        ConfigData config;
	ChewingOutput gOut;
	char *prefix = CHEWING_DATA_PREFIX;
	int i;
	int ctrl_shifted;

	/* Initialize libchewing */
	cf->kb_type = KBStr2Num( "KB_DEFAULT" );
	cf->inp_cname = (char *) strdup( "·s»Å­µ" );
	cf->inp_ename = (char *) strdup( "Chewing" );
	ReadTree( prefix );
	InitChar( prefix );
	InitDict( prefix );
	/* for the sake of testing, we should not change existing hash data */
	ReadHash( TEST_HASH_DIR );
	InitChewing( da, cf );

        config.selectAreaLen = 40;
        config.maxChiSymbolLen = 16;

        for ( i = 0; i < 10; i++ )
                config.selKey[ i ] = selKey_define[ i ];
        SetConfig( da, &config );

	while ( 1 ) {
		i = get_keystroke();
		switch ( i ) {
			case KEY_LEFT:
				OnKeyLeft( da, &gOut );
				break;
			case KEY_RIGHT:
				OnKeyRight( da, &gOut );
				break;
			case KEY_UP:
				OnKeyUp( da, &gOut );
				break;
			case KEY_DOWN:
				OnKeyDown( da, &gOut );
				break;
			case KEY_SPACE:
				OnKeySpace( da, &gOut );
				break;
			case KEY_ENTER:
				OnKeyEnter( da, &gOut );
				break;
			case KEY_BACKSPACE:
				OnKeyBackspace( da, &gOut );
				break;
			case KEY_ESC:
				OnKeyEsc( da, &gOut );
				break;
			case KEY_DELETE:
				OnKeyDel( da, &gOut );
				break;
			case KEY_HOME:
				OnKeyHome( da, &gOut );
				break;
			case KEY_END:
				OnKeyEnd( da, &gOut );
				break;
			case KEY_TAB:
				OnKeyTab( da, &gOut );
				break;			
			#if 0
			case XK_Caps_Lock:
				OnKeyCapslock(da, &gOut);
				break;
			#endif
			case KEY_CAPSLOCK:
				OnKeyCapslock( da, &gOut );
				break;
			case END:
				goto end;
			default:
				ctrl_shifted = ( i - KEY_CTRL_BASE );
				if ( ( ctrl_shifted >= '0' ) && ( ctrl_shifted <= '9' ) ) {
					OnKeyCtrlNum( da, ctrl_shifted, &gOut );
				} else {
					OnKeyDefault( da, (char) i, &gOut );
				}
				break;
		}
		commit_string( &gOut );
	}
end:
	TerminateChewing();
	printf( "\n" );
	if (da)
		free( da );
	if (cf) {
		free( cf->kb_type );
		free( cf->inp_cname );
		free( cf->inp_ename );
		free( cf );
	}

	return 0;
}

/* vim:tenc=big5:
 * */
