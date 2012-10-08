/**
 * hanyupinyin.c
 *
 * Copyright (c) 2005, 2006, 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* @(#)hanyupinyin.c
 */

#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <stdlib.h>

#include "global-private.h"
#include "hanyupinyin-private.h"
#include "hash-private.h"
#include "private.h"

void TerminateHanyuPinyin( ChewingData *pgdata )
{ 
	free( pgdata->hanyuInitialsMap );
	free( pgdata->hanyuFinalsMap );
}

#if 0
static int compkey( const void *k1, const void *k2 )
{
	keymap *key1 = (keymap *) k1;
	keymap *key2 = (keymap *) k2;
	return strcmp( key1->pinyin, key2->pinyin );
}
#endif

int InitHanyuPinYin( ChewingData *pgdata, const char *prefix )
{
	char filename[PATH_MAX];
	int i;
	FILE *fd;

	sprintf( filename,
		"%s" PLAT_SEPARATOR "%s",
		prefix, PINYIN_TAB_NAME );

	fd = fopen(filename, "r");

	if ( ! fd )
		return 0;

	fscanf( fd, "%d", &pgdata->HANYU_INITIALS );
	++pgdata->HANYU_INITIALS;
	pgdata->hanyuInitialsMap = ALC( keymap, pgdata->HANYU_INITIALS );
	for ( i = 0; i < pgdata->HANYU_INITIALS - 1; i++ ) {
		fscanf( fd, "%s %s",
			pgdata->hanyuInitialsMap[ i ].pinyin,
			pgdata->hanyuInitialsMap[ i ].zuin );
	}

	fscanf( fd, "%d", &pgdata->HANYU_FINALS );
	++pgdata->HANYU_FINALS;
	pgdata->hanyuFinalsMap = ALC( keymap, pgdata->HANYU_FINALS );
	for ( i = 0; i < pgdata->HANYU_FINALS - 1; i++ ) {
		fscanf( fd, "%s %s",
			pgdata->hanyuFinalsMap[ i ].pinyin,
			pgdata->hanyuFinalsMap[ i ].zuin );
	}

	fclose( fd );

	return 1;
}

/**
 * Map pinyin key-sequence to Zuin key-sequence.
 * Caller should allocate char zuin[4].
 * 
 * Non-Zero: Fail to fully convert
 * 
 * @retval 0 Success
 */
int HanyuPinYinToZuin( ChewingData *pgdata, char *pinyinKeySeq, char *zuinKeySeq )
{
	char *p, *cursor = NULL;
	char *initial = 0;
	char *final = 0;
	int i;

	for ( i = 0; i < pgdata->HANYU_INITIALS; i++ ) {
		p = strstr( pinyinKeySeq, pgdata->hanyuInitialsMap[ i ].pinyin );
		if ( p == pinyinKeySeq ) {
			initial = pgdata->hanyuInitialsMap[ i ].zuin;
			cursor = pinyinKeySeq +
				strlen( pgdata->hanyuInitialsMap[ i ].pinyin );
			break;
		}
	}
	if ( i == pgdata->HANYU_INITIALS ) {
		/* No initials. might be ㄧㄨㄩ */
		/* XXX: I NEED Implementation
		   if(finalsKeySeq[0] != ) {
		   }
		   */
		return 1;
	}

	if ( cursor ) {
		for ( i = 0; i < pgdata->HANYU_FINALS; i++ ) {
			p = strstr( cursor, pgdata->hanyuFinalsMap[ i ].pinyin );
			if ( p == cursor ) {
				final = pgdata->hanyuFinalsMap[ i ].zuin;
				break;
			}
		}
		if ( i == pgdata->HANYU_FINALS ){
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
	
	sprintf( zuinKeySeq, "%s%s", initial, final );
	return 0;
}
