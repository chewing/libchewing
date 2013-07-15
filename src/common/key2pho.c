/**
 * key2pho.c
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

/**
 * @file key2pho.c
 * @brief map zuins to uint16_t type according to different kb_type
 */

/* This file is encoded in UTF-8 */

#include <stdio.h>
#include <string.h>
#include "chewing-utf8-util.h"
#include "chewing-private.h"

/* NOTE:
 * The reason why we convert string literal to hex representation is for the
 * sake of portability, that avoid some buggy or faulty environment like
 * Microsoft VC9 to misinterpret the string.
 */
const char *const zhuin_tab[] = {				/* number of bits */
	  "  \xE3\x84\x85\xE3\x84\x86\xE3\x84\x87\xE3\x84\x88\xE3\x84\x89"
	  "\xE3\x84\x8A\xE3\x84\x8B\xE3\x84\x8C\xE3\x84\x8D\xE3\x84\x8E"
	  "\xE3\x84\x8F\xE3\x84\x90\xE3\x84\x91\xE3\x84\x92\xE3\x84\x93"
	  "\xE3\x84\x94\xE3\x84\x95\xE3\x84\x96\xE3\x84\x97\xE3\x84\x98"
	  "\xE3\x84\x99",						/* 5 */
	  	/* ㄅㄆㄇㄈㄉㄊㄋㄌㄍㄎㄏㄐㄑㄒㄓㄔㄕㄖㄗㄘㄙ */
	  "  \xE3\x84\xA7\xE3\x84\xA8\xE3\x84\xA9",			/* 2 */
	  	/* ㄧㄨㄩ */
	  "  \xE3\x84\x9A\xE3\x84\x9B\xE3\x84\x9C\xE3\x84\x9D\xE3\x84\x9E"
	  "\xE3\x84\x9F\xE3\x84\xA0\xE3\x84\xA1\xE3\x84\xA2\xE3\x84\xA3"
	  "\xE3\x84\xA4\xE3\x84\xA5\xE3\x84\xA6",			/* 4 */
	  	/* ㄚㄛㄜㄝㄞㄟㄠㄡㄢㄣㄤㄥㄦ */
	  "  \xCB\x99\xCB\x8A\xCB\x87\xCB\x8B"				/* 3 */
		/* ˙ˊˇˋ */
};

static const int zhuin_tab_num[] = { 22, 4, 14, 5 };
static const int shift[] = { 9, 7, 3, 0 };
static const int sb[] = { 31, 3, 15, 7 };

static const char * const ph_str =
	"\xE3\x84\x85\xE3\x84\x86\xE3\x84\x87\xE3\x84\x88"
		/* ㄅㄆㄇㄈ */
	"\xE3\x84\x89\xE3\x84\x8A\xE3\x84\x8B\xE3\x84\x8C"
		/* ㄉㄊㄋㄌ */
	"\xE3\x84\x8D\xE3\x84\x8E\xE3\x84\x8F\xE3\x84\x90"
		/* ㄍㄎㄏㄐ */
	"\xE3\x84\x91\xE3\x84\x92\xE3\x84\x93\xE3\x84\x94"
		/* ㄑㄒㄓㄔ */
	"\xE3\x84\x95\xE3\x84\x96\xE3\x84\x97\xE3\x84\x98"
		/* ㄕㄖㄗㄘ */
	"\xE3\x84\x99\xE3\x84\xA7\xE3\x84\xA8\xE3\x84\xA9"
		/* ㄙㄧㄨㄩ */
	"\xE3\x84\x9A\xE3\x84\x9B\xE3\x84\x9C\xE3\x84\x9D"
		/* ㄚㄛㄜㄝ */
	"\xE3\x84\x9E\xE3\x84\x9F\xE3\x84\xA0\xE3\x84\xA1"
		/* ㄞㄟㄠㄡ */
	"\xE3\x84\xA2\xE3\x84\xA3\xE3\x84\xA4\xE3\x84\xA5\xE3\x84\xA6"
		/* ㄢㄣㄤㄥㄦ */
	"\xCB\x99\xCB\x8A\xCB\x87\xCB\x8B" ;
		/* ˙ˊˇˋ */
static const char * const key_str[ MAX_KBTYPE ] = {
	 "1qaz2wsxedcrfv5tgbyhnujm8ik,9ol.0p;/-7634",		/* standard kb */
	 "bpmfdtnlgkhjvcjvcrzasexuyhgeiawomnkllsdfj",		/* hsu */
	 "1234567890-qwertyuiopasdfghjkl;zxcvbn/m,.",		/* IBM */
	 "2wsx3edcrfvtgb6yhnujm8ik,9ol.0p;/-['=1qaz",		/* Gin-yieh */
	 "bpmfdtnlvkhg7c,./j;'sexuaorwiqzy890-=1234",		/* ET  */
	 "bpmfdtnlvkhgvcgycjqwsexuaorwiqzpmntlhdfjk",		/* ET26 */
	 "1'a;2,oq.ejpuk5yixfdbghm8ctw9rnv0lsz[7634",		/* Dvorak */
	 "bpmfdtnlgkhjvcjvcrzasexuyhgeiawomnkllsdfj",		/* Dvorak Hsu */
	 "qqazwwsxedcrfvttgbyhnujmuikbiolmoplnpyerd",		/* DACHEN-CP26 */
	 "1qaz2wsxedcrfv5tgbyhnujm8ik,9ol.0p;/-7634",           /* Hanyu Pinyin */
	 "1qaz2wsxedcrfv5tgbyhnujm8ik,9ol.0p;/-7634",           /* Luoma Pinyin */
	 "1qaz2wsxedcrfv5tgbyhnujm8ik,9ol.0p;/-7634",           /* secondary Bopomofo Pinyin */

} ;

/*
 * Read one zhuin string,
 *
 * return the number it means. 0 means error.
 */
uint16_t UintFromPhone( const char *zhuin )
{
	const char *iter;
	char *pos;
	char buf[ MAX_UTF8_SIZE + 1 ];
	int len, result = 0;
	int zhuin_index = 0;

	iter = zhuin;

	while ( *iter ) {
		len = ueStrNCpy( buf, iter, 1, STRNCPY_CLOSE );

		for (; zhuin_index < ZUIN_SIZE; ++zhuin_index ) {
			pos = strstr( zhuin_tab[ zhuin_index ], buf );
			if ( pos ) {
				break;
			}
		}

		if ( zhuin_index >= ZUIN_SIZE ) {
			return 0;
		}

		result |= (zhuin_tab_num[ zhuin_index ] - ueStrLen( pos )) << shift[ zhuin_index ];
		++zhuin_index;
		iter += len;
	}
	return result;
}

int PhoneFromKey( char *pho, const char *inputkey, int kbtype, int searchTimes )
{
	int len = strlen( inputkey ), i, s;
	const char *pTarget;

	pho[ 0 ] = '\0';
	for ( i = 0; i < len; i++ ) {
		char *findptr = NULL;
		int _index;

		for (
				s = 0, pTarget = key_str[ kbtype ];
				s < searchTimes;
				s++, pTarget = findptr + 1 ) {
			findptr = strchr( pTarget, inputkey[ i ] );
			if ( ! findptr ) {
				return 0;
			}
		}
		_index = findptr - key_str[ kbtype ];
		ueStrNCpy( ueStrSeek( pho, i ),
		           ueConstStrSeek( ph_str, _index ),
			   1, 0);
	}
	pho = ueStrSeek( pho, len );
	pho[0] = '\0';
	return 1;
}

int PhoneFromUint( char *phone, size_t phone_len, uint16_t phone_num )
{
	int i;
	int index;
	const char *pos;
	char tmp[ MAX_UTF8_SIZE + 1 ];
	char buffer[ MAX_UTF8_SIZE * ZUIN_SIZE + 1 ] = { 0 };

	for ( i = 0; i < ZUIN_SIZE; ++i ) {
		// The first two characters in zhuin_tab are space, so we need
		// to add 1 here.
		index = ((phone_num >> shift[ i ]) & sb[ i ]) + 1;
		if ( index >= 2 ) {
			pos = ueConstStrSeek( zhuin_tab[ i ], index );
			ueStrNCpy( tmp, pos, 1, 1 );
			strcat( buffer, tmp );
		}
	}
	strncpy( phone, buffer, phone_len );
	return 0;
}

int PhoneInxFromKey( int key, int type, int kbtype, int searchTimes )
{
	char keyStr[ 2 ], rtStr[ 10 ], *p;

	keyStr[ 0 ] = key;
	keyStr[ 1 ] = '\0';
	if ( ! PhoneFromKey( rtStr, keyStr, kbtype, searchTimes ) )
		return 0;
	p = strstr( zhuin_tab[ type ], rtStr );
	if ( ! p )
		return 0;
	return zhuin_tab_num[type] - ueStrLen(p);
}

uint16_t UintFromPhoneInx( const int ph_inx[] )
{
	int i;
	uint16_t result = 0;

	for ( i = 0; i < ZUIN_SIZE; i++ )
		result |= ph_inx[ i ] << shift[ i ];
	return result;
}

