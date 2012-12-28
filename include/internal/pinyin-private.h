/**
 * pinyin-private.h
 *
 * Copyright (c) 2008
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* @(#)pinyin-private.h
 */

#ifndef _CHEWING_PINYIN_PRIVATE_H
#define _CHEWING_PINYIN_PRIVATE_H

#include "chewing-private.h"

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

int HanyuPinYinToZuin( ChewingData *pgdata, char *pinyinKeySeq, char *zuinKeySeq );
int InitHanyuPinYin( ChewingData *pgdata, const char * );
void TerminateHanyuPinyin( ChewingData *pgdata );

#endif
