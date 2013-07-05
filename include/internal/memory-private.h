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

#ifdef _MSC_VER
#define inline __inline
#endif

static inline uint16_t GetUint16( const char *ptr )
{
	uint16_t val;
#if WORDS_BIGENDIAN
	val =
		( ptr[0] << 8 ) |
		( ptr[1] << 0 );
#else
	val =
		( ptr[0] << 0 ) |
		( ptr[1] << 8 );
#endif
	return val;
}

static inline void PutUint16( uint16_t val, char *ptr )
{
#if WORDS_BIGENDIAN
	ptr[0] = ( val >> 8 ) & 0xff;
	ptr[1] = ( val >> 0 ) & 0xff;
#else
	ptr[0] = ( val >> 0 ) & 0xff;
	ptr[1] = ( val >> 8 ) & 0xff;
#endif
}

static inline int GetInt32( const char *ptr )
{
	int val;
#if WORDS_BIGENDIAN
	val =
		( ptr[0] << 24 ) |
		( ptr[1] << 16 ) |
		( ptr[2] <<  8 ) |
		( ptr[3] <<  0 );
#else
	val =
		( ptr[0] <<  0 ) |
		( ptr[1] <<  8 ) |
		( ptr[2] << 16 ) |
		( ptr[3] << 24 );
#endif
	return val;
}

static inline void PutInt32( int val, char *ptr )
{
#if WORDS_BIGENDIAN
	ptr[0] = ( val >> 24 ) & 0xff;
	ptr[1] = ( val >> 16 ) & 0xff;
	ptr[2] = ( val >>  8 ) & 0xff;
	ptr[3] = ( val >>  0 ) & 0xff;
#else
	ptr[0] = ( val >>  0 ) & 0xff;
	ptr[1] = ( val >>  8 ) & 0xff;
	ptr[2] = ( val >> 16 ) & 0xff;
	ptr[3] = ( val >> 24 ) & 0xff;
#endif
}
