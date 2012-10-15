/**
 * sort_word.c
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004-2006, 2008, 2010, 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

#include "global-private.h"
#include "chewing-private.h"
#include "key2pho-private.h"
#include "zuin-private.h"
#include "config.h"

#define CHARDEF_BEGIN	"%chardef  begin"
#define CHARDEF_END	"%chardef  end"
#define DO_WORD_ERROR (1)
#define MAX_WORD	(60000)
#define MAX_NUMBER	(12000)
#define MAX_BUF_LEN	(4096)

typedef struct {
	uint16 num;
	char word[ 8 ];
} WORD_DATA;

WORD_DATA word_data[ MAX_WORD ];
int nWord;
int phone_num;

int SortWord( const WORD_DATA *a, const WORD_DATA *b )
{
	return ( a->num - b->num );
}

int DoWord( char *buf )
{
	char keyBuf[ 128 ], phoneBuf[ 128 ];
	int phoneInx[ ZUIN_SIZE ];

	memset( phoneInx, 0, sizeof( phoneInx ) );
	sscanf( buf, "%s %s", keyBuf, word_data[ nWord ].word );
	if ( strlen( keyBuf ) > ZUIN_SIZE )
		return DO_WORD_ERROR;

	PhoneFromKey( phoneBuf, keyBuf, KB_DEFAULT, 1 );
	word_data[ nWord ].num = UintFromPhone( phoneBuf );
	nWord++ ;
	return 0;
}

void Output()
{
	FILE *indexfile, *datafile, *configfile;
	int i;
	uint16 previous;

#ifdef USE_BINARY_DATA
	int tmp;
	unsigned char size;
	FILE *indexfile2;
	indexfile = fopen( CHAR_INDEX_BEGIN_FILE, "wb" );
	indexfile2 = fopen( CHAR_INDEX_PHONE_FILE, "wb" );
	datafile = fopen( CHAR_FILE, "wb" );
#else
	indexfile = fopen( CHAR_INDEX_FILE, "w" );
	datafile = fopen( CHAR_FILE, "w" );
#endif
	configfile = fopen( CHEWING_DEFINITION_FILE, "a" );
	if ( ! indexfile || ! datafile || ! configfile ) {
		fprintf( stderr, "File Write Error\n" );
		exit( 1 );
	}

	previous = 0 ;
	phone_num = 0;
	for ( i = 0; i < nWord; i++ ) {
		if ( word_data[ i ].num != previous ) {
			previous = word_data[ i ].num;
#ifdef USE_BINARY_DATA
			tmp = ftell( datafile );
			fwrite( &tmp, sizeof(int), 1, indexfile );
			fwrite( &previous, sizeof(uint16), 1, indexfile2 );
#else
			fprintf( indexfile, "%hu %ld\n", previous, ftell( datafile ) );
#endif
			phone_num++;
		}
#ifdef USE_BINARY_DATA
		size = strlen( word_data[ i ].word );
		fwrite( &size, sizeof(size), 1, datafile );
		fwrite( word_data[ i ].word, size, 1, datafile );
#else
		fprintf( datafile, "%hu %s\t", word_data[ i ].num, word_data[ i ].word );
#endif
	}
#ifdef USE_BINARY_DATA
	tmp = ftell( datafile );
	fwrite( &tmp, sizeof(int), 1, indexfile );
	previous = 0;
	fwrite( &previous, sizeof(uint16), 1, indexfile2 );
#else
	fprintf( indexfile, "0 %ld\n", ftell( datafile ) );
#endif
	fprintf( configfile, "#define PHONE_NUM (%d)\n", phone_num );
	fclose( indexfile );
#ifdef USE_BINARY_DATA  
	fclose( indexfile2 );  
#endif
	fclose( datafile );
	fclose( configfile );
}

void CountSort()
{
	int number[ MAX_NUMBER ], i, place;
	WORD_DATA oldData[ MAX_WORD ];

	memset( number, 0, sizeof( number ) );
	for ( i = 0; i < nWord; i++ )
		number[ word_data[ i ].num ]++;
	memmove( &number[ 1 ], number, sizeof( int ) * ( MAX_NUMBER - 1 ) );
	for ( i = 2; i < MAX_NUMBER; i++)
		number[ i ] += number[ i - 1 ];

	memcpy( oldData, word_data, sizeof( WORD_DATA ) * nWord );
	for ( i = 0; i < nWord; i++ ) {
		place = number[ oldData[ i ].num ]++;
		memcpy( &word_data[ place ], &oldData[ i ], sizeof( WORD_DATA ) );
	}
}

int main(int argc, char* argv[])
{
	FILE *cinfile;
	char buf[ MAX_BUF_LEN ];
	char *phone_cin;
	char *ret;

	if (argc < 2) {
		fprintf( stderr, "Usage: sort_word <phone.cin>\n" );
		return 1;
	}

	phone_cin = argv[1];
	cinfile = fopen( phone_cin, "r" );
	if ( ! cinfile ) {
		fprintf( stderr, "Error opening the file %s\n", phone_cin );
		return 1;
	}

	do {
		ret = fgets( buf, MAX_BUF_LEN, cinfile );
		if ( !ret ) {
			fprintf( stderr, "Cannot find %s", CHARDEF_BEGIN );
			return 1;
		}
	} while ( strncmp( buf, CHARDEF_BEGIN, strlen( CHARDEF_BEGIN ) ) );

	for ( ; ; ) {
		ret = fgets( buf, MAX_BUF_LEN, cinfile );
		if ( !ret || buf[ 0 ] == '%' )
			break;
		if ( DoWord( buf ) == DO_WORD_ERROR ) {
			fprintf( stderr, "The file %s is corrupted!\n", phone_cin );
			return 1;
		}
	}
	fclose( cinfile );

	if ( strncmp( buf, CHARDEF_END, strlen( CHARDEF_END ) ) ) {
		fprintf( stderr, "The end of the file %s is error!\n", phone_cin );
		return 1;
	}

	CountSort();
	Output();

	return 0;
}

