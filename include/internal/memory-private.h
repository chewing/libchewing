/**
 * memory-private.h
 *
 * Copyright (c) 2013
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#  include <config.h>
#endif

#ifdef HAVE_INTTYPES_H
#  include <inttypes.h>
#elif defined HAVE_STDINT_H
#  include <stdint.h>
#endif

uint16_t GetUint16( const char *ptr );
void PutUint16( uint16_t val, char *ptr );

int GetInt32( const char *ptr );
void PutInt32( int val, char *ptr );
