/**
 * @file key2pho-private.h
 *
 * Copyright (c) 2008
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* *INDENT-OFF* */
#ifndef _CHEWING_KEY2PHO_PRIVATE_H
#define _CHEWING_KEY2PHO_PRIVATE_H
/* *INDENT-ON* */

#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#ifdef HAVE_INTTYPES_H
#    include <inttypes.h>
#elif defined HAVE_STDINT_H
#    include <stdint.h>
#endif

#include <sys/types.h>

/* visual C++ does not have ssize_t type */
#if defined(_MSC_VER)
#    include <BaseTsd.h>
typedef SSIZE_T ssize_t;
#endif

#include "chewing-private.h"

/**
 * @brief Get the unsigned 16-bit representation of phonetic symbols.
 *
 * Each valid combination of phonetic symbols is mapped to an unique 16-bit
 * unsigned integer. This function returns the mapped integer of a given
 * combination of phonetic symbols. If the combination is invalid, 0 is
 * returned. For example: `UintFromPhone("ㄎㄨˋ")` returns 5380 and
 * `UintFromPhone("ㄧㄣ")` returns 208, while `UintFromPhone("ㄎㄨㄨ")`
 * returns 0.
 *
 * @param[in] phone UTF-8 encoded string of phonetic symbols.
 * @return an 16-bit unsigned integer or 0 if the phonetic symbols are illegal.
 */
uint16_t UintFromPhone(const char *phone);

/**
 * @brief Get the unsigned 16-bit representation of phonetic symbols by index.
 *
 * The argument `ph_inx` is an array of the index of phonetic symbols. Each
 * phonetic symbol is categorized into one of the four categories:
 *
 * 1. Consonants: ㄅㄆㄇㄈㄉㄊㄋㄌㄍㄎㄏㄐㄑㄒㄓㄔㄕㄖㄗㄘㄙ
 * 2. Medials: ㄧㄨㄩ
 * 3. Rhymes: ㄚㄛㄜㄝㄞㄟㄠㄡㄢㄣㄤㄥㄦ
 * 4. Tonal marks: ˙ˊˇˋ
 *
 * The index of a phonetic symbol is defined by the order in its corresponding
 * category starting from 1.
 *
 * The argument `ph_inx` should be of size 4. Each element is the index of the
 * four categories respectively. If some categories are not used, the
 * index should be 0.
 *
 * @param[in] ph_inx array of index of phonetic symbols.
 * @return an 16-bit unsigned integer or 0 if any index is illegal.
 */
uint16_t UintFromPhoneInx(const int ph_inx[]);

/**
 * @brief Get the phonetic symbols by the given keystroke.
 * @param[out] pho pointer to destination string.
 * @param[in] inputkey string of keystroke.
 * @param[in] kbtype keyboard type. See \ref KBTYPE.
 * @param[in] searchTimes times to search.
 * @return 1 if succeed or 0 if failed.
 */
int PhoneFromKey(char *pho, const char *inputkey, KBTYPE kbtype, int searchTimes);

/**
 * @brief Get the string of phonetic symbols by its phonetic number.
 * @param[out] phone destination string.
 * @param[in] phone_len the length of the destination string.
 * @param[in] phone_num phonetic number.
 * @return 1 if succeed or 0 if failed.
 */
int PhoneFromUint(char *phone, size_t phone_len, uint16_t phone_num);

/**
 * @brief Get the index of a phonetic symbols in its category.
 * @param[out]
 * @param[in]
 * @param[in]
 * @return
 */
int PhoneInxFromKey(int key, int type, KBTYPE kbtype, int searchTimes);

/**
 * @brief
 * @param[out]
 * @param[in]
 * @param[in]
 * @return
 */
size_t BopomofoFromUintArray(char *const bopomofo_buf, const size_t bopomofo_len, const uint16_t *phoneSeq);

/**
 * @brief
 * @param[out]
 * @param[in]
 * @param[in]
 * @return
 */
ssize_t UintArrayFromBopomofo(uint16_t *phone_seq, const size_t phone_len, const char *bopomofo_buf);

/**
 * @brief Get the length of the array of phones.
 *
 * The length is defined by the number of consecutive non-0 elements from the
 * beginning of the array.
 *
 * @param[in] phoneSeq pointer to the array.
 * @return the length of the array of phones.
 */
size_t GetPhoneLen(const uint16_t *phoneSeq);

/**
 * @brief Get the length of bopomofo buffer needed in bytes by the given number
 * of chinese characters.
 *
 * @param[in] len the number of chinese characters.
 * @return the length of bopomofo buffer in bytes.
 */
size_t GetBopomofoBufLen(size_t len);

/* *INDENT-OFF* */
#endif
/* *INDENT-ON* */
