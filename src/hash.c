/**
 * hash.c
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004, 2005, 2006, 2007, 2008, 2011
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include <string.h>
#include <sys/stat.h>
/* ISO C99 Standard: 7.10/5.2.4.2.1 Sizes of integer types */
#include <limits.h>
#include <assert.h>
#include <stdlib.h>
#include <stdio.h>

#include "chewing-utf8-util.h"
#include "hash-private.h"
#include "private.h"
#include "global.h"

int AlcUserPhraseSeq( UserPhraseData *pData, int phonelen, int wordlen )
{
	pData->phoneSeq = ALC( uint16, phonelen + 1 );
	pData->wordSeq = ALC( char, wordlen + 1 );
	return ( pData->phoneSeq && pData->wordSeq );
}

static int PhoneSeqTheSame( const uint16 p1[], const uint16 p2[] )
{
	int i;
	if ( ! p1 || ! p2 )	/* FIXME: should not happend. */
		return 0;

	for ( i = 0; ( p1[ i ] != 0 && p2[ i ] != 0 ); i++ ) {
		if ( p1[ i ] != p2[ i ] )
			return 0;
	}
	if ( p1[ i ] != p2[ i ] )
		return 0;
	return 1;
}

static unsigned int HashFunc( const uint16 phoneSeq[] )
{
	int i, value = 0;

	for ( i = 0; phoneSeq[ i ] != 0; i++ )
		value ^= phoneSeq[ i ];
	return ( value & ( HASH_TABLE_SIZE - 1 ) );
}

HASH_ITEM *HashFindPhonePhrase( ChewingData *pgdata, const uint16 phoneSeq[], HASH_ITEM *pItemLast )
{
	HASH_ITEM *pNow = pItemLast ?
			pItemLast->next :
			pgdata->hashtable[ HashFunc( phoneSeq ) ];
	
	for ( ; pNow; pNow = pNow->next ) 
		if ( PhoneSeqTheSame( pNow->data.phoneSeq, phoneSeq ) )
			return pNow;
	return NULL;
}

HASH_ITEM *HashFindEntry( ChewingData *pgdata, const uint16 phoneSeq[], const char wordSeq[] )
{
	HASH_ITEM *pItem;
	int hashvalue;

	hashvalue = HashFunc( phoneSeq );

	for ( pItem = pgdata->hashtable[ hashvalue ]; pItem ; pItem = pItem->next ) {
		if ( 
			! strcmp( pItem->data.wordSeq, wordSeq ) && 
			PhoneSeqTheSame( pItem->data.phoneSeq, phoneSeq ) ) {
			return pItem;
		}
	}
	return NULL;
}

HASH_ITEM *HashInsert( ChewingData *pgdata, UserPhraseData *pData )
{
	int hashvalue, len;
	HASH_ITEM *pItem;

	pItem = HashFindEntry( pgdata, pData->phoneSeq, pData->wordSeq );
	if ( pItem != NULL )
		return pItem;

	pItem = ALC( HASH_ITEM, 1 );
	if ( ! pItem )
		return NULL;  /* Error occurs */
	len = ueStrLen( pData->wordSeq );
	if ( ! AlcUserPhraseSeq( &( pItem->data ), len, strlen( pData->wordSeq ) ) )
		return NULL; /* Error occurs */

	hashvalue = HashFunc( pData->phoneSeq );
	/* set the new element */
	pItem->next = pgdata->hashtable[ hashvalue ];

	memcpy( &( pItem->data ), pData, sizeof( pItem->data ) );
	pItem->item_index = -1;

	/* set link to the new element */
	pgdata->hashtable[ hashvalue ] = pItem;

	return pItem;
}

#ifdef ENABLE_DEBUG
static void HashItem2String( char *str, HASH_ITEM *pItem )
{
	int i, len;
	char buf[ FIELD_SIZE ];

	sprintf( str, "%s ", pItem->data.wordSeq );
	len = ueStrLen( pItem->data.wordSeq );
	for ( i = 0; i < len; i++ ) {
		sprintf( buf, "%hu ", pItem->data.phoneSeq[ i ] );
		strcat( str, buf );
	}
	sprintf(
		buf, "%d %d %d %d",
		pItem->data.userfreq, pItem->data.recentTime,
		pItem->data.maxfreq, pItem->data.origfreq );
	strcat( str, buf );
}
#endif

/* 
 * capacity of 'str' MUST bigger then FIELD_SIZE !
 */
void HashItem2Binary( char *str, HASH_ITEM *pItem )
{
	int i, phraselen;
	uint16 *pshort;
	unsigned char *puc;

	memset( str, 0, FIELD_SIZE );
	if ( sizeof(int) * 4 + ueStrLen( pItem->data.wordSeq ) * 2 +
	     strlen( pItem->data.wordSeq ) >= FIELD_SIZE ) {
		/* exceed buffer size */
		return;
	}

	/* freq info */
	*(int*) &str[ 0 ] = pItem->data.userfreq;
	*(int*) &str[ 4 ] = pItem->data.recentTime;
	*(int*) &str[ 8 ] = pItem->data.maxfreq;
	*(int*) &str[ 12 ] = pItem->data.origfreq;

	/* phone seq*/
	phraselen = ueStrLen( pItem->data.wordSeq );
	str[ 16 ] = phraselen;
	pshort = (uint16 *) &str[ 17 ];
	for ( i = 0; i < phraselen; i++ ) {
		*pshort = pItem->data.phoneSeq[ i ];
		pshort++;
	}

	/* phrase */
	puc = (unsigned char *) pshort;
	*puc = strlen( pItem->data.wordSeq );
	strcpy( (char *) (puc + 1), pItem->data.wordSeq );
	pItem->data.wordSeq[ (int) *puc ] = '\0';
}

void HashModify( ChewingData *pgdata, HASH_ITEM *pItem )
{
	FILE *outfile;
	char str[ FIELD_SIZE + 1 ];

	outfile = fopen( pgdata->hashfilename, "r+b" );

	/* update "lifetime" */
	fseek( outfile, strlen( BIN_HASH_SIG ), SEEK_SET );
	fwrite( &pgdata->chewing_lifetime, 1, 4, outfile );
#ifdef ENABLE_DEBUG
	sprintf( str, "%d", pgdata->chewing_lifetime );
	DEBUG_OUT( "HashModify-1: '%-75s'\n", str );
	DEBUG_FLUSH;
#endif

	/* update record */
	if ( pItem->item_index < 0 ) {
		fseek( outfile, 0, SEEK_END );
		pItem->item_index =
			( ftell( outfile ) - 4 - strlen( BIN_HASH_SIG ) ) / FIELD_SIZE;
	}
	else {
		fseek( outfile,
			pItem->item_index * FIELD_SIZE + 4 + strlen( BIN_HASH_SIG ),
			SEEK_SET );
	}
#ifdef ENABLE_DEBUG
	HashItem2String( str, pItem );
	DEBUG_OUT( "HashModify-2: '%-75s'\n", str );
	DEBUG_FLUSH;
#endif
	HashItem2Binary( str, pItem );
	fwrite( str, 1, FIELD_SIZE, outfile );
	fflush( outfile );
	fclose( outfile );
}

static int isValidChineseString( char *str )
{
	if ( str == NULL || *str == '\0' ) {
		return 0;
	}
	while ( *str != '\0' )  {
		int len = ueBytesFromChar( (unsigned char) *str );
		if ( len <= 1 ) {
			return 0;
		}
		str += len;
	};
	return 1;
}

static int ReadInt(unsigned char *addr)
{
	/* TODO: Use bit-wise operation to read */
	int *p = (void*)addr;
	return *p;
}

/**
 * @return 1, 0 or -1
 * retval 0	end of file
 * retval 1	continue
 * retval -1	ignore this record
 */
int ReadHashItem_bin( const char *srcbuf, HASH_ITEM *pItem, int item_index )
{
	int len, i;
	uint16 *pshort;
	unsigned char recbuf[ FIELD_SIZE ], *puc;

	memcpy( recbuf, srcbuf, FIELD_SIZE );
	memset( pItem, 0, sizeof(HASH_ITEM) );

	/* freq info */
	pItem->data.userfreq	= ReadInt(&recbuf[ 0 ]);
	pItem->data.recentTime	= ReadInt(&recbuf[ 4 ]);
	pItem->data.maxfreq	= ReadInt(&recbuf[ 8 ]);
	pItem->data.origfreq	= ReadInt(&recbuf[ 12 ]);

	/* phone seq, length in num of chi words */
	len = (int) recbuf[ 16 ];
	pItem->data.phoneSeq = ALC( uint16, len + 1 );
	pshort = (uint16 *) &recbuf[ 17 ];
	for ( i = 0; i < len; i++ ) {
		pItem->data.phoneSeq[ i ] = *pshort;
		++pshort;
	}
	pItem->data.phoneSeq[ i ] = 0;

	/* phrase, length in num of bytes */
	puc = (unsigned char *) pshort;
	pItem->data.wordSeq = ALC( char, (*puc) + 1 );
	strcpy( pItem->data.wordSeq, (char *) (puc + 1) );
	pItem->data.wordSeq[ (int) *puc ] = '\0';

	/* Invalid UTF-8 Chinese characters found */
	if ( ! isValidChineseString( pItem->data.wordSeq ) ) {
		goto ignore_corrupted_record;
	}

	/* set item_index */
	pItem->item_index = item_index;

	return 1; /* continue */

ignore_corrupted_record:
	if ( pItem->data.phoneSeq != NULL ) {
		free( pItem->data.phoneSeq );
		pItem->data.phoneSeq = NULL;
	}
	if ( pItem->data.wordSeq != NULL ) {
		free( pItem->data.wordSeq );
		pItem->data.wordSeq = NULL;
	}
	return -1; /* ignore */
}

/**
 * @return 1, 0 or -1
 * retval -1 Ignore bad data item
 */
int ReadHashItem_txt( FILE *infile, HASH_ITEM *pItem, int item_index )
{
	int len, i, word_len;
	char wordbuf[ 64 ];

	/* read wordSeq */
	if ( fscanf( infile, "%s", wordbuf ) != 1 )
		return 0;

	/* Invalid UTF-8 Chinese characters found */
	if ( ! isValidChineseString( wordbuf ) ) {
		fseek( infile, FIELD_SIZE - strlen( wordbuf ) - 1, SEEK_CUR );
		return -1;
	}

	word_len = strlen( wordbuf );
	pItem->data.wordSeq = ALC( char, word_len + 1 );
	strcpy( pItem->data.wordSeq, wordbuf );

	/* read phoneSeq */
	len = ueStrLen( pItem->data.wordSeq );
	pItem->data.phoneSeq = ALC( uint16, len + 1 );
	for ( i = 0; i < len; i++ )
		if ( fscanf( infile, "%hu", &( pItem->data.phoneSeq[ i ] ) ) != 1 )
			return 0;
	pItem->data.phoneSeq[ len ] = 0;

	/* read userfreq & recentTime */
	if ( fscanf( infile, "%d %d %d %d",
	             &(pItem->data.userfreq),
	             &(pItem->data.recentTime),
	             &(pItem->data.maxfreq),
	             &(pItem->data.origfreq) ) != 4 )
		return 0;

	/* set item_index */
	pItem->item_index = item_index;

	return 1;
}

static FILE *open_file_get_length(
		const char *filename, 
		const char *otype, int *size)
{
	FILE *tf = fopen( filename, otype );
	if ( tf == NULL ) {
		return NULL;
	}
	if ( size != NULL ) {
		fseek( tf, 0, SEEK_END );
		*size = ftell( tf );
		fseek( tf, 0, SEEK_SET );
	}
	return tf;
}

char *_load_hash_file( const char *filename, int *size )
{
	int flen;
	char *pd = NULL;
	FILE *tf;

	tf = open_file_get_length( filename, "rb", &flen );
	if ( tf == NULL ) {
		goto err_load_file;
	}
	pd = (char *) malloc( flen );
	if ( pd == NULL ) {
		goto err_load_file;
	}
	if ( fread( pd, flen, 1, tf ) != 1 ) {
		goto err_load_file;
	}
	fclose( tf );
	if ( size != NULL )
		*size = flen;
	return pd;

err_load_file:
	if ( pd != NULL )
		free( pd );
	if ( tf != NULL )
		fclose( tf );
	return NULL;
}

// FIXME: Remove ofliename
static int migrate_hash_to_bin( ChewingData *pgdata, const char *ofilename )
{
	FILE *txtfile;
	char oldname[ 256 ], *dump, *seekdump;
	HASH_ITEM item;
	int item_index, iret, tflen;
	int ret;

	/* allocate dump buffer */
	txtfile = open_file_get_length( ofilename, "r", &tflen );
	if ( txtfile == NULL ) {
		return 0;
	}
	dump = (char *) malloc( tflen * 2 );
	if ( dump == NULL ) {
		fclose( txtfile );
		return 0;
	}
	ret = fscanf( txtfile, "%d", &pgdata->chewing_lifetime );
	if ( ret != 1 ) {
		return 0;
	}

	/* prepare the bin file */
	seekdump = dump;
	memcpy( seekdump, BIN_HASH_SIG, strlen( BIN_HASH_SIG ) );
	memcpy( seekdump + strlen( BIN_HASH_SIG ),
	        &pgdata->chewing_lifetime,
		sizeof(pgdata->chewing_lifetime) );
	seekdump += strlen( BIN_HASH_SIG ) + sizeof(pgdata->chewing_lifetime);

	/* migrate */
	item_index = 0;
	while ( 1 ) {
		iret = ReadHashItem_txt( txtfile, &item, ++item_index );

		if ( iret == -1 ) {
			--item_index;
			continue;
		}
		else if ( iret==0 )
			break;

		HashItem2Binary( seekdump, &item );
		seekdump += FIELD_SIZE;
		free( item.data.phoneSeq );
		free( item.data.wordSeq );
	};
	fclose( txtfile );

	/* backup as *.old */
	strncpy( oldname, ofilename, sizeof(oldname) );
	strncat( oldname, ".old", sizeof(oldname) );
	oldname[ sizeof(oldname) - 1 ] = '\0';
	PLAT_UNLINK( oldname );
	PLAT_RENAME( ofilename, oldname );

	/* dump new file */
	PLAT_UNLINK( ofilename );
	txtfile = fopen( ofilename, "w+b" );
	fwrite( dump, seekdump - dump, 1, txtfile );
	fflush( txtfile );
	fclose( txtfile );
	free( dump );

	return  1;
}

#if 0
/**
 * Attempt to re-compute lifetime
 */
static int ComputeChewingLifeTime()
{
       HASH_ITEM *item;
       int i, min;
       
       i = 0;

       chewing_lifetime++;
       min = chewing_lifetime;

       while ( hashtable[ i ] ) {
               item = hashtable[ i ];
               while ( item ) {
                       if ( item->data.recentTime < min )
                               min = item->data.recentTime;
                       item = item->next;
               }
               i++;
       }

       chewing_lifetime -= min;
       i = 0;

       while ( hashtable[ i ] ) {
               item = hashtable[ i ];
               while ( item ) {
                       item->data.recentTime -= min;
                       HashModify( item );
                       item = item->next;
               }
               i++;
       }
       return 0;
}
#endif

static void FreeHashItem( HASH_ITEM *aItem )
{
	if ( aItem ) {
		HASH_ITEM *pItem = aItem->next;
		free( aItem->data.phoneSeq );
		free( aItem->data.wordSeq );
		free( aItem );
		if ( pItem ) {
			FreeHashItem( pItem );
		}
	}
}

void TerminateHash( ChewingData *pgdata )
{
	HASH_ITEM *pItem;
	int i;
	for ( i = 0; i < HASH_TABLE_SIZE; ++i ) {
		pItem = pgdata->hashtable[ i ];
		DEBUG_CHECKPOINT();
		FreeHashItem( pItem );
	}
}

int InitHash( ChewingData *pgdata )
{
	HASH_ITEM item, *pItem, *pPool = NULL;
	int item_index, hashvalue, iret, fsize, hdrlen, oldest = INT_MAX;
	char *dump, *seekdump;

	const char *path = getenv( "CHEWING_PATH" );

	/* make sure of write permission */
	if ( path && access( path, W_OK ) == 0 ) {
		sprintf( pgdata->hashfilename, "%s" PLAT_SEPARATOR "%s", path, HASH_FILE );
	} else {
		if ( getenv( "HOME" ) ) {
			sprintf(
				pgdata->hashfilename, "%s%s",
				getenv( "HOME" ), CHEWING_HASH_PATH );
		}
		else {
			sprintf(
				pgdata->hashfilename, "%s%s",
				PLAT_TMPDIR, CHEWING_HASH_PATH );
		}
		PLAT_MKDIR( pgdata->hashfilename );
		strcat( pgdata->hashfilename, PLAT_SEPARATOR );
		strcat( pgdata->hashfilename, HASH_FILE );
	}
	memset( pgdata->hashtable, 0, sizeof( pgdata->hashtable ) );

open_hash_file:
	dump = _load_hash_file( pgdata->hashfilename, &fsize );
	hdrlen = strlen( BIN_HASH_SIG ) + sizeof(pgdata->chewing_lifetime);
	item_index = 0;
	if ( dump == NULL || fsize < hdrlen ) {
		FILE *outfile;
		outfile = fopen( pgdata->hashfilename, "w+b" );
		if ( ! outfile ) {
			if ( dump ) {
				free( dump );
			}
			return 0;
		}
		pgdata->chewing_lifetime = 0;
		fwrite( BIN_HASH_SIG, 1, strlen( BIN_HASH_SIG ), outfile );
		fwrite( &pgdata->chewing_lifetime, 1,
		                sizeof(pgdata->chewing_lifetime), outfile );
		fclose( outfile );
	}
	else {
		if ( memcmp(dump, BIN_HASH_SIG, strlen(BIN_HASH_SIG)) != 0 ) {
			/* perform migrate from text-based to binary form */
			free( dump );
			if ( ! migrate_hash_to_bin( pgdata, pgdata->hashfilename ) ) {
				return  0;
			}
			goto open_hash_file;
		}

		pgdata->chewing_lifetime = *(int *) (dump + strlen( BIN_HASH_SIG ));
		seekdump = dump + hdrlen;
		fsize -= hdrlen;

		while ( fsize >= FIELD_SIZE ) {
			iret = ReadHashItem_bin( seekdump, &item, item_index++ );
			/* Ignore illegal data */
			if ( iret == -1 ) {
				seekdump += FIELD_SIZE;
				fsize -= FIELD_SIZE;
				--item_index;
				continue;
			}
			else if ( iret == 0 )
				break;

			pItem = ALC( HASH_ITEM, 1 );
			memcpy( pItem, &item, sizeof( HASH_ITEM ) );
			pItem->next = pPool;
			pPool = pItem;

			if ( oldest > pItem->data.recentTime ) {
				oldest = pItem->data.recentTime;
			}

			seekdump += FIELD_SIZE;
			fsize -= FIELD_SIZE;
		}
		free( dump );

		while ( pPool ) {
			pItem = pPool;
			pPool = pItem->next;

			hashvalue = HashFunc( pItem->data.phoneSeq );
			pItem->next = pgdata->hashtable[ hashvalue ];
			pgdata->hashtable[ hashvalue ] = pItem;
			pItem->data.recentTime -= oldest;
		}
		pgdata->chewing_lifetime -= oldest;
	}
	return 1;
}

