/**
 * memory.c
 *
 * Copyright (c) 2013
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#include "memory-private.h"

#include <assert.h>
#include <string.h>

uint16_t GetUint16( const char *ptr )
{
	uint16_t val;
	assert( ptr );
	memcpy( &val, ptr, sizeof( val ) );
	return val;
}

void PutUint16( uint16_t val, char *ptr )
{
	assert(ptr);
	memcpy( ptr, &val, sizeof( val ) );
}

int GetInt32( const char *ptr )
{
	int val;
	assert( ptr );
	memcpy( &val, ptr, sizeof( val ) );
	return val;
}

void PutInt32( int val, char *ptr )
{
	assert( ptr );
	memcpy( ptr, &val, sizeof( val ) );
}
