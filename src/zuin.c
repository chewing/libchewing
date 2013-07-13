/**
 * zuin.c
 *
 * Copyright (c) 1999, 2000, 2001
 *      Lu-chuan Kung and Kang-pen Chen.
 *      All rights reserved.
 *
 * Copyright (c) 2004-2006, 2008-2010, 2012
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/**
 * @file zuin.c
 *
 * control keyboard mapping
 * include the definition of ZuinData structure
 */

#include <ctype.h>
#include <string.h>

#include "zuin-private.h"
#include "char-private.h"
#include "key2pho-private.h"
#include "pinyin-private.h"
#include "private.h"

/*
 * process a key input
 * return value:
 *	ZUIN_ABSORB
 *	ZUIN_COMMIT
 *	ZUIN_KEY_ERROR
 *	ZUIN_ERROR
 */
static int IsHsuPhoEndKey( const int pho_inx[], int key )
{
	switch ( key ) {
		case 's':
		case 'd':
		case 'f':
		case 'j':
		case ' ':
			return ( pho_inx[ 0 ] || pho_inx[ 1 ] || pho_inx[ 2 ] );
		default:
			return 0;
	}
}

/* copy the idea from HSU keyboard */
static int IsET26PhoEndKey( const int pho_inx[], int key )
{
	switch ( key ) {
		case 'd':
		case 'f':
		case 'j':
		case 'k':
		case ' ':
			return ( pho_inx[ 0 ] || pho_inx[ 1 ] || pho_inx[ 2 ] );
		default:
			return 0;
	}
}

/* copy the idea from HSU keyboard */
static int IsDACHENCP26PhoEndKey( const int pho_inx[], int key )
{
	switch ( key ) {
		case 'e':
		case 'r':
		case 'd':
		case 'y':
		case ' ':
			return ( pho_inx[ 0 ] || pho_inx[ 1 ] || pho_inx[ 2 ] );
		default:
			return 0;
	}
}

static int IsDefPhoEndKey( int key, int kbtype )
{
	if ( PhoneInxFromKey( key, 3, kbtype, 1 )  )
		return 1;

	if ( key == ' ' )
		return 1;
	return 0;
}

static int EndKeyProcess( ChewingData *pgdata, int key, int searchTimes )
{
	ZuinData *pZuin = &(pgdata->zuinData);
	uint16_t u16Pho, u16PhoAlt;
	Word tempword;
	int pho_inx;

	if (
		pZuin->pho_inx[ 0 ] == 0 &&
		pZuin->pho_inx[ 1 ] == 0 &&
		pZuin->pho_inx[ 2 ] == 0 &&
		pZuin->pho_inx[ 3 ] == 0 ) {
		/*
		 * Special handle for space key (Indeed very special one).
		 * Un-break the situation that OnKeySpace() is not called,
		 * hence the Candidate window doesn't show up, because
		 * ZUIN_NO_WORD is returned.
		 */
		return (key == ' ') ? ZUIN_KEY_ERROR : ZUIN_NO_WORD;
	}

	pho_inx = PhoneInxFromKey( key, 3, pZuin->kbtype, searchTimes );
	if ( pZuin->pho_inx[ 3 ] == 0 ) {
		pZuin->pho_inx[ 3 ] = pho_inx;
		pZuin->pho_inx_alt[ 3 ] = pho_inx;
	}
	else if ( key != ' ' ) {
		pZuin->pho_inx[ 3 ] = pho_inx;
		pZuin->pho_inx_alt[ 3 ] = pho_inx;
		return ZUIN_NO_WORD;
	}

	u16Pho = UintFromPhoneInx( pZuin->pho_inx );
	if ( GetCharFirst( pgdata, &tempword, u16Pho ) == 0 ) {
		ZuinRemoveAll( pZuin );
		return ZUIN_NO_WORD;
	}

	pZuin->phone = u16Pho;

	if ( pZuin->pho_inx_alt[ 0 ] == 0 &&
	     pZuin->pho_inx_alt[ 1 ] == 0 &&
	     pZuin->pho_inx_alt[ 2 ] == 0 ) {
		/* no alternative phone, copy from default as alt */
		pZuin->phoneAlt = u16Pho;
	}
	else {
		u16PhoAlt = UintFromPhoneInx( pZuin->pho_inx_alt );
		pZuin->phoneAlt = u16PhoAlt;
	}

	memset( pZuin->pho_inx, 0, sizeof( pZuin->pho_inx ) );
	memset( pZuin->pho_inx_alt, 0, sizeof( pZuin->pho_inx_alt ) );
	return ZUIN_COMMIT;
}

static int DefPhoInput( ChewingData *pgdata, int key )
{
	ZuinData *pZuin = &(pgdata->zuinData);
	int type = 0, inx = 0;
	int i;

	if ( IsDefPhoEndKey( key, pZuin->kbtype ) ) {
		for ( i = 0; i < ZUIN_SIZE; ++i )
			if ( pZuin->pho_inx[ i ] != 0 )
				break;
		if ( i < ZUIN_SIZE )
			return EndKeyProcess( pgdata, key, 1 );
	}
	else {
		pZuin->pho_inx[ 3 ] = 0;
	}

	/* decide if the key is a phone */
	for ( type = 0; type <= 3; type++ ) {
		inx = PhoneInxFromKey( key, type, pZuin->kbtype, 1 );
		if ( inx )
			break;
	}

	/* the key is NOT a phone */
	if ( type > 3 ) {
		return ZUIN_KEY_ERROR;
	}

	/* fill the key into the phone buffer */
	pZuin->pho_inx[ type ] = inx;
	return ZUIN_ABSORB;
}

static int HsuPhoInput( ChewingData *pgdata, int key )
{
	ZuinData *pZuin = &(pgdata->zuinData);
	int type = 0, searchTimes = 0, inx = 0;

	/* Dvorak Hsu key has already converted to Hsu */
	if ( IsHsuPhoEndKey( pZuin->pho_inx, key ) ) {
		if ( pZuin->pho_inx[ 1 ] == 0 && pZuin->pho_inx[ 2 ] == 0 ) {
			/* convert "ㄐㄑㄒ" to "ㄓㄔㄕ" */
			if ( 12 <= pZuin->pho_inx[ 0 ] && pZuin->pho_inx[ 0 ] <= 14 ) {
				pZuin->pho_inx[ 0 ] += 3 ;
			}
			/* convert "ㄏ" to "ㄛ" */
			else if ( pZuin->pho_inx[ 0 ] == 11 ) {
				pZuin->pho_inx[ 0 ] = 0;
				pZuin->pho_inx[ 2 ] = 2;
			}
			/* convert "ㄍ" to "ㄜ" */
			else if ( pZuin->pho_inx[ 0 ] == 9 ) {
				pZuin->pho_inx[ 0 ] = 0;
				pZuin->pho_inx[ 2 ] = 3;
			}
			/* convert "ㄇ" to "ㄢ" */
			else if ( pZuin->pho_inx[ 0 ] == 3 ) {
				pZuin->pho_inx[ 0 ] = 0;
				pZuin->pho_inx[ 2 ] = 9;
			}
			/* convert "ㄋ" to "ㄣ" */
			else if ( pZuin->pho_inx[ 0 ] == 7 ) {
				pZuin->pho_inx[ 0 ] = 0;
				pZuin->pho_inx[ 2 ] = 10;
			}
			/* convert "ㄎ" to "ㄤ" */
			else if ( pZuin->pho_inx[ 0 ] == 10 ) {
				pZuin->pho_inx[ 0 ] = 0;
				pZuin->pho_inx[ 2 ] = 11;
			}
			/* convert "ㄌ" to "ㄦ" */
			else if ( pZuin->pho_inx[ 0 ] == 8 ) {
				pZuin->pho_inx[ 0 ] = 0;
				pZuin->pho_inx[ 2 ] = 13;
			}
		}

		if (
			( pZuin->pho_inx[ 0 ] == 9 ) &&
			( ( pZuin->pho_inx[ 1 ] == 1 ) ||
				( pZuin->pho_inx[ 1 ] == 3 ) ) ) {
			pZuin->pho_inx[ 0 ] = 12;
		}

		searchTimes = ( key == 'j' ) ? 3 : 2;

		return EndKeyProcess( pgdata, key, searchTimes );
	}
	else {
		/* decide if the key is a phone */
		for ( type = 0, searchTimes = 1; type < 3; type++ ) {
			inx = PhoneInxFromKey( key, type, pZuin->kbtype, searchTimes );
			if ( ! inx )
				continue; /* if inx == 0, next type */
			else if ( type == 0 ) {
				if ( pZuin->pho_inx[ 0 ] || pZuin->pho_inx[ 1 ] ) {
					/* if inx !=0 */
					searchTimes = 2 ; /* possible infinite loop here */
				}
				else
					break;
			}
			else if ( type == 1 && inx == 1 ) { /* handle i and e*/
				if ( pZuin->pho_inx[ 1 ] ) {
					searchTimes = 2;
				}
				else
					break;
			}
			else
				break;
		}
		/* processing very special cases "j v c" */
		if (
			type == 1 &&
			inx == 2 &&
			12 <= pZuin->pho_inx[ 0 ] &&
			pZuin->pho_inx[ 0 ] <= 14 ) {
			pZuin->pho_inx[ 0 ] += 3;
		}

		/* Fuzzy "g e" to "j e" */
		if (
			( pZuin->pho_inx[ 0 ] == 9 ) &&
			( ( pZuin->pho_inx[ 1 ] == 1 ) || ( pZuin->pho_inx[ 1 ] == 3 ) ) ) {
			pZuin->pho_inx[ 0 ] = 12;
		}

		/* ㄐㄑㄒ must follow ㄧㄩ */
		if (
			type == 2 &&
			pZuin->pho_inx[ 1 ] == 0 &&
			12 <= pZuin->pho_inx[ 0 ] &&
			pZuin->pho_inx[ 0 ] <= 14 ) {
			pZuin->pho_inx[ 0 ] += 3;
		}

		if ( type == 3 ) { /* the key is NOT a phone */
			if ( isalpha( key ) )
				return ZUIN_NO_WORD;
			return ZUIN_KEY_ERROR;
		}
		/* fill the key into the phone buffer */
		pZuin->pho_inx[ type ] = inx;
		return ZUIN_ABSORB;
	}
}

/* copy the idea from hsu */
static int ET26PhoInput( ChewingData *pgdata, int key )
{
	ZuinData *pZuin = &(pgdata->zuinData);
	int type = 0, searchTimes = 0, inx = 0;

	if ( IsET26PhoEndKey( pZuin->pho_inx, key ) ) {
		if ( pZuin->pho_inx[ 1 ] == 0 && pZuin->pho_inx[ 2 ] == 0 ) {
			/* convert "ㄐㄒ" to "ㄓㄕ" */
			if ( pZuin->pho_inx[ 0 ] == 12 || pZuin->pho_inx[ 0 ] == 14 ) {
				pZuin->pho_inx[ 0 ] += 3;
			}
			/* convert "ㄆ" to "ㄡ" */
			else if ( pZuin->pho_inx[ 0 ] == 2 ) {
				pZuin->pho_inx[ 0 ] = 0;
				pZuin->pho_inx[ 2 ] = 8;
			}
			/* convert "ㄇ" to "ㄢ" */
			else if ( pZuin->pho_inx[ 0 ] == 3 ) {
				pZuin->pho_inx[ 0 ] = 0;
				pZuin->pho_inx[ 2 ] = 9;
			}
			/* convert "ㄋ" to "ㄣ" */
			else if ( pZuin->pho_inx[ 0 ] == 7) {
				pZuin->pho_inx[ 0 ] = 0;
				pZuin->pho_inx[ 2 ] = 10;
			}
			/* convert "ㄊ" to "ㄤ" */
			else if ( pZuin->pho_inx[ 0 ] == 6 ) {
				pZuin->pho_inx[ 0 ] = 0;
				pZuin->pho_inx[ 2 ] = 11;
			}
			/* convert "ㄌ" to "ㄥ" */
			else if ( pZuin->pho_inx[ 0 ] == 8 ) {
				pZuin->pho_inx[ 0 ] = 0;
				pZuin->pho_inx[ 2 ] = 12;
			}
			/* convert "ㄏ" to "ㄦ" */
			else if ( pZuin->pho_inx[ 0 ] == 11 ) {
				pZuin->pho_inx[ 0 ] = 0;
				pZuin->pho_inx[ 2 ] = 13;
			}
		}
		searchTimes = 2;
		return EndKeyProcess( pgdata, key, searchTimes );
	}
	else {
		/* decide if the key is a phone */
		for ( type = 0, searchTimes = 1; type < 3; type++ ) {
			inx = PhoneInxFromKey( key, type, pZuin->kbtype, searchTimes );
			if ( ! inx )
				continue; /* if inx == 0, next type */
			else if ( type == 0 ) {
				if ( pZuin->pho_inx[ 0 ] || pZuin->pho_inx[ 1 ] ) {
					/* if inx !=0 */
					searchTimes = 2 ; /* possible infinite loop here */
				}
				else
					break;
			}
			else
				break;
		}
		/* convert "ㄐㄒ" to "ㄓㄕ" */
		if ( type == 1 ) {
			if ( inx == 2 ) {
				if (
					pZuin->pho_inx[ 0 ] == 12 ||
					pZuin->pho_inx[ 0 ] == 14 ) {
					pZuin->pho_inx[ 0 ] += 3;
				}
			}
			else {
				/* convert "ㄍ" to "ㄑ" */
				if ( pZuin->pho_inx[ 0 ] == 9 ) {
					pZuin->pho_inx[ 0 ] = 13;
				}
			}
		}

		if (
			type == 2 &&
			pZuin->pho_inx[ 1 ] == 0 &&
			(pZuin->pho_inx[ 0 ] == 12 || pZuin->pho_inx[ 0 ] == 14 ) ) {
			pZuin->pho_inx[ 0 ] += 3;
		}

		if ( type == 3 ) { /* the key is NOT a phone */
			if ( isalpha( key ) )
				return ZUIN_NO_WORD;
			return ZUIN_KEY_ERROR;
		}
		/* fill the key into the phone buffer */
		pZuin->pho_inx[ type ] = inx;
		return ZUIN_ABSORB;
	}
}

static int SwitchingBetween( int *pho_idx, int a, int b ) {
	if ( *pho_idx == a ) {
		*pho_idx = b;
		return 1;
	} else if ( *pho_idx == b ) {
		*pho_idx = a;
		return 1;
	}
	return 0;
}

static int DACHENCP26PhoInput( ChewingData *pgdata, int key )
{
	ZuinData *pZuin = &(pgdata->zuinData);
	int type = 0, searchTimes = 0, inx = 0;

	if ( IsDACHENCP26PhoEndKey( pZuin->pho_inx, key ) ) {
		searchTimes = 2;
		return EndKeyProcess( pgdata, key, searchTimes );
	}
	else {
		/* decide if the key is a phone */
		for ( type = 0, searchTimes = 1; type < 3; type++ ) {
			inx = PhoneInxFromKey( key, type, pZuin->kbtype, searchTimes );
			if ( ! inx )
				continue; /* if inx == 0, next type */
			else if ( type == 0 ) {
				break;
				if ( pZuin->pho_inx[ 0 ] || pZuin->pho_inx[ 1 ] ) {
					/* if inx !=0 */
					searchTimes = 2 ; /* possible infinite loop here */
				}
				else
					break;
			}
			else
				break;
		}
		/* switching between "ㄅ" and "ㄆ" */
		if ( key == 'q' ) {
			if ( SwitchingBetween ( &pZuin->pho_inx[ 0 ], 1, 2 ) ) {
				return ZUIN_ABSORB;
			}
		}
		/* switching between "ㄉ" and "ㄊ" */
		else if ( key == 'w' ) {
			if ( SwitchingBetween ( &pZuin->pho_inx[ 0 ], 5, 6 ) ) {
				return ZUIN_ABSORB;
			}
		}
		/* switching between "ㄓ" and "ㄔ" */
		else if ( key == 't' ) {
			if ( SwitchingBetween ( &pZuin->pho_inx[ 0 ], 15, 16 ) ) {
				return ZUIN_ABSORB;
			}
		}
		/* converting "ㄖ" to "ㄝ" */
		else if ( key == 'b' ) {
			if ( pZuin->pho_inx[ 0 ] != 0 || pZuin->pho_inx[1] != 0 ) {
			pZuin->pho_inx[ 2 ] = 4;
				return ZUIN_ABSORB;
			}
		}
		/* converting "ㄙ" to "ㄣ" */
		else if ( key == 'n' ) {
			if ( pZuin->pho_inx[ 0 ] != 0 || pZuin->pho_inx[1] != 0 ) {
				pZuin->pho_inx[ 2 ] = 12;
				return ZUIN_ABSORB;
			}
		}
		/* switching between "ㄧ", "ㄚ", and "ㄧㄚ" */
		else if ( key == 'u' ) {
			if ( pZuin->pho_inx[ 1 ] == 1 && pZuin->pho_inx[ 2 ] != 1 ) {
				pZuin->pho_inx[1] = 0;
				pZuin->pho_inx[2] = 1;
				return ZUIN_ABSORB;
			}
			else if (pZuin->pho_inx[ 1 ] != 1 && pZuin->pho_inx[2] == 1) {
				pZuin->pho_inx[1] = 1;
				return ZUIN_ABSORB;
			}
			else if (pZuin->pho_inx[1] == 1 && pZuin->pho_inx[2]==1) {
				pZuin->pho_inx[1]=0;
				pZuin->pho_inx[2]=0;
				return ZUIN_ABSORB;
			}
			else if (pZuin->pho_inx[1] != 0) {
				pZuin->pho_inx[2] = 1;
				return ZUIN_ABSORB;
			}
		}
		/* switching between "ㄩ" and "ㄡ" */
		else if ( key == 'm' ) {
			if ( pZuin->pho_inx[ 1 ] == 3 && pZuin->pho_inx[ 2 ] != 8 ) {
				pZuin->pho_inx[1] = 0;
				pZuin->pho_inx[2] = 8;
				return ZUIN_ABSORB;
			}
			else if (pZuin->pho_inx[ 1 ] != 3 && pZuin->pho_inx[2] == 8) {
				pZuin->pho_inx[1] = 3;
				pZuin->pho_inx[2] = 0;
				return ZUIN_ABSORB;
			}
			else if (pZuin->pho_inx[ 1 ] != 0) {
				pZuin->pho_inx[2] = 8;
				return ZUIN_ABSORB;
			}
		}
		/* switching between "ㄛ" and "ㄞ" */
		else if ( key == 'i' ) {
			if ( SwitchingBetween ( &pZuin->pho_inx[ 2 ], 2, 5 ) ) {
				return ZUIN_ABSORB;
			}
		}
		/* switching between "ㄟ" and "ㄢ" */
		else if ( key == 'o' ) {
			if ( SwitchingBetween ( &pZuin->pho_inx[ 2 ], 6, 9 ) ) {
				return ZUIN_ABSORB;
			}
		}
		/* switching between "ㄠ" and "ㄤ" */
		else if ( key == 'l' ) {
			if ( SwitchingBetween ( &pZuin->pho_inx[ 2 ], 7, 11 ) ) {
				return ZUIN_ABSORB;
			}
		}
		/* switching between "ㄣ" and "ㄦ" */
		else if ( key == 'p' ) {
			if ( SwitchingBetween ( &pZuin->pho_inx[ 2 ], 10, 13 ) ) {
				return ZUIN_ABSORB;
			}
		}
		if ( type == 3 ) { /* the key is NOT a phone */
			if ( isalpha( key ) )
				return ZUIN_NO_WORD;
			return ZUIN_KEY_ERROR;
		}
		/* fill the key into the phone buffer */
		pZuin->pho_inx[ type ] = inx;
		return ZUIN_ABSORB;
	}
}

static int IsPinYinEndKey(int key )
{
	if ( (key == ' ') || (key == '1') || (key == '2') ||
			(key == '3') || (key == '4') || (key == '5') ) {
		return 1;
	}
	return 0;
}

static int IsSymbolKey(int key)
{
	if ( (key < 97) || (key > 122) ) {
		return 1;
	}

	return 0;
}

static int PinYinInput( ChewingData *pgdata, int key )
{
	ZuinData *pZuin = &(pgdata->zuinData);
	int err = 0;
	unsigned int i;
	char zuinKeySeq[ 5 ], zuinKeySeqAlt[ 5 ], buf[ 2 ];
	size_t len;

	DEBUG_CHECKPOINT();

	if ( pZuin->pinYinData.keySeq[ 0 ] == 0 && IsSymbolKey( key ) ) {
		return ZUIN_KEY_ERROR;
	}

	if ( IsPinYinEndKey( key ) ) {
		err = PinyinToZuin( pgdata, pZuin->pinYinData.keySeq,
		                    zuinKeySeq, zuinKeySeqAlt );
		if ( err ) {
			pZuin->pinYinData.keySeq[ 0 ] = '\0';
			return ZUIN_ABSORB;
		}

		DEBUG_OUT( "zuinKeySeq: %s\n", zuinKeySeq );
		DEBUG_OUT( "zuinKeySeqAlt: %s\n", zuinKeySeqAlt );

		len = strlen( zuinKeySeq );
		for ( i = 0; i < len; i++ ) {
			int type = 0, inx = 0;
			for ( type = 0; type <= 3; type++ ) {
				inx = PhoneInxFromKey( zuinKeySeq[ i ],
				                       type, pZuin->kbtype, 1 );
				if ( inx )
					break;
			}

			/* the key is NOT a phone */
			if ( type > 3 ) {
				return ZUIN_KEY_ERROR;
			}

			pZuin->pho_inx[ type ] = inx;

		}

		len = strlen( zuinKeySeqAlt );
		for ( i = 0; i < len; i++ ) {
			int type = 0, inx = 0;
			for ( type = 0; type <= 3; type++ ) {
				inx = PhoneInxFromKey( zuinKeySeqAlt[ i ],
				                       type, pZuin->kbtype, 1 );
				if ( inx )
					break;
			}

			/* the key is NOT a phone */
			if ( type > 3 ) {
				return ZUIN_KEY_ERROR;
			}

			pZuin->pho_inx_alt[ type ] = inx;

		}

		switch ( key ) {
			case '1':
				key = ' ';
				break;
			case '2':
				key = '6';
				break;
			case '5':
				key = '7';
		}
		pZuin->pinYinData.keySeq[ 0 ] = '\0';
		return EndKeyProcess( pgdata, key, 1 );
	}
	buf[ 0 ] = key; buf[ 1 ] = '\0';
	strcat( pZuin->pinYinData.keySeq, buf );

	DEBUG_OUT( "PinYin Seq: %s\n", pZuin->pinYinData.keySeq );

	return ZUIN_ABSORB;
}

/* key: ascii code of input, including space */
int ZuinPhoInput( ChewingData *pgdata, int key )
{
	ZuinData *pZuin = &(pgdata->zuinData);
	switch ( pZuin->kbtype ) {
		case KB_HSU:
		case KB_DVORAK_HSU:
			return HsuPhoInput( pgdata, key );
			break;
		case KB_ET26:
			return ET26PhoInput( pgdata, key );
			break;
		case KB_DACHEN_CP26:
			return DACHENCP26PhoInput( pgdata, key );
			break;
		case KB_HANYU_PINYIN:
		case KB_THL_PINYIN:
		case KB_MPS2_PINYIN:
			return PinYinInput( pgdata, key );
			break;
		default:
			return DefPhoInput( pgdata, key );
	}
	return ZUIN_ERROR;
}

/* remove the latest key */
int ZuinRemoveLast( ZuinData *pZuin )
{
	int i;
	if ( pZuin->kbtype >= KB_HANYU_PINYIN ) {
		i = strlen( pZuin->pinYinData.keySeq );
		pZuin->pinYinData.keySeq[ i - 1 ] = '\0';
	} else {
		for ( i = 3; i >= 0; i-- ) {
			if ( pZuin->pho_inx[ i ] ) {
				pZuin->pho_inx[ i ] = 0;
				return 0;
			}
		}
	}
	return 0;
}

/* remove all the key entered */
int ZuinRemoveAll( ZuinData *pZuin )
{
	memset( pZuin->pho_inx, 0, sizeof( pZuin->pho_inx ) );
	memset( pZuin->pinYinData.keySeq, 0, sizeof( pZuin->pinYinData.keySeq ) );
	return 0;
}

int ZuinIsEntering( ZuinData *pZuin )
{
	int i;
        if ( pZuin->kbtype >= KB_HANYU_PINYIN ) {
	    if ( pZuin->pinYinData.keySeq[0] )
		return 1;
	} else {
	    for ( i = 0; i < ZUIN_SIZE; i++ )
		if ( pZuin->pho_inx[ i ] )
		    return 1;
	}
	return 0;
}
