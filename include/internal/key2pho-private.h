/**
 * key2pho-private.h
 *
 * Copyright (c) 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_KEY2PHO_PRIVATE_H
#define _CHEWING_KEY2PHO_PRIVATE_H

#ifdef HAVE_CONFIG_H
#  include <config.h>
#endif

#ifdef HAVE_INTTYPES_H
#  include <inttypes.h>
#elif defined HAVE_STDINT_H
#  include <stdint.h>
#endif

#include <sys/types.h>

/* visual C++ does not have ssize_t type */
#if defined(_MSC_VER)
#include <BaseTsd.h>
typedef SSIZE_T ssize_t;
#endif

uint16_t UintFromPhone( const char *phone );
uint16_t UintFromPhoneInx( const int ph_inx[] );
int PhoneFromKey( char *pho, const char *inputkey, int kbtype, int searchTimes );
int PhoneFromUint( char *phone, size_t phone_len, uint16_t phone_num );
int PhoneInxFromKey( int key, int type, int kbtype, int searchTimes );
size_t BopomofoFromUintArray( char * const bopomofo_buf, const size_t bopomofo_len, const uint16_t *phoneSeq );
ssize_t UintArrayFromBopomofo( uint16_t *phone_seq, const size_t phone_len, const char * bopomofo_buf );
size_t GetPhoneLen( const uint16_t *phoneSeq );
size_t GetBopomofoBufLen( size_t len );

#endif
