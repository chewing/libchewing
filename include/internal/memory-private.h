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

static inline uint16_t GetUint16( const void *ptr )
{
	uint16_t val;
	const unsigned char *uptr = ptr;
#if WORDS_BIGENDIAN
	val =
		( uptr[0] << 8 ) |
		( uptr[1] << 0 );
#else
	val =
		( uptr[0] << 0 ) |
		( uptr[1] << 8 );
#endif
	return val;
}

static inline void PutUint16( uint16_t val, void *ptr )
{
	unsigned char *uptr = (unsigned char *) ptr;
#if WORDS_BIGENDIAN
	uptr[0] = ( val >> 8 ) & 0xff;
	uptr[1] = ( val >> 0 ) & 0xff;
#else
	uptr[0] = ( val >> 0 ) & 0xff;
	uptr[1] = ( val >> 8 ) & 0xff;
#endif
}

static inline int GetInt32( const void *ptr )
{
	int val;
	const unsigned char *uptr = ptr;
#if WORDS_BIGENDIAN
	val =
		( uptr[0] << 24 ) |
		( uptr[1] << 16 ) |
		( uptr[2] <<  8 ) |
		( uptr[3] <<  0 );
#else
	val =
		( uptr[0] <<  0 ) |
		( uptr[1] <<  8 ) |
		( uptr[2] << 16 ) |
		( uptr[3] << 24 );
#endif
	return val;
}

static inline void PutInt32( int val, void *ptr )
{
	unsigned char *uptr = (unsigned char *) ptr;
#if WORDS_BIGENDIAN
	uptr[0] = ( val >> 24 ) & 0xff;
	uptr[1] = ( val >> 16 ) & 0xff;
	uptr[2] = ( val >>  8 ) & 0xff;
	uptr[3] = ( val >>  0 ) & 0xff;
#else
	uptr[0] = ( val >>  0 ) & 0xff;
	uptr[1] = ( val >>  8 ) & 0xff;
	uptr[2] = ( val >> 16 ) & 0xff;
	uptr[3] = ( val >> 24 ) & 0xff;
#endif
}
