/**
 * tree.c
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004, 2005, 2006, 2008, 2011
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/**
 *	@file tree.c
 *	@brief API for accessing the phrase tree.
 */
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "chewing-private.h"
#include "chewing-utf8-util.h"
#include "chewing-definition.h"
#include "userphrase-private.h"
#include "global.h"
#include "global-private.h"
#include "dict-private.h"
#include "char-private.h"
#include "tree-private.h"
#include "private.h"
#include "plat_mmap.h"

#define INTERVAL_SIZE ( ( MAX_PHONE_SEQ_LEN + 1 ) * MAX_PHONE_SEQ_LEN / 2 )

typedef struct {
	int from, to, pho_id, source;
	Phrase *p_phr;
} PhraseIntervalType;

typedef struct tagRecordNode {
	int *arrIndex;		/* the index array of the things in "interval" */
	int nInter, score;
	struct tagRecordNode *next;
	int nMatchCnnct;	/* match how many Cnnct. */
} RecordNode;

typedef struct {
	int leftmost[ MAX_PHONE_SEQ_LEN + 1 ] ;
	char graph[ MAX_PHONE_SEQ_LEN + 1 ][ MAX_PHONE_SEQ_LEN + 1 ];
	PhraseIntervalType interval[ MAX_INTERVAL ];
	int nInterval;
	RecordNode *phList;
	int nPhListLen;
} TreeDataType;

static int IsContain( IntervalType in1, IntervalType in2 )
{
	return ( in1.from <= in2.from && in1.to >= in2.to );
}

int IsIntersect( IntervalType in1, IntervalType in2 )
{
	return ( max( in1.from, in2.from ) < min( in1.to, in2.to ) );
}

static int PhraseIntervalContain(PhraseIntervalType in1, PhraseIntervalType in2)
{
	return ( in1.from <= in2.from && in1.to >= in2.to );
}

static int PhraseIntervalIntersect(PhraseIntervalType in1, PhraseIntervalType in2)
{
	return ( max( in1.from, in2.from ) < min( in1.to, in2.to ) );
}

void TerminateTree( ChewingData *pgdata )
{
#ifdef USE_BINARY_DATA
		pgdata->static_data.tree = NULL;
		plat_mmap_close( &pgdata->static_data.tree_mmap );
#else
		free( pgdata->static_data.tree );
		pgdata->static_data.tree = NULL;
#endif
}


int InitTree( ChewingData *pgdata, const char * prefix )
{
#ifdef USE_BINARY_DATA
	char filename[ PATH_MAX ];
	size_t len;
	size_t offset;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, PHONE_TREE_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	plat_mmap_set_invalid( &pgdata->static_data.tree_mmap );
	pgdata->static_data.tree_size = plat_mmap_create( &pgdata->static_data.tree_mmap, filename, FLAG_ATTRIBUTE_READ );
	if ( pgdata->static_data.tree_size <= 0 )
		return -1;

	offset = 0;
	pgdata->static_data.tree = (TreeType *) plat_mmap_set_view( &pgdata->static_data.tree_mmap, &offset, &pgdata->static_data.tree_size );
	if ( !pgdata->static_data.tree )
		return -1;

	return 0;
#else
	char filename[ PATH_MAX ];
	int len;
	FILE *infile = NULL;
	int i;

	len = snprintf( filename, sizeof( filename ), "%s" PLAT_SEPARATOR "%s", prefix, PHONE_TREE_FILE );
	if ( len + 1 > sizeof( filename ) )
		return -1;

	infile = fopen( filename, "r" );
	if ( !infile )
		return -1;

	pgdata->static_data.tree = ALC( TreeType, TREE_SIZE );
	if ( !pgdata->static_data.tree ) {
		fclose( infile );
		return -1;
	}

	/* XXX: What happen if infile contains more than TREE_SIZE data? */
	for ( i = 0; i < TREE_SIZE; i++ ) {
		if ( fscanf( infile, "%hu%d%d%d",
					&pgdata->static_data.tree[ i ].phone_id,
					&pgdata->static_data.tree[ i ].phrase_id,
					&pgdata->static_data.tree[ i ].child_begin,
					&pgdata->static_data.tree[ i ].child_end ) != 4 )
			break;
	}

	fclose( infile );
	return 0;
#endif
}

static int CheckBreakpoint( int from, int to, int bArrBrkpt[] )
{
	int i;
	for ( i = from + 1; i < to; i++ )
		if ( bArrBrkpt[ i ] )
			return 0;
	return 1;
}

static int CheckUserChoose(
		ChewingData *pgdata,
		uint16_t *new_phoneSeq, int from , int to,
		Phrase **pp_phr,
		char selectStr[][ MAX_PHONE_SEQ_LEN * MAX_UTF8_SIZE + 1 ],
		IntervalType selectInterval[], int nSelect )
{
	IntervalType inte, c;
	int chno, len;
	int user_alloc;
	UserPhraseData *pUserPhraseData;
	Phrase *p_phr = ALC( Phrase, 1 );

	assert( p_phr );
	inte.from = from;
	inte.to = to;
	*pp_phr = NULL;

	/* pass 1
	 * if these exist one selected interval which is not contained by inte
	 * but has intersection with inte, then inte is an unacceptable interval
	 */
	for ( chno = 0; chno < nSelect; chno++ ) {
		c = selectInterval[ chno ];
		if ( IsIntersect( inte, c ) && ! IsContain( inte, c ) ) {
			free( p_phr );
			return 0;
		}
	}

	/* pass 2
	 * if there exist one phrase satisfied all selectStr then return 1, else return 0.
	 * also store the phrase with highest freq
	 */
	pUserPhraseData = UserGetPhraseFirst( pgdata, new_phoneSeq );
	p_phr->freq = -1;
	do {
		for ( chno = 0; chno < nSelect; chno++ ) {
			c = selectInterval[ chno ];

			if ( IsContain( inte, c ) ) {
				/*
				 * find a phrase of ph_id where the text contains
				 * 'selectStr[chno]' test if not ok then return 0,
				 * if ok then continue to test. */
				len = c.to - c.from;
				if ( memcmp(
					ueStrSeek( pUserPhraseData->wordSeq, c.from - from ),
					selectStr[ chno ],
					ueStrNBytes( selectStr[ chno ], len ) ) )
					break;
			}

		}
		if ( chno == nSelect ) {
			/* save phrase data to "pp_phr" */
			if ( pUserPhraseData->userfreq > p_phr->freq ) {
				if ( ( user_alloc = ( to - from ) ) > 0 ) {
					ueStrNCpy( p_phr->phrase,
							pUserPhraseData->wordSeq,
							user_alloc, 1);
				}
				p_phr->freq = pUserPhraseData->userfreq;
				*pp_phr = p_phr;
			}
		}
	} while ( ( pUserPhraseData = UserGetPhraseNext( pgdata, new_phoneSeq ) ) != NULL );

	if ( p_phr->freq != -1 )
		return 1;

	free( p_phr );
	return 0;
}

/*
 * phrase is said to satisfy a choose interval if
 * their intersections are the same */
static int CheckChoose(
		ChewingData *pgdata,
		int ph_id, int from, int to, Phrase **pp_phr,
		char selectStr[][ MAX_PHONE_SEQ_LEN * MAX_UTF8_SIZE + 1 ],
		IntervalType selectInterval[], int nSelect )
{
	IntervalType inte, c;
	int chno, len;
	Phrase *phrase = ALC( Phrase, 1 );

	assert( phrase );
	inte.from = from;
	inte.to = to;
	*pp_phr = NULL;

	/* if there exist one phrase satisfied all selectStr then return 1, else return 0. */
	GetPhraseFirst( pgdata, phrase, ph_id );
	do {
		for ( chno = 0; chno < nSelect; chno++ ) {
			c = selectInterval[ chno ];

			if ( IsContain( inte, c ) ) {
				/* find a phrase of ph_id where the text contains
				 * 'selectStr[chno]' test if not ok then return 0, if ok
				 * then continue to test
				 */
				len = c.to - c.from;
				if ( memcmp(
					ueStrSeek( phrase->phrase, c.from - from ),
					selectStr[ chno ],
					ueStrNBytes( selectStr[ chno ], len ) ) )
					break;
			}
			else if ( IsIntersect( inte, selectInterval[ chno ] ) ) {
				free( phrase );
				return 0;
			}
		}
		if ( chno == nSelect ) {
			*pp_phr = phrase;
			return 1;
		}
	} while ( GetPhraseNext( pgdata, phrase ) );
	free( phrase );
	return 0;
}

/** @brief search for the phrases have the same pronunciation.*/
/* if phoneSeq[begin] ~ phoneSeq[end] is a phrase, then add an interval
 * from (begin) to (end+1) */
int TreeFindPhrase( ChewingData *pgdata, int begin, int end, const uint16_t *phoneSeq )
{
	int child, tree_p, i;

	tree_p = 0;
	for ( i = begin; i <= end; i++ ) {
		for (
			child = pgdata->static_data.tree[ tree_p ].child_begin;
			child != -1 && child <= pgdata->static_data.tree[ tree_p ].child_end;
			child++ ) {

#ifdef USE_BINARY_DATA
			assert(0 <= child && child * sizeof(TreeType) < pgdata->static_data.tree_size);
#endif
			if ( pgdata->static_data.tree[ child ].phone_id == phoneSeq[ i ] )
				break;
		}
		/* if not found any word then fail. */
		if ( child == -1 || child > pgdata->static_data.tree[ tree_p ].child_end )
			return -1;
		else {
			tree_p = child;
		}
	}
	return pgdata->static_data.tree[ tree_p ].phrase_id;
}

static void AddInterval(
		TreeDataType *ptd, int begin , int end,
		int p_id, Phrase *p_phrase, int dict_or_user )
{
	ptd->interval[ ptd->nInterval ].from = begin;
	ptd->interval[ ptd->nInterval ].to = end + 1;
	ptd->interval[ ptd->nInterval ].pho_id = p_id;
	ptd->interval[ ptd->nInterval ].p_phr = p_phrase;
	ptd->interval[ ptd->nInterval ].source = dict_or_user;
	ptd->nInterval++;
}

/* Item which inserts to interval array */
typedef enum {
	USED_PHRASE_NONE,	/**< none of items used */
	USED_PHRASE_USER,	/**< User phrase */
	USED_PHRASE_DICT	/**< Dict phrase */
} UsedPhraseMode;

static void internal_release_Phrase( UsedPhraseMode mode, Phrase *pUser, Phrase *pDict )
{
	/* we must free unused phrase entry to avoid memory leak. */
	switch ( mode ) {
		case USED_PHRASE_USER:
			if ( pDict != NULL )
				free( pDict );
			break;
		case USED_PHRASE_DICT:
			if ( pUser != NULL )
				free( pUser );
			break;
		default: /* In fact, it is alwyas 0 */
			if ( pDict != NULL )
				free( pDict );
			if ( pUser != NULL )
				free( pUser );
			break;
	}
}

static void FindInterval( ChewingData *pgdata, TreeDataType *ptd )
{
	int end, begin, pho_id;
	Phrase *p_phrase, *puserphrase, *pdictphrase;
	UsedPhraseMode i_used_phrase;
	uint16_t new_phoneSeq[ MAX_PHONE_SEQ_LEN ];

	for ( begin = 0; begin < pgdata->nPhoneSeq; begin++ ) {
		for ( end = begin; end < pgdata->nPhoneSeq; end++ ) {
			if ( ! CheckBreakpoint( begin, end + 1, pgdata->bArrBrkpt ) )
				continue;

			/* set new_phoneSeq */
			memcpy(
				new_phoneSeq,
				&pgdata->phoneSeq[ begin ],
				sizeof( uint16_t ) * ( end - begin + 1 ) );
			new_phoneSeq[ end - begin + 1 ] = 0;
			puserphrase = pdictphrase = NULL;
			i_used_phrase = USED_PHRASE_NONE;

			/* check user phrase */
			if ( UserGetPhraseFirst( pgdata, new_phoneSeq ) &&
					CheckUserChoose( pgdata, new_phoneSeq, begin, end + 1,
					&p_phrase, pgdata->selectStr, pgdata->selectInterval, pgdata->nSelect ) ) {
				puserphrase = p_phrase;
			}

			/* check dict phrase */
			pho_id = TreeFindPhrase( pgdata, begin, end, pgdata->phoneSeq );
			if (
				( pho_id != -1 ) &&
				CheckChoose(
					pgdata,
					pho_id, begin, end + 1,
					&p_phrase, pgdata->selectStr,
					pgdata->selectInterval, pgdata->nSelect ) ) {
				pdictphrase = p_phrase;
			}

			/* add only one interval, which has the largest freqency
			 * but when the phrase is the same, the user phrase overrides
			 * static dict
			 */
			if ( puserphrase != NULL && pdictphrase == NULL ) {
				i_used_phrase = USED_PHRASE_USER;
			}
			else if ( puserphrase == NULL && pdictphrase != NULL ) {
				i_used_phrase = USED_PHRASE_DICT;
			}
			else if ( puserphrase != NULL && pdictphrase != NULL ) {
				/* the same phrase, userphrase overrides */
				if ( ! strcmp(
					puserphrase->phrase,
					pdictphrase->phrase ) ) {
					i_used_phrase = USED_PHRASE_USER;
				}
				else {
					if ( puserphrase->freq > pdictphrase->freq ) {
						i_used_phrase = USED_PHRASE_USER;
					}
					else {
						i_used_phrase = USED_PHRASE_DICT;
					}
				}
			}
			switch ( i_used_phrase ) {
				case USED_PHRASE_USER:
					AddInterval( ptd, begin, end, -1, puserphrase,
							IS_USER_PHRASE );
					break;
				case USED_PHRASE_DICT:
					AddInterval( ptd, begin, end, pho_id, pdictphrase,
							IS_DICT_PHRASE );
					break;
				case USED_PHRASE_NONE:
				default:
					break;
			}
			internal_release_Phrase(
				i_used_phrase,
				puserphrase,
				pdictphrase );
		}
	}
}

static void SetInfo( int len, TreeDataType *ptd )
{
	int i, a;

	for ( i = 0; i <= len; i++ )
		ptd->leftmost[ i ] = i;
	for ( i = 0; i < ptd->nInterval; i++ ) {
		ptd->graph[ ptd->interval[ i ].from ][ ptd->interval[ i ].to ] = 1;
		ptd->graph[ ptd->interval[ i ].to ][ ptd->interval[ i ].from ] = 1;
	}

	/* set leftmost */
	for ( a = 0; a <= len; a++ ) {
		for ( i = 0; i <= len; i++ ) {
			if ( ! ( ptd->graph[ a ][ i ] ) )
				continue;
			if ( ptd->leftmost[ i ] < ptd->leftmost[ a ] )
				ptd->leftmost[ a ] = ptd->leftmost[ i ];
		}
	}
}

/*
 * First we compare the 'nMatchCnnct'.
 * If the values are the same, we will compare the 'score'
 */
static int CompRecord( const RecordNode **pa, const RecordNode **pb )
{
	int diff = (*pb)->nMatchCnnct - (*pa)->nMatchCnnct;

	if ( diff )
		return diff;
	return ( (*pb)->score - (*pa)->score );
}


/*
 * Remove the interval containing in another interval.
 *
 * Example:
 * 國民大會 has three interval: 國民, 大會, 國民大會. This function removes
 * 國名, 大會 becasue 國民大會 contains 國民 and 大會.
 */
static void Discard1( TreeDataType *ptd )
{
	int a, b;
	char failflag[ INTERVAL_SIZE ];
	int nInterval2;

	memset( failflag, 0, sizeof( failflag ) );
	for ( a = 0; a < ptd->nInterval; a++ ) {
		if ( failflag[ a ] )
			continue;
		for ( b = 0; b < ptd->nInterval; b++ ) {
			if ( a == b || failflag[ b ] )
				continue ;
			if ( ptd->interval[ b ].from >= ptd->interval[ a ].from &&
				ptd->interval[ b ].to <= ptd->interval[ a ].to )
				continue;
			if ( ptd->interval[ b ].from <= ptd->interval[ a ].from &&
				ptd->interval[ b ].to <= ptd->interval[ a ].from )
				continue;
			if ( ptd->interval[ b ].from >= ptd->interval[ a ].to &&
				ptd->interval[ b ].to >= ptd->interval[ a ].to )
				continue;
			break;
		}
		/* if any other interval b is inside or leftside or rightside the
		 * interval a */
		if ( b >= ptd->nInterval ) {
			/* then kill all the intervals inside the interval a */
			int i;
			for ( i = 0; i < ptd->nInterval; i++ )  {
				if (
					! failflag[ i ] && i != a &&
					ptd->interval[ i ].from >=
						ptd->interval[ a ].from &&
					ptd->interval[ i ].to <= ptd->interval[ a ].to ) {
					failflag[ i ] = 1;
				}
			}
		}
	}
	/* discard all the intervals whose failflag[a] = 1 */
	nInterval2 = 0;
	for ( a = 0; a < ptd->nInterval; a++ ) {
		if ( ! failflag[ a ] ) {
			ptd->interval[ nInterval2++ ] = ptd->interval[ a ];
		}
		else {
			if ( ptd->interval[ a ].p_phr != NULL ) {
				free( ptd->interval[ a ].p_phr );
			}
		}
	}
	ptd->nInterval = nInterval2;
}

/*
 * Remove the interval that cannot connect to head or tail by other intervals.
 *
 * Example:
 * The input string length is 5
 * The available intervals are [1,1], [1,2], [2,3], [2,4], [5,5], [3,5].
 *
 * The possible connection from head to tail are [1,2][3,5], and
 * [1,1][2,4][5,5]. Since [2,3] cannot connect to head or tail, it is removed
 * by this function.
 */
static void Discard2( TreeDataType *ptd )
{
	int i, j;
	char overwrite[ MAX_PHONE_SEQ_LEN ], failflag[ MAX_PHONE_SEQ_LEN ];
	int nInterval2;

	memset( failflag, 0, sizeof( failflag ) );
	for ( i = 0; i < ptd->nInterval; i++ ) {
		if ( ptd->leftmost[ ptd->interval[ i ].from ] == 0 )
			continue;
		/* test if interval i is overwrited by other intervals */
		memset( overwrite, 0, sizeof( overwrite ) );
		for ( j = 0; j < ptd->nInterval; j++ ) {
			if ( j == i )
				continue;
			memset(
				&overwrite[ ptd->interval[ j ].from ],
				1,
				ptd->interval[ j ].to - ptd->interval[ j ].from );
		}
		if ( memchr(
			&overwrite[ ptd->interval[ i ].from ],
			1,
			ptd->interval[ i ].to - ptd->interval[ i ].from ) )
			failflag[ i ] = 1;
	}
	/* discard all the intervals whose failflag[a] = 1 */
	nInterval2 = 0;
	for ( i = 0; i < ptd->nInterval; i++ )
		if ( ! failflag[ i ] )
			ptd->interval[ nInterval2++ ] = ptd->interval[ i ];
	ptd->nInterval = nInterval2;
}

static void LoadChar( ChewingData *pgdata, char *buf, int buf_len, const uint16_t phoneSeq[], int nPhoneSeq )
{
	int i;
	Word word;

	memset(buf, 0, buf_len);
	for ( i = 0; i < nPhoneSeq; i++ ) {
		GetCharFirst( pgdata, &word, phoneSeq[ i ] );
		strncat(buf, word.word, buf_len - strlen(buf) - 1);
	}
	buf[ buf_len - 1 ] = '\0';
}

/* kpchen said, record is the index array of interval */
static void OutputRecordStr(
		ChewingData *pgdata,
		char *out_buf, int out_buf_len,
		const int *record, int nRecord,
		uint16_t phoneSeq[], int nPhoneSeq,
		char selectStr[][ MAX_PHONE_SEQ_LEN * MAX_UTF8_SIZE + 1 ],
		IntervalType selectInterval[],
		int nSelect, const TreeDataType *ptd )
{
	PhraseIntervalType inter;
	int i;

	LoadChar( pgdata, out_buf, out_buf_len, phoneSeq, nPhoneSeq );
	for ( i = 0; i < nRecord; i++ ) {
		inter = ptd->interval[ record[ i ] ];
		ueStrNCpy(
				ueStrSeek( out_buf, inter.from ),
				( inter.p_phr )->phrase,
				( inter.to - inter.from ), -1);
	}
	for ( i = 0; i < nSelect; i++ ) {
		inter.from = selectInterval[ i ].from;
		inter.to = selectInterval[ i ].to ;
		ueStrNCpy(
				ueStrSeek( out_buf, inter.from ),
				selectStr[ i ], ( inter.to - inter.from ), -1);
	}
}

static int rule_largest_sum( const int *record, int nRecord, const TreeDataType *ptd )
{
	int i, score = 0;
	PhraseIntervalType inter;

	for ( i = 0; i < nRecord; i++ ) {
		inter = ptd->interval[ record[ i ] ];
		assert( inter.p_phr );
		score += inter.to - inter.from;
	}
	return score;
}

static int rule_largest_avgwordlen( const int *record, int nRecord, const TreeDataType *ptd )
{
	/* constant factor 6=1*2*3, to keep value as integer */
	return 6 * rule_largest_sum( record, nRecord, ptd ) / nRecord;
}

static int rule_smallest_lenvariance( const int *record, int nRecord, const TreeDataType *ptd )
{
	int i, j, score = 0;
	PhraseIntervalType inter1, inter2;

	/* kcwu: heuristic? why variance no square function? */
	for ( i = 0; i < nRecord; i++ ) {
		for ( j = i + 1; j < nRecord; j++ ) {
			inter1 = ptd->interval[ record[ i ] ];
			inter2 = ptd->interval[ record[ j ] ];
			assert( inter1.p_phr && inter2.p_phr );
			score += abs((inter1.to - inter1.from) - (inter2.to - inter2.from));
		}
	}
	return -score;
}

static int rule_largest_freqsum( const int *record, int nRecord, const TreeDataType *ptd )
{
	int i, score = 0;
	PhraseIntervalType inter;

	for ( i = 0; i < nRecord; i++ ) {
		inter = ptd->interval[ record[ i ] ];
		assert( inter.p_phr );

		/* We adjust the 'freq' of One-word Phrase */
		score += ( inter.to - inter.from == 1 ) ?
			( inter.p_phr->freq / 512 ) :
			inter.p_phr->freq;
	}
	return score;
}

static int LoadPhraseAndCountScore( const int *record, int nRecord, const TreeDataType *ptd )
{
	int total_score = 0;
	/* NOTE: the balance factor is tuneable */
	if (nRecord) {
		total_score += 1000*rule_largest_sum( record, nRecord, ptd );
		total_score += 1000*rule_largest_avgwordlen( record, nRecord, ptd );
		total_score += 100*rule_smallest_lenvariance( record, nRecord, ptd );
		total_score += rule_largest_freqsum( record, nRecord, ptd );
	}
	return total_score;
}

static int IsRecContain( const int *intA, int nA, const int *intB, int nB, const TreeDataType *ptd )
{
	int big, sml;

	for ( big = 0, sml = 0; sml < nB; sml++ ) {
		while (
			( big < nA ) &&
			ptd->interval[ intA[ big ] ].from <
				ptd->interval[ intB[ sml ] ].to ) {
			if ( PhraseIntervalContain(
				ptd->interval[ intA[ big ] ],
				ptd->interval[ intB[ sml ] ] ) )
				break;
			big++;
		}
		if (
			( big >= nA ) ||
			ptd->interval[ intA[ big ] ].from >=
				ptd->interval[ intB[ sml ] ].to )
			return 0;
	}
	return 1;
}

static void SortListByScore( TreeDataType *ptd )
{
	int i, listLen;
	RecordNode *p, **arr;

	for (
		listLen = 0, p = ptd->phList;
		p;
		listLen++, p = p->next )
		;
	ptd->nPhListLen = listLen;

	arr = ALC( RecordNode *, listLen );
	assert( arr );

	for (
		i = 0, p = ptd->phList;
		i < listLen;
		p = p->next, i++ ) {
		arr[ i ] = p;
		p->score = LoadPhraseAndCountScore(
			p->arrIndex,
			p->nInter,
			ptd );
	}

	qsort( arr, listLen, sizeof( RecordNode * ), (CompFuncType) CompRecord );

	ptd->phList = arr[ 0 ];
	for ( i = 1; i < listLen; i++ ) {
		arr[ i - 1 ]->next = arr[ i ];
	}
	arr[ listLen - 1 ]->next = NULL;

	free( arr );
}

/* when record==NULL then output the "link list" */
static void SaveRecord( const int *record, int nInter, TreeDataType *ptd )
{
	RecordNode *now, *p, *pre;

	pre = NULL;
	for ( p = ptd->phList; p; ) {
		/* if  'p' contains 'record', then discard 'record'. */
		if ( IsRecContain( p->arrIndex, p->nInter, record, nInter, ptd ) )
			return;

		/* if 'record' contains 'p', then discard 'p'
		 * -- We must deal with the linked list. */
		if ( IsRecContain( record, nInter, p->arrIndex, p->nInter, ptd ) ) {
			RecordNode *tp = p;

			if ( pre )
				pre->next = p->next;
			else
				ptd->phList = ptd->phList->next;
			p = p->next;
			free( tp->arrIndex );
			free( tp );
		}
		else
			pre = p, p = p->next;
	}
	now = ALC( RecordNode, 1 );
	assert( now );
	now->next = ptd->phList;
	now->arrIndex = ALC( int, nInter );
	assert( now->arrIndex );
	now->nInter = nInter;
	memcpy( now->arrIndex, record, nInter * sizeof( int ) );
	ptd->phList = now;
}

static void RecursiveSave( int depth, int to, int *record, TreeDataType *ptd )
{
	int first, i;
	/* to find first interval */
	for (
		first = record[ depth - 1 ] + 1;
		ptd->interval[ first ].from < to && first < ptd->nInterval;
		first++ )
		;

	if ( first == ptd->nInterval ) {
		SaveRecord( record + 1, depth - 1, ptd );
		return;
	}
	record[ depth ] = first;
	RecursiveSave( depth + 1, ptd->interval[ first ].to, record, ptd );
	/* for each interval which intersects first */
	for (
		i= first + 1;
		PhraseIntervalIntersect(
			ptd->interval[ first ],
			ptd->interval[ i ] ) && i < ptd->nInterval;
			i++ ) {
		record[ depth ] = i;
		RecursiveSave( depth + 1, ptd->interval[ i ].to,record, ptd );
	}
}

static void SaveList( TreeDataType *ptd )
{
	int record[ MAX_PHONE_SEQ_LEN + 1 ] = { -1 };

	RecursiveSave( 1, 0, record, ptd );
}

static void InitPhrasing( TreeDataType *ptd )
{
	memset( ptd, 0, sizeof( TreeDataType ) );
}

static void SaveDispInterval( PhrasingOutput *ppo, TreeDataType *ptd )
{
	int i;

	for ( i = 0; i < ptd->phList->nInter; i++ ) {
		ppo->dispInterval[ i ].from =
			ptd->interval[ ptd->phList->arrIndex[ i ] ].from;
		ppo->dispInterval[ i ].to =
			ptd->interval[ ptd->phList->arrIndex[ i ] ].to;
	}
	ppo->nDispInterval = ptd->phList->nInter;
}

static void CleanUpMem( TreeDataType *ptd )
{
	int i;
	RecordNode *pNode;

	for ( i = 0; i < ptd->nInterval; i++ ) {
		if ( ptd->interval[ i ].p_phr ) {
			free( ptd->interval[ i ].p_phr );
			ptd->interval[ i ].p_phr = NULL;
		}
	}
	while ( ptd->phList != NULL ) {
		pNode = ptd->phList;
		ptd->phList = pNode->next;
		free( pNode->arrIndex );
		free( pNode );
	}
}

static void CountMatchCnnct( TreeDataType *ptd, const int *bUserArrCnnct, int nPhoneSeq )
{
	RecordNode *p;
	int i, k, sum;

	for ( p = ptd->phList; p; p = p->next ) {
		/* for each record, count its 'nMatchCnnct' */
		for ( sum = 0, i = 1; i < nPhoneSeq; i++ ) {
			if ( !bUserArrCnnct[ i ] )
				continue;
			/* check if matching 'cnnct' */
			for ( k = 0; k < p->nInter; k++ ) {
				if (
					ptd->interval[ p->arrIndex[ k ] ].from < i &&
					ptd->interval[ p->arrIndex[ k ] ].to > i ) {
					sum++;
					break;
				}
			}
		}
		p->nMatchCnnct = sum;
	}
}

static void ShowList( ChewingData *pgdata, const TreeDataType *ptd )
{
	const RecordNode *p;
	int i;

	DEBUG_OUT( "After SaveList :\n" );
	for ( p = ptd->phList; p; p = p->next ) {
		DEBUG_OUT( "  interval : " );
		for ( i = 0; i < p->nInter; i++ ) {
			DEBUG_OUT(
				"[%d %d] ",
				ptd->interval[ p->arrIndex[ i ] ].from,
				ptd->interval[ p->arrIndex[ i ] ].to );
		}
		DEBUG_OUT(
			"\n"
			   "      score : %d , nMatchCnnct : %d\n",
			p->score,
			p->nMatchCnnct );
	}
	DEBUG_OUT( "\n" );
}

static RecordNode* NextCut( TreeDataType *tdt, PhrasingOutput *ppo )
{
	/* pop nNumCut-th candidate to first */
	int i;
	RecordNode *former;
	RecordNode *want;

	if ( ppo->nNumCut >= tdt->nPhListLen )
		ppo->nNumCut = 0;
	if (ppo->nNumCut == 0)
		return tdt->phList;

	/* find the former of our candidate */
	former = tdt->phList;
	for ( i = 0; i < ppo->nNumCut - 1; i++ ) {
		former = former->next;
		assert( former );
	}

	/* take the candidate out of the listed list */
	want = former->next;
	assert( want );
	former->next = former->next->next;

	/* prepend to front of list */
	want->next = tdt->phList;
	tdt->phList = want;

	return tdt->phList;
}

int Phrasing( ChewingData *pgdata )
{
	TreeDataType treeData;

	InitPhrasing( &treeData );

	FindInterval( pgdata, &treeData );
	SetInfo( pgdata->nPhoneSeq, &treeData );
	Discard1( &treeData );
	Discard2( &treeData );
	SaveList( &treeData );
	CountMatchCnnct( &treeData, pgdata->bUserArrCnnct, pgdata->nPhoneSeq );
	SortListByScore( &treeData );
	NextCut( &treeData, &pgdata->phrOut );

	ShowList( pgdata, &treeData );

	/* set phrasing output */
	OutputRecordStr(
		pgdata,
		pgdata->phrOut.chiBuf, sizeof(pgdata->phrOut.chiBuf),
		( treeData.phList )->arrIndex,
		( treeData.phList )->nInter,
		pgdata->phoneSeq,
		pgdata->nPhoneSeq,
		pgdata->selectStr, pgdata->selectInterval, pgdata->nSelect, &treeData );
	SaveDispInterval( &pgdata->phrOut, &treeData );

	/* free "phrase" */
	CleanUpMem( &treeData );
	return 0;
}
