/**
 * hash.c
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004, 2005, 2006, 2007, 2008, 2011, 2012, 2013
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include <string.h>
#include <sys/stat.h>
/* ISO C99 Standard: 7.10/5.2.4.2.1 Sizes of integer types */
#include <limits.h>
#include <stdlib.h>
#include <stdio.h>

#include "chewing-private.h"
#include "chewing-utf8-util.h"
#include "hash-private.h"
#include "private.h"
#include "memory-private.h"

int AlcUserPhraseSeq( UserPhraseData *pData, int phonelen, int wordlen )
{
	pData->phoneSeq = ALC( uint16_t, phonelen + 1 );
	if ( !pData->phoneSeq )
		goto error;
	pData->wordSeq = ALC( char, wordlen + 1 );
	if ( !pData->wordSeq )
		goto error;

	return 1;

error:
	free( pData->phoneSeq );
	free( pData->wordSeq );
	return 0;
}

static int PhoneSeqTheSame( const uint16_t p1[], const uint16_t p2[] )
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

static unsigned int HashFunc( const uint16_t phoneSeq[] )
{
	int i, value = 0;

	for ( i = 0; phoneSeq[ i ] != 0; i++ )
		value ^= phoneSeq[ i ];
	return ( value & ( HASH_TABLE_SIZE - 1 ) );
}

HASH_ITEM *HashFindPhonePhrase( ChewingData *pgdata, const uint16_t phoneSeq[], HASH_ITEM *pItemLast )
{
	HASH_ITEM *pNow = pItemLast ?
			pItemLast->next :
			pgdata->static_data.hashtable[ HashFunc( phoneSeq ) ];

	for ( ; pNow; pNow = pNow->next )
		if ( PhoneSeqTheSame( pNow->data.phoneSeq, phoneSeq ) )
			return pNow;
	return NULL;
}

HASH_ITEM *HashFindEntry( ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[] )
{
	HASH_ITEM *pItem;
	int hashvalue;

	hashvalue = HashFunc( phoneSeq );

	for ( pItem = pgdata->static_data.hashtable[ hashvalue ]; pItem ; pItem = pItem->next ) {
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
	int hashvalue;
	HASH_ITEM *pItem;

	pItem = HashFindEntry( pgdata, pData->phoneSeq, pData->wordSeq );
	if ( pItem != NULL )
		return pItem;

	pItem = ALC( HASH_ITEM, 1 );
	if ( ! pItem )
		return NULL;  /* Error occurs */

	hashvalue = HashFunc( pData->phoneSeq );
	/* set the new element */
	pItem->next = pgdata->static_data.hashtable[ hashvalue ];

	memcpy( &( pItem->data ), pData, sizeof( pItem->data ) );
	pItem->item_index = -1;

	/* set link to the new element */
	pgdata->static_data.hashtable[ hashvalue ] = pItem;

	return pItem;
}

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

/*
 * capacity of 'str' MUST bigger then FIELD_SIZE !
 */
static void HashItem2Binary( char *str, HASH_ITEM *pItem )
{
	int i, phraselen;
	char *pc;

	memset( str, 0, FIELD_SIZE );
	if ( sizeof(int) * 4 + ueStrLen( pItem->data.wordSeq ) * 2 +
	     strlen( pItem->data.wordSeq ) >= FIELD_SIZE ) {
		/* exceed buffer size */
		return;
	}

	/* freq info */
	PutInt32( pItem->data.userfreq, &str[ 0 ] );
	PutInt32( pItem->data.recentTime, &str[ 4 ] );
	PutInt32( pItem->data.maxfreq, &str[ 8 ] );
	PutInt32( pItem->data.origfreq, &str[ 12 ] );

	/* phone seq*/
	phraselen = ueStrLen( pItem->data.wordSeq );
	str[ 16 ] = phraselen;
	pc = &str[ 17 ];
	for ( i = 0; i < phraselen; i++ ) {
		PutUint16( pItem->data.phoneSeq[ i ], pc );
		pc += 2;
	}

	/* phrase */
	*pc = strlen( pItem->data.wordSeq );
	strcpy( (pc + 1), pItem->data.wordSeq );
	pItem->data.wordSeq[ (unsigned char) *pc ] = '\0';
}

void HashModify( ChewingData *pgdata, HASH_ITEM *pItem )
{
	FILE *outfile;
	char str[ FIELD_SIZE + 1 ];

	outfile = fopen( pgdata->static_data.hashfilename, "r+b" );
	if ( !outfile )
		return;

	/* update "lifetime" */
	fseek( outfile, strlen( BIN_HASH_SIG ), SEEK_SET );
	fwrite( &pgdata->static_data.chewing_lifetime, 1, 4, outfile );
	sprintf( str, "%d", pgdata->static_data.chewing_lifetime );
	DEBUG_OUT( "HashModify-1: '%-75s'\n", str );

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

	HashItem2String( str, pItem );
	DEBUG_OUT( "HashModify-2: '%-75s'\n", str );

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

/**
 * @return 1, 0 or -1
 * retval 0	end of file
 * retval 1	continue
 * retval -1	ignore this record
 */
static int ReadHashItem_bin( const char *srcbuf, HASH_ITEM *pItem, int item_index )
{
	int len, i;
	const char *pc;

	memset( pItem, 0, sizeof(HASH_ITEM) );

	/* freq info */
	pItem->data.userfreq	= GetInt32(&srcbuf[ 0 ]);
	pItem->data.recentTime	= GetInt32(&srcbuf[ 4 ]);
	pItem->data.maxfreq	= GetInt32(&srcbuf[ 8 ]);
	pItem->data.origfreq	= GetInt32(&srcbuf[ 12 ]);

	/* phone seq, length in num of chi words */
	len = (int) srcbuf[ 16 ];
	pItem->data.phoneSeq = ALC( uint16_t, len + 1 );
	pc = &srcbuf[ 17 ];
	for ( i = 0; i < len; i++ ) {
		pItem->data.phoneSeq[ i ] = GetUint16( pc );
		pc += 2;
	}
	pItem->data.phoneSeq[ i ] = 0;

	/* phrase, length in num of bytes */
	pItem->data.wordSeq = ALC( char, (*pc) + 1 );
	strcpy( pItem->data.wordSeq, (char *) (pc + 1) );
	pItem->data.wordSeq[ (unsigned int) *pc ] = '\0';

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
static int ReadHashItem_txt( FILE *infile, HASH_ITEM *pItem, int item_index )
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
	pItem->data.phoneSeq = ALC( uint16_t, len + 1 );
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

static char *_load_hash_file( const char *filename, int *size )
{
	int flen;
	char *pd = NULL;
	FILE *tf;

	tf = open_file_get_length( filename, "rb", &flen );
	if ( tf == NULL ) {
		goto err_load_file;
	}
	pd = ALC( char, flen );
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

/* migrate from text-based hash to binary form */
static int migrate_hash_to_bin( ChewingData *pgdata )
{
	FILE *txtfile;
	char oldname[ 256 ], *dump, *seekdump;
	HASH_ITEM item;
	int item_index, iret, tflen;
	int ret;
	const char *ofilename = pgdata->static_data.hashfilename;

	/* allocate dump buffer */
	txtfile = open_file_get_length( ofilename, "r", &tflen );
	if ( txtfile == NULL ) {
		return 0;
	}
	dump = ALC( char, tflen * 2 );
	if ( dump == NULL ) {
		fclose( txtfile );
		return 0;
	}
	ret = fscanf( txtfile, "%d", &pgdata->static_data.chewing_lifetime );
	if ( ret != 1 ) {
		return 0;
	}

	/* prepare the bin file */
	seekdump = dump;
	memcpy( seekdump, BIN_HASH_SIG, strlen( BIN_HASH_SIG ) );
	memcpy( seekdump + strlen( BIN_HASH_SIG ),
	        &pgdata->static_data.chewing_lifetime,
		sizeof(pgdata->static_data.chewing_lifetime) );
	seekdump += strlen( BIN_HASH_SIG ) + sizeof(pgdata->static_data.chewing_lifetime);

	/* migrate */
	item_index = 0;
	while ( 1 ) {
		iret = ReadHashItem_txt( txtfile, &item, ++item_index );

		if ( iret == -1 ) {
			--item_index;
			continue;
		}
		else if ( iret == 0 )
			break;

		HashItem2Binary( seekdump, &item );
		seekdump += FIELD_SIZE;
		free( item.data.phoneSeq );
		free( item.data.wordSeq );
	};
	fclose( txtfile );

	/* backup as *.old */
	strncpy( oldname, ofilename, sizeof(oldname) );
	strncat( oldname, ".old", sizeof(oldname) - strlen(oldname) - 1 );
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

	return 1;
}

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
		pItem = pgdata->static_data.hashtable[ i ];
		DEBUG_CHECKPOINT();
		FreeHashItem( pItem );
	}
}

int InitHash( ChewingData *pgdata )
{
	HASH_ITEM item, *pItem, *pPool = NULL;
	int item_index, hashvalue, iret, fsize, hdrlen, oldest = INT_MAX;
	char *dump, *seekdump;

	const char *path = getenv( "CHEWING_USER_PATH" );

	/* make sure of write permission */
	if ( path && access( path, W_OK ) == 0 ) {
		sprintf( pgdata->static_data.hashfilename, "%s" PLAT_SEPARATOR "%s", path, HASH_FILE );
	} else {
		if ( getenv( "HOME" ) ) {
			sprintf(
				pgdata->static_data.hashfilename, "%s%s",
				getenv( "HOME" ), CHEWING_HASH_PATH );
		}
		else {
			sprintf(
				pgdata->static_data.hashfilename, "%s%s",
				PLAT_TMPDIR, CHEWING_HASH_PATH );
		}
		PLAT_MKDIR( pgdata->static_data.hashfilename );
		strcat( pgdata->static_data.hashfilename, PLAT_SEPARATOR );
		strcat( pgdata->static_data.hashfilename, HASH_FILE );
	}
	memset( pgdata->static_data.hashtable, 0, sizeof( pgdata->static_data.hashtable ) );

open_hash_file:
	dump = _load_hash_file( pgdata->static_data.hashfilename, &fsize );
	hdrlen = strlen( BIN_HASH_SIG ) + sizeof(pgdata->static_data.chewing_lifetime);
	item_index = 0;
	if ( dump == NULL || fsize < hdrlen ) {
		FILE *outfile;
		outfile = fopen( pgdata->static_data.hashfilename, "w+b" );
		if ( ! outfile ) {
			if ( dump ) {
				free( dump );
			}
			return 0;
		}
		pgdata->static_data.chewing_lifetime = 0;
		fwrite( BIN_HASH_SIG, 1, strlen( BIN_HASH_SIG ), outfile );
		fwrite( &pgdata->static_data.chewing_lifetime, 1,
		                sizeof(pgdata->static_data.chewing_lifetime), outfile );
		fclose( outfile );
	}
	else {
		if ( memcmp(dump, BIN_HASH_SIG, strlen(BIN_HASH_SIG)) != 0 ) {
			/* perform migrate from text-based to binary form */
			free( dump );
			if ( ! migrate_hash_to_bin( pgdata ) ) {
				return  0;
			}
			goto open_hash_file;
		}

		pgdata->static_data.chewing_lifetime = *(int *) (dump + strlen( BIN_HASH_SIG ));
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
			pItem->next = pgdata->static_data.hashtable[ hashvalue ];
			pgdata->static_data.hashtable[ hashvalue ] = pItem;
			pItem->data.recentTime -= oldest;
		}
		pgdata->static_data.chewing_lifetime -= oldest;
	}
	return 1;
}

