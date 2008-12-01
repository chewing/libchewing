/**
 * tree-private.h
 *
 * Copyright (c) 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_TREE_PRIVATE_H
#define _CHEWING_TREE_PRIVATE_H

#define IS_USER_PHRASE 1
#define IS_DICT_PHRASE 0

void InitTree( const char * );
int Phrasing( PhrasingOutput *ppo, uint16 phoneSeq[], int nPhoneSeq, 
		char selectStr[][ MAX_PHONE_SEQ_LEN * MAX_UTF8_SIZE + 1 ], 
		IntervalType selectInterval[], int nSelect, 
		int bArrBrkpt[], int bUserArrCnnct[] );
int IsIntersect( IntervalType in1, IntervalType in2 );

int TreeFindPhrase( int begin, int end, const uint16 *phoneSeq );

#endif
