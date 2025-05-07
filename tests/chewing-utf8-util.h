/*
 * chewing-utf8-util.h
 *
 * Copyright (c) 2005, 2006, 2008, 2012
 *      libchewing Core Team.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* *INDENT-OFF* */
#ifndef CHEWING_UTF8_UTILS_H
#define CHEWING_UTF8_UTILS_H
/* *INDENT-ON* */

#include <stddef.h>

/**
 * @brief Get the number of characters in UTF-8 string.
 *
 * For example: `ueStrLen("新酷音")` returns 3.
 *
 * @param[in] str the UTF-8 string.
 * @return the number of characters in this UTF-8 string.
 */
int ueStrLen(const char *str);

/**
 * @brief Get the number of bytes in the sequence of a UTF-8 character.
 * @param[in] b the leading byte of a UTF-8 character.
 * @return the number of bytes in the sequence of a UTF-8 character.
 */
int ueBytesFromChar(unsigned char b);

#define STRNCPY_CLOSE 1
#define STRNCPY_NOT_CLOSE 0

/**
 * @brief Copy UTF-8 characters.
 *
 * Copies the first `n` UTF-8 characters from `src` to `dest`.
 *
 * The argument `end` takes two values: `STRNCPY_CLOSE` or
 * `STRNCPY_NOT_CLOSE`. `STRNCPY_CLOSE` will terminate `dest` with '\0',
 * while `STRNCPY_NOT_CLOSE` will not.
 *
 * @param[out] dest the memory area to copy to.
 * @param[in]  src  the memory area to copy from.
 * @param[in]  n    the number of characters to copy.
 * @param[in]  end  STRNCPY_CLOSE or STRNCPY_NOT_CLOSE.
 * @return the number of bytes copied.
 */
int ueStrNCpy(char dest[], const char *src, size_t n, int end);

/**
 * @brief Get the pointer to the nth UTF-8 character. (0-based)
 * @param[in] src the UTF-8 string.
 * @param[in] n   the nth character.
 * @return a pointer to the first byte of the character.
 */
char *ueStrSeek(char *src, size_t n);

/* *INDENT-OFF* */
#endif                          /* CHEWING_UTF8_UTILS_H */
/* *INDENT-ON* */
