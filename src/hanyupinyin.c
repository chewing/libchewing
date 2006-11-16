/**
 * hanyupinyin.c
 *
 * Copyright (c) 2005, 2006
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* @(#)hanyupinyin.c
 */

#include <stdio.h>
#include <string.h>
#include "hanyupinyin.h"
#include "hash.h"
#include "private.h"

static char* PINYIN_TAB_NAME[] = { "pinyin.tab" };
static PinYinMethodType INPUT_METHOD = PINYIN_HANYU;
static char TAB_PATH[255];

static keymap *hanyuInitialsMap, *hanyuFinalsMap;
static int HANYU_INITIALS, HANYU_FINALS, INIT_FLAG = 0;

CHEWING_API int chewing_set_PinYinMethod(
		const PinYinMethodType methodType,
		const char *filePath )
{
	if ( methodType < 0 || methodType >= PINYIN_NONE )
		return -1; /* invaild PinYinMethodType */


	if ( methodType == PINYIN_EXTERNAL ) {
		if ( access( filePath, R_OK ) != 0 )
			return -2; /* invaild external table */
		
		INPUT_METHOD = methodType;
		strcpy( TAB_PATH, filePath );
	}
	return 0;
}

static void FreeMap()
{ 
	free( hanyuInitialsMap );
	free( hanyuFinalsMap );
}

static int compkey( const void *k1, const void *k2 )
{
	keymap *key1 = (keymap *) k1;
	keymap *key2 = (keymap *) k2;
	return strcmp( key1->pinyin, key2->pinyin );
}

static void InitMap()
{
	if ( INPUT_METHOD != PINYIN_EXTERNAL ) {
		if ( getenv( "HOME" ) ) {
			/* Use user-defined tables */
			strcpy( TAB_PATH, getenv( "HOME" ));
			strcat( TAB_PATH, CHEWING_HASH_PATH "/");
			strcat( TAB_PATH, PINYIN_TAB_NAME[INPUT_METHOD] );

			if (access(TAB_PATH, R_OK) != 0) {
				strcpy(TAB_PATH, CHEWING_DATADIR "/");
				strcat(TAB_PATH, PINYIN_TAB_NAME[PINYIN_HANYU]);
			}
		}
		else {
				strcpy(TAB_PATH, CHEWING_DATADIR "/");
				strcat(TAB_PATH, PINYIN_TAB_NAME[PINYIN_HANYU]);
		}			
	}
	
	int i;
	FILE *fd = fopen(TAB_PATH, "r");
	
	if ( fd ) {
		addTerminateService( FreeMap );
		fscanf( fd, "%d", &HANYU_INITIALS );
		++HANYU_INITIALS;
		hanyuInitialsMap = ALC( keymap, HANYU_INITIALS );
		for ( i = 0; i < HANYU_INITIALS - 1; i++ ) {
			fscanf( fd, "%s %s",
				hanyuInitialsMap[ i ].pinyin,
				hanyuInitialsMap[ i ].zuin );
		}
		fscanf( fd, "%d", &HANYU_FINALS );
		++HANYU_FINALS;
		hanyuFinalsMap = ALC( keymap, HANYU_FINALS );
		for ( i = 0; i < HANYU_FINALS - 1; i++ ) {
			fscanf( fd, "%s %s",
				hanyuFinalsMap[ i ].pinyin,
				hanyuFinalsMap[ i ].zuin );
		}
		fclose( fd );
		INIT_FLAG = 1;
	}
}

/**
 * Map pinyin key-sequence to Zuin key-sequence.
 * Caller should allocate char zuin[4].
 * 
 * Non-Zero: Fail to fully convert
 * 
 * @retval 0 Success
 */
int HanyuPinYinToZuin( char *pinyinKeySeq, char *zuinKeySeq )
{
	char *p, *cursor;
	char *initial = 0;
	char *final = 0;
	int i;

	if ( ! INIT_FLAG )
		InitMap();

	for ( i = 0; i < HANYU_INITIALS; i++ ) {
		p = strstr( pinyinKeySeq, hanyuInitialsMap[ i ].pinyin );
		if ( p == pinyinKeySeq ) {
			initial = hanyuInitialsMap[ i ].zuin;
			cursor = pinyinKeySeq +
				strlen( hanyuInitialsMap[ i ].pinyin );
			break;
		}
	}
	if ( i == HANYU_INITIALS ) {
		// No initials. might be ㄧㄨㄩ
		/* XXX: I NEED Implementation
		   if(finalsKeySeq[0] != ) {
		   }
		   */
		return 1;
	}

	if ( cursor ) {
		for ( i = 0; i < HANYU_FINALS; i++ ) {
			p = strstr( cursor, hanyuFinalsMap[ i ].pinyin );
			if ( p == cursor ) {
				final = hanyuFinalsMap[ i ].zuin;
				break;
			}
		}
		if ( i == HANYU_FINALS ){
			return 2;
		}
	}
	
	if ( ! strcmp( final, "j0" ) ) {
		if (
			! strcmp( initial, "f" ) || 
			! strcmp( initial, "r" ) ||
			! strcmp( initial, "v" ) ) {
			final = "m0";
		}
	}
	
	sprintf( zuinKeySeq, "%s%s\0", initial, final );
	return 0;
}
