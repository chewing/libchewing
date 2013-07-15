/*
 * chewing-utf8-util.h
 *
 * Copyright (c) 2005, 2006, 2008, 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef CHEWING_UTF8_UTILS_H
#define CHEWING_UTF8_UTILS_H

#include <stddef.h>

/* Return length of UTF-8 string */
int ueStrLen( const char *str );

/* Return bytes of a UTF-8 character */
int ueBytesFromChar( unsigned char b );

/* Return byets of a UTF-8 string until len position */
int ueStrNBytes( const char *, int );

#define STRNCPY_CLOSE 1
#define STRNCPY_NOT_CLOSE 0

/*!
 * Return how many bytes was copied
 * @param[out] dest 	The memory area to copy to.
 * @param[in] src 	The memory area to copy from.
 * @param[in] n 	The number to copy.
 * @param[in] end
 */
int ueStrNCpy( char dest[], const char *src, size_t n, int end );

/* Return address from n length after src */
char *ueStrSeek( char *src, size_t n );

/* Const version of ueStrSeek */
const char *ueConstStrSeek( const char *src, size_t n );

/*!
 * Locate a UTF-8 substring from UTF-8 string
 * @param[in] str     UTF-8 string from which substr is located
 * @param[in] lstr    Length of str in bytes.
 * @param[in] substr  UTF-8 string that is located in str.
 * @param[in] lsub    Length of substr in bytes.
 */
const char* ueStrStr( const char *str, size_t lstr,
                      const char *substr, size_t lsub );

#endif /* CHEWING_UTF8_UTILS_H */
