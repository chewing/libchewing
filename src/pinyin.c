/**
 * pinyin.c
 *
 * Copyright (c) 2005, 2006, 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* @(#)pinyin.c
 */

#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <stdlib.h>

#include "global-private.h"
#include "pinyin-private.h"
#include "hash-private.h"
#include "private.h"

void TerminateHanyuPinyin( ChewingData *pgdata )
{ 
	free( pgdata->static_data.hanyuInitialsMap );
	free( pgdata->static_data.hanyuFinalsMap );
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
	int ret;

	sprintf( filename,
		"%s" PLAT_SEPARATOR "%s",
		prefix, PINYIN_TAB_NAME );

	fd = fopen(filename, "r");

	if ( ! fd )
		return 0;

	ret = fscanf( fd, "%d", &pgdata->static_data.HANYU_INITIALS );
	if ( ret != 1 ) {
		return 0;
	}
	++pgdata->static_data.HANYU_INITIALS;
	pgdata->static_data.hanyuInitialsMap = ALC( keymap, pgdata->static_data.HANYU_INITIALS );
	for ( i = 0; i < pgdata->static_data.HANYU_INITIALS - 1; i++ ) {
		ret = fscanf( fd, "%s %s",
			pgdata->static_data.hanyuInitialsMap[ i ].pinyin,
			pgdata->static_data.hanyuInitialsMap[ i ].zuin );
		if ( ret != 2 ) {
			return 0;
		}
	}

	ret = fscanf( fd, "%d", &pgdata->static_data.HANYU_FINALS );
	if ( ret != 1 ) {
		return 0;
	}
	++pgdata->static_data.HANYU_FINALS;
	pgdata->static_data.hanyuFinalsMap = ALC( keymap, pgdata->static_data.HANYU_FINALS );
	for ( i = 0; i < pgdata->static_data.HANYU_FINALS - 1; i++ ) {
		ret = fscanf( fd, "%s %s",
			pgdata->static_data.hanyuFinalsMap[ i ].pinyin,
			pgdata->static_data.hanyuFinalsMap[ i ].zuin );
		if ( ret != 2 ) {
			return 0;
		}
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

	for ( i = 0; i < pgdata->static_data.HANYU_INITIALS; i++ ) {
		p = strstr( pinyinKeySeq, pgdata->static_data.hanyuInitialsMap[ i ].pinyin );
		if ( p == pinyinKeySeq ) {
			initial = pgdata->static_data.hanyuInitialsMap[ i ].zuin;
			cursor = pinyinKeySeq +
				strlen( pgdata->static_data.hanyuInitialsMap[ i ].pinyin );
			break;
		}
	}
	if ( i == pgdata->static_data.HANYU_INITIALS ) {
		/* No initials. might be ㄧㄨㄩ */
		/* XXX: I NEED Implementation
		   if(finalsKeySeq[0] != ) {
		   }
		   */
		return 1;
	}

	if ( cursor ) {
		for ( i = 0; i < pgdata->static_data.HANYU_FINALS; i++ ) {
			p = strstr( cursor, pgdata->static_data.hanyuFinalsMap[ i ].pinyin );
			if ( p == cursor ) {
				final = pgdata->static_data.hanyuFinalsMap[ i ].zuin;
				break;
			}
		}
		if ( i == pgdata->static_data.HANYU_FINALS ){
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
