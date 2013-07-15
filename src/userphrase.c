/**
 * userphrase.c
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

#include <stdlib.h>
#include <string.h>

#include "chewing-utf8-util.h"
#include "hash-private.h"
#include "dict-private.h"
#include "tree-private.h"
#include "userphrase-private.h"
#include "private.h"

/* load the orginal frequency from the static dict */
static int LoadOriginalFreq( ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[], int len )
{
	int pho_id;
	int retval;
	Phrase *phrase = ALC( Phrase, 1 );

	pho_id = TreeFindPhrase( pgdata, 0, len - 1, phoneSeq );
	if ( pho_id != -1 ) {
		GetPhraseFirst( pgdata, phrase, pho_id );
		do {
			/* find the same phrase */
			if ( ! strcmp(
				phrase->phrase,
				wordSeq ) ) {
				retval = phrase->freq;
				free( phrase );
				return retval;
			}
		} while ( GetPhraseNext( pgdata, phrase ) );
	}

	free( phrase );
	return FREQ_INIT_VALUE;
}

/* find the maximum frequency of the same phrase */
static int LoadMaxFreq( ChewingData *pgdata, const uint16_t phoneSeq[], int len )
{
	int pho_id;
	Phrase *phrase = ALC( Phrase, 1 );
	int maxFreq = FREQ_INIT_VALUE;
	UserPhraseData *uphrase;

	pho_id = TreeFindPhrase( pgdata, 0, len - 1, phoneSeq );
	if ( pho_id != -1 ) {
		GetPhraseFirst( pgdata, phrase, pho_id );
		do {
			if ( phrase->freq > maxFreq )
				maxFreq = phrase->freq;
		} while( GetPhraseNext( pgdata, phrase ) );
	}
	free( phrase );

	uphrase = UserGetPhraseFirst( pgdata, phoneSeq );
	while ( uphrase ) {
		if ( uphrase->userfreq > maxFreq )
			maxFreq = uphrase->userfreq;
		uphrase = UserGetPhraseNext( pgdata, phoneSeq );
	}

	return maxFreq;
}

/* compute the new updated freqency */
static int UpdateFreq( int freq, int maxfreq, int origfreq, int deltatime )
{
	int delta;

	/* Short interval */
	if ( deltatime < 4000 ) {
		delta = ( freq >= maxfreq ) ?
			min(
				( maxfreq - origfreq ) / 5 + 1,
				SHORT_INCREASE_FREQ ) :
			max(
				( maxfreq - origfreq ) / 5 + 1,
				SHORT_INCREASE_FREQ );
		return min( freq + delta, MAX_ALLOW_FREQ );
	}
	/* Medium interval */
	else if ( deltatime < 50000 ) {
		delta = ( freq >= maxfreq ) ?
			min(
				( maxfreq - origfreq ) / 10 + 1,
				MEDIUM_INCREASE_FREQ ) :
			max(
				( maxfreq - origfreq ) / 10 + 1,
				MEDIUM_INCREASE_FREQ );
		return min( freq + delta, MAX_ALLOW_FREQ );
	}
	/* long interval */
	else {
		delta = max( ( freq - origfreq ) / 5, LONG_DECREASE_FREQ );
		return max( freq - delta, origfreq );
	}
}

int UserUpdatePhrase( ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[] )
{
	HASH_ITEM *pItem;
	UserPhraseData data;
	int len;

	len = ueStrLen( wordSeq );
	pItem = HashFindEntry( pgdata, phoneSeq, wordSeq );
	if ( ! pItem ) {
		if ( ! AlcUserPhraseSeq( &data, len, strlen( wordSeq ) ) ) {
			return USER_UPDATE_FAIL;
		}

		memcpy( data.phoneSeq, phoneSeq, len * sizeof( phoneSeq[ 0 ] ) );
		data.phoneSeq[ len ] = 0;
		strcpy( data.wordSeq, wordSeq );

		/* load initial freq */
		data.origfreq = LoadOriginalFreq( pgdata, phoneSeq, wordSeq, len );
		data.maxfreq = LoadMaxFreq( pgdata, phoneSeq, len );

		data.userfreq = data.origfreq;
		data.recentTime = pgdata->static_data.chewing_lifetime;
		pItem = HashInsert( pgdata, &data );
		HashModify( pgdata, pItem );
		return USER_UPDATE_INSERT;
	}
	else {
		pItem->data.maxfreq = LoadMaxFreq( pgdata, phoneSeq, len );
		pItem->data.userfreq = UpdateFreq(
			pItem->data.userfreq,
			pItem->data.maxfreq,
			pItem->data.origfreq,
			pgdata->static_data.chewing_lifetime - pItem->data.recentTime );
		pItem->data.recentTime = pgdata->static_data.chewing_lifetime;
		HashModify( pgdata, pItem );
		return USER_UPDATE_MODIFY;
	}
}

UserPhraseData *UserGetPhraseFirst( ChewingData *pgdata, const uint16_t phoneSeq[] )
{
	pgdata->prev_userphrase = HashFindPhonePhrase( pgdata, phoneSeq, NULL );
	if ( ! pgdata->prev_userphrase )
		return NULL;
	return &( pgdata->prev_userphrase->data );
}

UserPhraseData *UserGetPhraseNext( ChewingData *pgdata, const uint16_t phoneSeq[] )
{
	pgdata->prev_userphrase = HashFindPhonePhrase( pgdata, phoneSeq, pgdata->prev_userphrase );
	if ( ! pgdata->prev_userphrase )
		return NULL;
	return &( pgdata->prev_userphrase->data );
}

