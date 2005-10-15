/**
 * gen_keystroke.c
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
#include <ncurses.h>

/* Avoid incorrect KEY_ENTER definition */
#ifdef KEY_ENTER
#undef KEY_ENTER
#endif

/* Key list */
#define KEY_ENTER	'\n'
#define KEY_TAB		'\t'
#define KEY_ESC		(27)
#define KEY_CTRL_A	(1)
#define KEY_CTRL_(n)	(KEY_CTRL_A + (n) - 'A')
#define KEY_SPACE	' '

#define CTRL_0		KEY_F0
#define CTRL_1		KEY_F(1)
#define CTRL_2		KEY_F(2)
#define CTRL_3		KEY_F(3)
#define CTRL_4		KEY_F(4)
#define CTRL_5		KEY_F(5)
#define CTRL_6		KEY_F(6)
#define CTRL_7		KEY_F(7)
#define CTRL_8		KEY_F(8)
#define CTRL_9		KEY_F(9)

/* Spacing */
#define FILL_LINE  "--------------------------------------------------------"
#define FILL_BLANK "                                                               "
static int hasColor = 0;
static char selKey_define[ 11 ] = "1234567890\0"; /* Default */

void drawline( int x, int y )
{
	move( x, y );
	addstr( FILL_LINE );
}

void show_edit_buffer( int x, int y, ChewingOutput *pgo )
{
	int i;
	move( x, y );
	addstr( FILL_BLANK );
	move( x, y );
	for ( i = 0; i < pgo->chiSymbolBufLen; i++ )
		addstr( pgo->chiSymbolBuf[ i ].s );
}

void show_interval_buffer( int x, int y, ChewingOutput *pgo )
{
	char out_buf[ 100 ];
	int i, count;
	int arrPos[ MAX_PHONE_SEQ_LEN ];

	move( x, y );
	addstr( FILL_BLANK );
	move( x, y );

	if ( pgo->chiSymbolBufLen == 0 ) {
                return;
        }

	count = 0;
	for ( i = 0 ;i < pgo->chiSymbolBufLen; i++ ) {
		arrPos[ i ] = count;
		count += strlen( pgo->chiSymbolBuf[ i ].s );
	}
	arrPos[ i ] = count;

	memset( out_buf, ' ', count * ( sizeof( char ) ) );
	out_buf[ count ] = '\0';

	for ( i = 0; i < pgo->nDispInterval; i++ ) {
		if ( ( pgo->dispInterval[ i ].to - pgo->dispInterval[ i ].from ) == 1 ) {
			out_buf[ arrPos[ pgo->dispInterval[ i ].from ] ] = ' ';
			out_buf[ arrPos[ pgo->dispInterval[ i ].to ] - 1 ] = ' ';
		}
		else {	
			out_buf[ arrPos[ pgo->dispInterval[ i ].from ] ] = '[';
			out_buf[ arrPos[ pgo->dispInterval[ i ].to ] - 1 ] =  ']';
		}
		memset(
			&out_buf[ arrPos[ pgo->dispInterval[ i ].from ] + 1 ], '-',
			arrPos[ pgo->dispInterval[ i ].to ] - arrPos[ pgo->dispInterval[ i ].from ] - 2 );
	}
	addstr( out_buf );
}

void showZuin( ChewingOutput *pgo )
{
	int i, a;
	if ( pgo->bChiSym )
		addstr( "[中]" );
	else
		addstr( "[英]" );
	addstr( "        " );
	for ( i = 0, a = 2; i < ZUIN_SIZE; i++ ) {
		if ( pgo->zuinBuf[ i ].s[ 0 ] != '\0' ) {
			addstr( pgo->zuinBuf[ i ].s );
		}
	}
}

void show_zuin_buffer( int x, int y, ChewingOutput *pgo )
{
	move( x, y );
	addstr( FILL_BLANK );
	move( x, y );
	if ( hasColor )
		attron( COLOR_PAIR( 1 ) );
	showZuin( pgo );
	if ( hasColor )
		attroff( COLOR_PAIR( 1 ) );
}

void show_full_shape( int x, int y, ChewingData *da )
{
	move( x, y );
	addstr( "[" );
	if ( hasColor )
		attron( COLOR_PAIR( 2 ) );
	if ( da->bFullShape )
		addstr( "全" );
	else
		addstr( "半" );
	if ( hasColor )
		attroff( COLOR_PAIR( 2 ) );
	addstr( "]" );
}

void show_userphrase( int x, int y, wch_t showMsg[], int len )
{
	char out_buf[ 40 ];
	int i;

	memset( out_buf, 0, sizeof( out_buf ) );
	for ( i = 0; i < len; i++ ) {
		strcat( out_buf, showMsg[ i ].s );
	}
	move( x, y );
	addstr( FILL_BLANK );
	move( x, y );
	if ( hasColor )
		attron( COLOR_PAIR( 2 ) );
	addstr( out_buf );
	if ( hasColor )
		attroff( COLOR_PAIR( 2 ) );
}

void show_choose_buffer( int x, int y, ChewingOutput *pgo )
{
	int i, no, len;
	char str[ 20 ];
	move( x, y );
	addstr( FILL_BLANK );
	move( x, y );

	if ( pgo->pci->nPage != 0 ) {
		no = pgo->pci->pageNo * pgo->pci->nChoicePerPage;
		len = 0;

		for ( i = 0; i < pgo->pci->nChoicePerPage; no++, i++ ) {
			if ( no >= pgo->pci->nTotalChoice )
				break;
			sprintf( str, "%d.", i + 1 );
			if ( hasColor )
				attron( COLOR_PAIR( 3 ) );
			addstr( str );
			if ( hasColor )
				attroff( COLOR_PAIR( 3 ) );
			sprintf( str, " %s ", pgo->pci->totalChoiceStr[ no ] );			
			addstr( str );
		}
		if ( pgo->pci->nPage != 1 ) {
			if ( pgo->pci->pageNo == 0 )
				addstr( "  >" );
			else if ( pgo->pci->pageNo == ( pgo->pci->nPage - 1 ) )
				addstr( "<  " );
			else
				addstr( "< >" );
		}
	}
}

void show_commit_string( ChewingOutput *pgo )
{
	static int x = 12;
	static int y = 0;
	int i;
	if ( pgo->keystrokeRtn & KEYSTROKE_COMMIT ) {
		for ( i = 0; i < pgo->nCommitStr; i++ ) {
			mvaddstr( x, y, pgo->commitStr[ i ].s );
			y = ( y >= 54 ) ?
				0 : 
				( y + strlen( pgo->commitStr[ i ].s ) );
			x = ( y == 0 ) ? ( x + 1 ) : x;
		}
	}
}

void set_cursor( int x, ChewingOutput *pgo )
{
	int i, count;

	for ( count = 0, i = 0; i < pgo->chiSymbolCursor; i++) {
		count += strlen( pgo->chiSymbolBuf[ i ].s );
	}
	move( x, count );
}

int main( int argc, char *argv[] )
{
	ChewingConf *cf = (ChewingConf *) malloc( sizeof( ChewingConf ) );
	ChewingData *da = (ChewingData *) malloc( sizeof( ChewingData ) );
        ConfigData config;
	ChewingOutput gOut;
	FILE *fout;
	char *prefix = CHEWING_DATA_PREFIX;
	int ch;
	int i;
	int width, height;
	int add_phrase_length;

	if ( argc < 2 ) {
		fprintf( stderr, "usage: genkeystroke filename\n" );
		exit( 1 );
	}
	else {
		fout = fopen( argv[ 1 ], "w" );
		if ( ! fout ) {
			fprintf( stderr, "Error: failed to open %s\n", argv[ 1 ] );
			exit( 1 );
		}
	}

	/* Initialize curses library */
	initscr();
	if ( has_colors() == TRUE ) {
		start_color();
		init_pair( 1, COLOR_WHITE, COLOR_BLUE );
		init_pair( 2, COLOR_RED, COLOR_YELLOW );
		init_pair( 3, COLOR_WHITE, COLOR_RED );
		hasColor = 1;
	}
	cbreak();
	noecho();
	keypad( stdscr, 1 );
	getmaxyx( stdscr, height, width );
	start_color();
	clear();
	refresh();

	/* Initialize libchewing */
	cf->kb_type = KBStr2Num( "KB_DEFAULT" );
	cf->inp_cname = ( char * ) strdup( "新酷音" );
	cf->inp_ename = ( char * ) strdup( "Chewing" );
	ReadTree( prefix );
	InitChar( prefix );
	InitDict( prefix );
	/* for the sake of testing, we should not change existing hash data */
	ReadHash( TEST_HASH_DIR );
	InitChewing( da, cf );

        config.selectAreaLen = 55;
        config.maxChiSymbolLen = 16;
	config.bAddPhraseForward = 1;

        for ( i = 0; i < 10; i++ )
                config.selKey[ i ] = selKey_define[ i ];
        SetConfig( da, &config );

	mvaddstr( 0, 0, "Any key to start testing..." );

	while ( TRUE ) {
		ch = getch();
		switch ( ch ) {
			case KEY_LEFT:
				OnKeyLeft( da, &gOut );
				fprintf( fout, "<L>" );
				break;
			case KEY_SLEFT:
				OnKeyShiftLeft( da, &gOut );
				fprintf( fout, "<SL>" );
				break;
			case KEY_RIGHT:
				OnKeyRight( da, &gOut );
				fprintf( fout, "<R>" );
				break;
			case KEY_SRIGHT:
				OnKeyShiftRight( da, &gOut );
				fprintf( fout, "<SR>" );
				break;
			case KEY_UP:
				OnKeyUp( da, &gOut );
				fprintf( fout, "<U>" );
				break;
			case KEY_DOWN:
				OnKeyDown( da, &gOut );
				fprintf( fout, "<D>" );
				break;
			case KEY_SPACE:
				OnKeySpace( da, &gOut );
				fprintf( fout, " " );
				break;
			case KEY_ENTER:
				OnKeyEnter( da, &gOut );
				fprintf( fout, "<E>" );
				break;
			case KEY_BACKSPACE:
				OnKeyBackspace( da, &gOut );
				fprintf( fout, "<B>" );
				break;
			case KEY_ESC:
				OnKeyEsc( da, &gOut );
				fprintf( fout, "<EE>" );
				break;
			case KEY_DC:
				OnKeyDel( da, &gOut );
				fprintf( fout, "<DC>" );
				break;
			case KEY_HOME:
				OnKeyHome( da, &gOut );
				fprintf( fout, "<H>" );
				break;
			case KEY_END:
				OnKeyEnd( da, &gOut );
				fprintf( fout, "<EN>" );
				break;
			case KEY_TAB:
				OnKeyTab( da, &gOut );
				fprintf( fout, "<T>" );
				break;
			case CTRL_0:
			case CTRL_1:
			case CTRL_2:
			case CTRL_3:
			case CTRL_4:
			case CTRL_5:
			case CTRL_6:
			case CTRL_7:
			case CTRL_8:
			case CTRL_9:
				add_phrase_length = ( ch - CTRL_0 + '0' );
				OnKeyCtrlNum( 
					da, 
					add_phrase_length, 
					&gOut );
				fprintf( fout, "<C%c>", add_phrase_length );
				break;
			#if 0
			case XK_Caps_Lock:
				OnKeyCapslock(da, &gOut);
				break;
			#endif
			case KEY_CTRL_('B'): /* emulate CapsLock */
				OnKeyCapslock(da, &gOut);
				fprintf( fout, "<CB>");
				break;
			case KEY_CTRL_('D'):
				goto end;
			case KEY_CTRL_('K'): /* emulate Shift */
				if ( da->bFullShape == FULLSHAPE_MODE )
					da->bFullShape = HALFSHAPE_MODE;
				else
					da->bFullShape = FULLSHAPE_MODE;
				break;
			default:
				OnKeyDefault( da, (char) ch, &gOut );
				fprintf( fout, "%c", (char) ch );
				break;
		}
		drawline( 0, 0 );
		show_edit_buffer( 1, 0, &gOut );
		drawline( 2, 0 );
		show_interval_buffer( 3, 0, &gOut );
		drawline( 4, 0 );
		show_choose_buffer( 5, 0, &gOut );
		drawline( 6, 0 );
		show_zuin_buffer( 7, 0, &gOut );
		show_full_shape( 7, 5, da );
		drawline( 8, 0 );
		mvaddstr( 9, 0, "Ctrl + d : leave" );
		mvaddstr( 9, 20, "Ctrl + b : toggle Eng/Chi mode" );
		mvaddstr( 10, 0, "F1, F2, F3, ..., F9 : Add user defined phrase");
		mvaddstr( 11, 0, "Crtl + k : toggle Full/Half shape mode" );
		show_commit_string( &gOut );
		if ( da->showMsgLen > 0 ) {
			show_userphrase( 7, 12, gOut.showMsg, gOut.showMsgLen );
			gOut.showMsgLen = 0;
		}
		set_cursor( 1, &gOut );
	}
end:
	endwin();

	fprintf( fout, "\n" );
	fclose( fout );
	return 0;
}

/* vim:tenc=big5:
 * */
