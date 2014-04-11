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

/* *INDENT-OFF* */
#ifndef _CHEWING_PINYIN_PRIVATE_H
#define _CHEWING_PINYIN_PRIVATE_H
/* *INDENT-ON* */

#include "chewing-private.h"

/*
  This is a key-sequense map.
  From pinyin sequence to a default-layout sequence.
  Eg: Zhang -> {"zh","5"}, {"ang",";"}
 */
typedef struct keymap {
    char pinyin[7];
    char bopomofo[4];
} keymap;

int PinyinToBopomofo(ChewingData *pgdata, const char *pinyinKeySeq, char *bopomofoKeySeq, char *bopomofoKeySeqAlt);
int InitPinyin(ChewingData *pgdata, const char *);
void TerminatePinyin(ChewingData *pgdata);

/* *INDENT-OFF* */
#endif
/* *INDENT-ON* */
