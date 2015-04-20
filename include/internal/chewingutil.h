/**
 * chewingutil.h
 *
 * Copyright (c) 1999, 2000, 2001
 *      Lu-chuan Kung and Kang-pen Chen.
 *      All rights reserved.
 *
 * Copyright (c) 2004, 2005, 2006, 2010
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* *INDENT-OFF* */
#ifndef _CHEWING_UTIL_H
#define _CHEWING_UTIL_H
/* *INDENT-ON* */

#include "chewing-private.h"

#define SYMBOL_KEY_OK 0
#define SYMBOL_KEY_ERROR 1
#define DECREASE_CURSOR 1
#define NONDECREASE_CURSOR 0

void AutoLearnPhrase(ChewingData *pgdata);
void SetUpdatePhraseMsg(ChewingData *pgdata, const char *addWordSeq, int len, int state);
int NoSymbolBetween(ChewingData *pgdata, int begin, int end);
int ChewingIsEntering(ChewingData *pgdata);
void CleanAllBuf(ChewingData *);
int SpecialSymbolInput(int key, ChewingData *pgdata);
int FullShapeSymbolInput(int key, ChewingData *pgdata);
int EasySymbolInput(int key, ChewingData *pgdata);
int SymbolInput(int key, ChewingData *pgdata);
int SymbolChoice(ChewingData *pgdata, int sel_i);
int HaninSymbolInput(ChewingData *pgdata);
void WriteChiSymbolToCommitBuf(ChewingData *pgdata, ChewingOutput *pgo, int len);
int ReleaseChiSymbolBuf(ChewingData *pgdata, ChewingOutput *);
int AddChi(uint16_t phone, uint16_t phoneAlt, ChewingData *pgdata);
int CallPhrasing(ChewingData *pgdata, int all_phrasing);
int MakeOutputWithRtn(ChewingOutput *pgo, ChewingData *pgdata, int keystrokeRtn);
int MakeOutput(ChewingOutput *pgo, ChewingData *pgdata);
void MakeOutputAddMsgAndCleanInterval(ChewingOutput *pgo, ChewingData *pgdata);
int AddSelect(ChewingData *pgdata, int sel_i);
int CountSelKeyNum(int key, const ChewingData *pgdata);
int CountSymbols(ChewingData *pgdata, int to);
int PhoneSeqCursor(ChewingData *pgdata);
int ChewingIsChiAt(int cursor, ChewingData *pgdata);
int ChewingKillChar(ChewingData *pgdata, int chiSymbolCursorToKill, int minus);
void RemoveSelectElement(int i, ChewingData *pgdata);
int IsPreferIntervalConnted(int cursor, ChewingData *pgdata);
int OpenSymbolChoice(ChewingData *pgdata);

int InitSymbolTable(ChewingData *pgdata, const char *prefix);
void TerminateSymbolTable(ChewingData *pgdata);

int InitEasySymbolInput(ChewingData *pgdata, const char *prefix);
void TerminateEasySymbolTable(ChewingData *pgdata);
void copyStringFromPreeditBuf(ChewingData *pgdata, int pos, int len, char *output, int output_len);
int toPreeditBufIndex(ChewingData *pgdata, int pos);

/* *INDENT-OFF* */
#endif
/* *INDENT-ON* */
