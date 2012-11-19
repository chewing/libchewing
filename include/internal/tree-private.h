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

int InitTree( ChewingData *pgdata, const char *prefix );
void TerminateTree( ChewingData *pgdata );

int Phrasing( ChewingData *pgdata );
int IsIntersect( IntervalType in1, IntervalType in2 );

int TreeFindPhrase( ChewingData *pgdata, int begin, int end, const uint16_t *phoneSeq );

#endif
