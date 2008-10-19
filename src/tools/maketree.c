/**
 * maketree.c
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/**
 * @file maketree.c
 *
 * @brief Phone phrase tree generator.\n
 *
 *	  This program reads in a dictionary with phone phrases(in uint16 form).\n
 *	  Output a database file which indicates a phone phrase tree.\n
 *	  Each node represents a single phone.\n
 *	  The output file was a random access file, a record was defined:\n\code
 *	  { 
 *		 uint16 key; the phone data
 *		 int32 phraseno; 
 *		 int32 begin,end; //the children of this node(-1,-1 indicate a leaf node)
 *	  }\endcode
 */

#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include "global.h"
#include "global-private.h"
#include "chewing-private.h"
#include "config.h"

/*
	defines
*/
#define MAX_PH_NODE	6400000
#define IN_FILE		"phoneid.dic"

/* 
	typedefs 
*/
typedef int int32;

/* Forward declaration */
struct _tLISTNODE;

typedef struct _tNODE {
	struct _tLISTNODE *childList;
	uint16 key;
	int32 phraseno,nodeno;
} NODE;

typedef struct _tLISTNODE {
	struct _tNODE *pNode;
	struct _tLISTNODE *next;
} LISTNODE;

/*
	global data
 */
NODE *root;
int32 ph_count;

NODE *queue[ MAX_PH_NODE ];
int head, tail;

int node_count;
int tree_size;

void QueuePut( NODE *pN )
{
	queue[ head++ ] = pN;
	if ( head == MAX_PH_NODE ) {
		fprintf( stderr, "Queue size is not enough!\n" );
		exit( 1 );
	}
}

NODE* QueueGet()
{
	assert( head != tail );
	return queue[ tail++ ];
}

int QueueEmpty()
{
	return ( head == tail );
}

NODE *NewNode( uint16 key )
{
	NODE *pnew = (NODE *) malloc( sizeof( NODE ) );
	pnew->key = key;
	pnew->childList = NULL;
	pnew->phraseno = -1;
	pnew->nodeno = -1;
	return pnew;
}

void InitConstruct()
{
	/* root has special key value 0 */
	root = NewNode( 0 );
}

NODE* FindKey( NODE *pN, uint16 key )
{
	LISTNODE *p;

	for ( p = pN->childList; p; p = p->next ) {
		if ( p->pNode->key == key )
			return p->pNode;
	}
	return NULL;
}

NODE* Insert( NODE *pN, uint16 key )
{
	LISTNODE *prev, *p;
	LISTNODE *pnew = (LISTNODE *) malloc( sizeof( LISTNODE ) );
	NODE *pnode = NewNode( key );
	
	pnew->pNode = pnode;
	pnew->next  = NULL;

	prev = pN->childList;
	if ( ! prev ) {
		pN->childList = pnew;
	}
	else {
		for ( 
			p = prev->next;
			(p) && (p->pNode->key < key);
			prev = p, p = p->next )
			;
		prev->next = pnew;
		pnew->next = p;
	}
	return pnode;
}

void Construct()
{
	FILE *input = fopen( IN_FILE, "r" );
	NODE *pointer, *tp;
	uint16 key;

	if ( ! input ) {
		fprintf( stderr, "Error opening " IN_FILE "\n" );
		exit( 1 );
	}
	InitConstruct();
	
	while ( 1 ) {	
		fscanf( input, "%hu", &key );
		if ( feof( input ) )
			break;
		pointer = root;
		/* for each phone in a phone phrase */
		for ( ; key != 0; fscanf( input, "%hu", &key ) ) {	
			if ( ( tp = FindKey( pointer, key ) ) ) {
				pointer = tp;
			}
			else {
				tp = Insert( pointer, key );
				pointer = tp;
			}
		}
		pointer->phraseno = ph_count++;
	}
}
/* Give the level-order travel number to each node */
void BFS1()
{
	NODE *pNode;
	LISTNODE *pList;

	QueuePut(root);
	while ( ! QueueEmpty() ) {
		pNode = QueueGet();
		pNode->nodeno = node_count++;

		for ( pList = pNode->childList; pList; pList = pList->next ) {
			QueuePut( pList->pNode );
		}
	}
}

void BFS2()
{
	NODE *pNode;
	LISTNODE *pList;
	TreeType tree = {0, 0, 0, 0};
#ifdef USE_BINARY_DATA
	FILE *output = fopen( PHONE_TREE_FILE, "wb" );
#else
	FILE *output = fopen( PHONE_TREE_FILE, "w" );
#endif
	FILE *config = fopen( CHEWING_DEFINITION_FILE, "a" );

	if ( ! output ) {
		fprintf( stderr, "Error opening file " PHONE_TREE_FILE " for output.\n" );
		exit( 1 );
	}

	if ( ! config ) {
		fprintf( stderr, "Error opening file " CHEWING_DEFINITION_FILE " for output.\n" );
		exit( 1 );
	}

	QueuePut( root );
	tree_size = 0;
	while ( ! QueueEmpty() ) {
		pNode = QueueGet();
		
		tree.phone_id = pNode->key;
		tree.phrase_id = pNode->phraseno;

		/* compute the begin and end index */
		pList = pNode->childList;
		if( pList ) {
			tree.child_begin = pList->pNode->nodeno;

			for ( ; pList->next; pList = pList->next ) {
				QueuePut( pList->pNode );
			}
			QueuePut( pList->pNode );
			tree.child_end = pList->pNode->nodeno;
		}
		else {
			tree.child_begin = -1;
			tree.child_end = -1;
		}
#ifdef USE_BINARY_DATA
		fwrite( &tree, sizeof(TreeType), 1, output );
#else
		fprintf( output, "%hu %d %d %d\n",
				tree.phone_id, tree.phrase_id,
				tree.child_begin, tree.child_end );
#endif
		tree_size++;
	}
	fprintf( config, "#define TREE_SIZE (%d)\n", tree_size );
	fclose( output );
	fclose( config );
}

int main()
{
	Construct();
	BFS1();		
	BFS2();

	return 0;
}

