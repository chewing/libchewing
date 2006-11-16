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

/*
  This is a key-sequense map.
  From pinyin sequence to a default-layout sequence.
  Eg: Zhang -> {"zh","5"}, {"ang",";"}
 */
struct keymap {
	char pinyin[7];
	char zuin[4];
};
typedef struct keymap keymap;

typedef enum {
	PINYIN_HANYU,
	PINYIN_EXTERNAL,
	PINYIN_NONE
} PinYinMethodType;


int HanyuPinYinToZuin( char *pinyinKeySeq, char *zuinKeySeq );

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

