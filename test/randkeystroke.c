/**
 * randkeystroke.c
 *
 * Copyright (c) 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#include <config.h>
#endif

#include "chewing.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

const char* zhuin_tab[] = {
	"1qaz2wsxedcrfv5tgbyhn", /* ㄅㄆㄇㄈㄉㄊㄋㄌㄍㄎㄏㄐㄑㄒㄓㄔㄕㄖㄗㄘㄙ */
	"ujm",                   /* ㄧㄨㄩ */
	"8ik,9ol.0p;/-",         /* ㄚㄛㄜㄝㄞㄟㄠㄡㄢㄣㄤㄥㄦ */
	"7634",                  /* ˙ˊˇˋ */
};

static char normal_keys[] = "abcdefghijklmnopqrstuvwxyz" \
                            "ABCDEFGHIJKLMNOPQRSTUVWXYZ" \
                            "`1234567890[]/=-?+_|!@#$%^&*(){} ";
static char* other_keys[] = {
    "<L>", "<SL>", "<R>", "<SR>", "<U>", "<D>", "<E>", "<B>", "<EE>", "<DC>", "<H>",
    "<EN>", "<T>", "<C0>", "<C1>", "<C2>", "<C3>", "<C4>", "<C5>", "<C6>",
    "<C7>", "<C8>", "<C9>", "<CB>", "<PU>", "<PD>", "<SS>", "<TT>",
    "<N0>", "<N1>", "<N2>", "<N3>", "<N4>", "<N5>", "<N6>", "<N7>", "<N8>", "<N9>",
    "<N+>", "<N->", "<N*>", "<N/>", "<N.>"};

#define n_nkeys (int)(sizeof(normal_keys) / sizeof(normal_keys[0]))
#define n_okeys (int)(sizeof(other_keys) / sizeof(other_keys[0]))

void usage()
{
	printf( "usage: randkeystroke [-r] [-n num] [-h]\n" \
		"\t -r     - total random\n" \
		"\t -n num - generate num keystrokes\n"
		"\t -s seed - random seed\n"
		"\t -h     - help\n" );
}

int main( int argc, char *argv[] )
{
	int nk = 100;
	int total_random = 0;
	int i, n;
	int n_tab1 = strlen(zhuin_tab[0]);
	int n_tab2 = strlen(zhuin_tab[1]);
	int n_tab3 = strlen(zhuin_tab[2]);
	int n_tab4 = strlen(zhuin_tab[3]);

	srand( time( NULL ) );

	for ( i = 1; i < argc; i++ ) {
		if ( ! strcmp( argv[i], "-n" ) ) {
			if ( ++i < argc )
				nk = atoi( argv[i] );
		} else if ( ! strcmp( argv[i], "-r" ) ) {
			total_random = 1;
		} else if ( ! strcmp( argv[i], "-s" ) ) {
		    	if ( ++i < argc )
			    	srand( atoi( argv[i] ) );
		} else if ( ! strcmp( argv[i], "-h" ) ) {
			usage();
			return 0;
		}
	}
	if (total_random) {
		for ( i = 0; i < nk; i++ ) {
			n = rand() % (n_nkeys + n_okeys);
			if ( n >= n_nkeys )
				printf( "%s", other_keys[n - n_nkeys] );
			else
				printf( "%c", normal_keys[n] );
		}
	} else {
		for ( i = 0; i < nk; i++ ) {
			if ( rand() % 2 )
				printf( "%c", zhuin_tab[0][ rand() % n_tab1 ] );
			if ( rand() % 2 )
				printf( "%c", zhuin_tab[1][ rand() % n_tab2 ] );
			if ( rand() % 2 )
				printf( "%c", zhuin_tab[2][ rand() % n_tab3 ] );
			if ( rand() % 2 )
				printf( "%c", zhuin_tab[3][ rand() % n_tab4 ] );
			else
				printf( " " );
			if ( rand() % 2 )
				printf( "<E>" );
		}
	}
	printf( "\n" );

	return 0;
}

/* vim: noet 
 */
