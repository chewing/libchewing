/**
 * char.c
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004, 2005, 2006, 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/** 
 * @file char.c
 * @brief word data file
 */
#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <string.h>

#include "global-private.h"
#include "chewing-definition.h"
#include "char-private.h"
#include "private.h"
#include "plat_mmap.h"

#ifdef USE_BINARY_DATA
static uint16* arrPhone = NULL;
static int *begin = NULL;
static size_t phone_num;
static plat_mmap char_begin_mmap;
static plat_mmap char_phone_mmap;
static void *dict = NULL;
static void *cur_pos = NULL;
static plat_mmap dict_mmap;
#else
static uint16 arrPhone[ PHONE_NUM + 1 ];
static int begin[ PHONE_NUM + 1 ];
static FILE *dictfile;
#endif
static int end_pos;

#if ! defined(USE_BINARY_DATA)
static char *fgettab( char *buf, int maxlen, FILE *fp )
{
	int i;

	for ( i = 0; i < maxlen; i++ ) {
		buf[ i ] = (char) fgetc( fp );
		if ( feof( fp ) )
			break;
		if ( buf[ i ] == '\t' )
			break;
	}
	if ( feof( fp ) )
		return 0;
	buf[ i ] = '\0';
	return buf;
}
#endif

static int CompUint16( const uint16 *pa, const uint16 *pb )
{
	return ( (*pa) - (*pb) );
}

static void TerminateChar()
{
#ifdef USE_BINARY_DATA
	plat_mmap_close( &char_begin_mmap );
	plat_mmap_close( &char_phone_mmap );
	plat_mmap_close( &dict_mmap );
#else
	if ( dictfile )
		fclose( dictfile );
#endif
}

int InitChar( const char *prefix )
{
	char filename[ PATH_MAX ];

#ifdef USE_BINARY_DATA
	long file_size;
	size_t offset = 0;
	size_t csize;
#else
	FILE *indexfile;
	int i;
#endif

	sprintf( filename, "%s" PLAT_SEPARATOR "%s", prefix, CHAR_FILE );
#ifdef USE_BINARY_DATA
	plat_mmap_set_invalid( &dict_mmap );
	file_size = plat_mmap_create( &dict_mmap, filename, FLAG_ATTRIBUTE_READ );
	assert( plat_mmap_is_valid( &dict_mmap ) );
	if ( file_size < 0 )
		return 0;
	csize = file_size;
	dict = (void *) plat_mmap_set_view( &dict_mmap, &offset, &csize );
	assert( dict );
#else
	dictfile = fopen( filename, "r" );
    if ( ! dictfile )
        return 0;
#endif

#ifdef USE_BINARY_DATA
	sprintf( filename, "%s" PLAT_SEPARATOR "%s", prefix, CHAR_INDEX_BEGIN_FILE );
	plat_mmap_set_invalid( &char_begin_mmap );
	file_size = plat_mmap_create( &char_begin_mmap, filename, FLAG_ATTRIBUTE_READ );
	assert( plat_mmap_is_valid( &char_begin_mmap ) );
	if ( file_size < 0 )
		return 0;

	phone_num = file_size / sizeof(int);

	csize = file_size;
	begin = (int *) plat_mmap_set_view( &char_begin_mmap, &offset, &csize );
	assert( begin );
	if ( ! begin )
		return 0;

	sprintf( filename, "%s" PLAT_SEPARATOR "%s", prefix, CHAR_INDEX_PHONE_FILE );
	plat_mmap_set_invalid( &char_phone_mmap );
	file_size = plat_mmap_create( &char_phone_mmap, filename, FLAG_ATTRIBUTE_READ );
	assert( plat_mmap_is_valid( &char_phone_mmap ) );
	if ( file_size < 0 )
		return 0;

	assert( phone_num == file_size / sizeof(uint16) );

	offset = 0;
	csize = file_size;
	arrPhone = (uint16 *) plat_mmap_set_view( &char_phone_mmap, &offset, &csize );
	assert( arrPhone );
	if ( ! arrPhone )
		return 0;
#else
	sprintf( filename, "%s" PLAT_SEPARATOR "%s", prefix, CHAR_INDEX_FILE );
	indexfile = fopen( filename, "r" );

	if ( ! dictfile || ! indexfile )
		return 0;

	for ( i = 0; i <= PHONE_NUM; i++ )
		fscanf( indexfile, "%hu %d", &arrPhone[ i ], &begin[ i ] );
	fclose( indexfile );
#endif
	addTerminateService( TerminateChar );
	return 1;
}

static void Str2Word( Word *wrd_ptr )
{
#ifndef USE_BINARY_DATA
	char buf[ 1000 ];
	uint16 sh;

	fgettab( buf, 1000, dictfile );
	/* only read 6 bytes to wrd_ptr->word avoid buffer overflow */
	sscanf( buf, "%hu %6[^ ]", &sh, wrd_ptr->word );
	assert( wrd_ptr->word != '\0' );
#else
	unsigned char size;
	size = *(unsigned char *) cur_pos;
	cur_pos = (unsigned char*)cur_pos + sizeof(unsigned char);
	memcpy( wrd_ptr->word, cur_pos, size );
	cur_pos = (unsigned char*)cur_pos + size;
	wrd_ptr->word[ size ] = '\0';
#endif
}

int GetCharFirst( Word *wrd_ptr, uint16 phoneid )
{
	uint16 *pinx;

	pinx = (uint16 *) bsearch(
#ifdef USE_BINARY_DATA
		&phoneid, arrPhone, phone_num,
#else
		&phoneid, arrPhone, PHONE_NUM, 
#endif
		sizeof( uint16 ), (CompFuncType) CompUint16 );
	if ( ! pinx )
		return 0;

#ifndef USE_BINARY_DATA
	fseek( dictfile, begin[ pinx - arrPhone ], SEEK_SET );
#else
	cur_pos = (unsigned char*)dict + begin[ pinx - arrPhone ];
#endif
	end_pos = begin[ pinx - arrPhone + 1 ];
	Str2Word( wrd_ptr );
	return 1;
}

int GetCharNext( Word *wrd_ptr )
{
#ifndef USE_BINARY_DATA
	if ( ftell( dictfile ) >= end_pos )
		return 0;
#else
	if ( (unsigned char*)cur_pos >= (unsigned char*)dict + end_pos )
		return 0;
#endif
	Str2Word( wrd_ptr );
	return 1;
}
