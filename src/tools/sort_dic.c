/**
 * sort_dic.c
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004, 2006
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/**
 * @file  sort_dic.c
 * @brief Sort and Index dictionary.\n
 *        Generate \b ph_index.dat (dictionary index) and \b dict.dat (content of dictionary) from
 *	  \b tsi.src (dictionary file in libtabe standard).
 *
 *	  Read dictionary format :
 *  	  phrase   frequency   zuin1 zuin2 zuin3 ... \n
 *  	  Output format : ( Sorted by zuin's uint16 number )
 *  	  phrase   frequency   zuin1 zuin2 zuin3 ... \n
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "global.h"
#include "global-private.h"
#include "key2pho-private.h"
#include "config.h"

#define MAXLEN		149
#define MAXZUIN		9
#define MAX_FILE_NAME	(256)

#define IN_FILE		"phoneid.dic"

typedef struct {
	char str[ MAXLEN ];
	int freq;
	uint16 num[ MAXZUIN ];
} RECORD;

RECORD data[ 420000L ];
long nData;

const char user_msg[] = 
	"sort_dic -- read chinese phrase input and generate data file for chewing\n" \
	"usage: \n" \
		"\tsort_dic <tsi file name> or \n" \
		"\tsort_dic (default name is tsi.src) \n" \
		"This program creates three new files. \n" \
		"1." DICT_FILE " \t-- main dictionary file \n" \
		"2." PH_INDEX_FILE " \t-- index file of phrase \n" \
		"3." IN_FILE " \t-- intermediate file for make_tree \n";

extern const char *ph_pho[];
/*extern uint16 PhoneBg2Uint( const char *phone );*/

void DataSetNum( long _index )
{
	char buf[ MAXLEN ], *p;
	int i = 0;

	strcpy( buf, data[ _index ].str );
	strtok( buf, " \n\t" );
	data[ _index ].freq = atoi( strtok( NULL, " \n\t" ) );
	for ( p = strtok( NULL, " \n\t" ); p; p = strtok( NULL, " \n\t" ) ) 
		data[ _index ].num[ i++ ] = UintFromPhone( p );
}

void DataStripSpace( long _index )
{
	long i, k = 0;
	char old[ MAXLEN ], last = ' ';
		/* If the first charactor of line in tsi.src is ' ',
		 * then it should be ignore? 
		 */

	strcpy( old, data[ _index ].str );
	for ( i = 0; old[ i ]; i++ ) {
		/* trans '\t' to ' ' , easy for process. */
		if ( old[ i ] == '\t' )
			old[ i ] = ' ';
		if ( old[ i ] == ' ' && last == ' ' )
			continue;
		/* Ignore '#' comment in tsi.src */
		if ( old[ i ] == '#') {
			data[ _index ].str[ k++ ] = '\n';
			break;
		}
		data[ _index ].str[ k++ ] = old[ i ];
		last = old[ i ];
	}
	data[ _index ].str[ k ] = '\0';
}

void DataStripAll( long _index )
{
	char *p;

	p = strchr( data[ _index ].str, ' ' );
	if ( p )
		*p = '\0';
}

int CompRecord( const void *a, const void *b )
{
	long i;
	int cmp;

	for ( i = 0; i < MAXZUIN; i++ ) {
		cmp = ((RECORD *) a)->num[ i ] - ((RECORD *) b)->num[ i ];
		if ( cmp )
			return cmp;
	}
	return ( ((RECORD *) b)->freq - ((RECORD *) a)->freq );
}

int CompUint( long a, long b )
{
	long i;

	for ( i = 0; i < MAXZUIN; i++ ) {
		if ( data[ a ].num[ i ] != data[ b ].num[ i ] )
			return 1;
	}
	return 0;
}

int main( int argc, char *argv[] )
{
	FILE *infile;
	FILE *dictfile, *treedata, *ph_index;
	char in_file[ MAX_FILE_NAME ] = "tsi.src";
	long i, k;
    int tmp;
#ifdef USE_BINARY_DATA
	unsigned char size;
#endif

	if ( argc < 2 ) 
		printf( user_msg );
	else 
		strcpy( in_file, argv[ 1 ] );

	infile = fopen( in_file, "r" );

	if ( !infile ) {
		fprintf ( stderr, "Error opening %s for reading!\n", in_file );
		exit( -1 );
	}

#ifdef USE_BINARY_DATA
	dictfile = fopen( DICT_FILE, "wb" );
	ph_index = fopen( PH_INDEX_FILE, "wb" );
#else
	dictfile = fopen( DICT_FILE, "w" );
	ph_index = fopen( PH_INDEX_FILE, "w" );
#endif
	treedata = fopen( IN_FILE, "w" );

	if ( !dictfile || !treedata || !ph_index ) {
		fprintf( stderr, "Error opening output file!\n" );
		exit( -1 );
	}

	while ( fgets( data[ nData ].str, MAXLEN, infile ) ) {
		DataStripSpace( nData );
		/* Ignore '#' comment for tsi.src */
		if ( data[ nData ].str[0] == '\n' )
			continue;
		DataSetNum( nData );
		DataStripAll( nData );
		nData++;
	}
	qsort( data, nData, sizeof( RECORD ), CompRecord );

	for ( i = 0; i < nData - 1; i++ ) {
		if ( ( i == 0 ) || ( CompUint( i, i - 1 ) != 0 ) )  {
#ifdef USE_BINARY_DATA
			tmp = ftell( dictfile );
			fwrite( &tmp, sizeof( tmp ), 1, ph_index );
#else
			fprintf( ph_index, "%ld\n", ftell( dictfile ) );
#endif
		}
#ifdef USE_BINARY_DATA
		size = sizeof( char ) * strlen( data[ i ].str );
		fwrite( &size, sizeof( unsigned char ), 1, dictfile );
		fwrite( data[ i ].str, size, 1, dictfile );
		fwrite( &data[ i ].freq, sizeof( int ), 1, dictfile );
#else
		fprintf( dictfile, "%s %d\t", data[ i ].str, data[ i ].freq );
#endif
	}
#ifdef USE_BINARY_DATA
	tmp = ftell( dictfile );
	fwrite( &tmp, sizeof( tmp ), 1, ph_index );
	size = sizeof( char ) * strlen( data[ nData - 1 ].str );
	fwrite( &size, sizeof( unsigned char ), 1, dictfile );
	fwrite( data[ nData - 1 ].str, size, 1, dictfile );
	fwrite( &data[ nData - 1 ].freq, sizeof( int ), 1, dictfile );
	tmp = ftell( dictfile );
	fwrite( &tmp, sizeof( tmp ), 1, ph_index );
#else
	fprintf( ph_index, "%ld\n", ftell( dictfile ) ); 
	fprintf( dictfile, "%s %d", data[ nData - 1 ].str, data[ nData - 1 ].freq );
	fprintf( ph_index, "%ld\n", ftell( dictfile ) );
#endif

	for ( i = 0; i < nData; i++ ) {
		if ( ( i > 0 ) && ( CompUint( i, i - 1 ) == 0 ) )
			continue;

		for ( k = 0; data[ i ].num[ k ]; k++ )
			fprintf (treedata, "%hu ", data[ i ].num[ k ] );
		fprintf( treedata, "0\n" );
	}
	fclose( infile );
	fclose( ph_index );
	fclose( dictfile );
	fclose( treedata );
	return 0;
}

