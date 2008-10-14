/**
 * hanyupinyin.h 
 *
 * Copyright (c) 2005, 2006
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* @(#)hanyupinyin.h
 */

#ifndef _HANYUPINYIN_H
#define _HANYUPINYIN_H

#include "global.h"

typedef enum {
	PINYIN_HANYU,
	PINYIN_EXTERNAL,
	PINYIN_NONE
} PinYinMethodType;

/**
 * @breif Set PinYin input method
 *
 * @param methodType the method type of PinYin
 * @retval 0 if succeed
 */
CHEWING_API int chewing_set_PinYinMethod(
		const PinYinMethodType methodType,
		const char* filePath );

#endif /* _HANYUPINYIN_H */

