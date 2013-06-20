/**
 * chewing-private.h
 *
 * Copyright (c) 2008, 2010
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_CORE_PRIVATE_H
#define _CHEWING_CORE_PRIVATE_H

#ifdef HAVE_CONFIG_H
#  include <config.h>
#endif

#ifdef HAVE_INTTYPES_H
#  include <inttypes.h>
#elif defined HAVE_STDINT_H
#  include <stdint.h>
#endif

#ifndef USE_BINARY_DATA
#include <stdio.h>
#endif

#include "global.h"
#include "plat_mmap.h"

#define MAX_KBTYPE 13
#define MAX_UTF8_SIZE 6
#define ZUIN_SIZE 4
#define PINYIN_SIZE 10
#define MAX_PHRASE_LEN 11
#define MAX_PHONE_SEQ_LEN 50
#define MIN_CHI_SYMBOL_LEN 0
#define MAX_CHI_SYMBOL_LEN (MAX_PHONE_SEQ_LEN - MAX_PHRASE_LEN)
#define MAX_INTERVAL ( ( MAX_PHONE_SEQ_LEN + 1 ) * MAX_PHONE_SEQ_LEN / 2 )
#define MAX_CHOICE (567)
#define MAX_CHOICE_BUF (50)                   /* max length of the choise buffer */
#define N_HASH_BIT (14)
#define HASH_TABLE_SIZE (1<<N_HASH_BIT)
#define EASY_SYMBOL_KEY_TAB_LEN (36)

/* For isSymbol */
#define WORD_CHOICE            (0)
#define SYMBOL_CATEGORY_CHOICE (1)
#define SYMBOL_CHOICE_INSERT   (2)
#define SYMBOL_CHOICE_UPDATE   (3)

#ifndef _MSC_VER
#undef max
static inline int max( int a, int b )
{
	return a > b ? a : b;
}

#undef min
static inline int min( int a, int b )
{
	return a < b ? a : b;
}
#endif

typedef union {
	unsigned char s[ MAX_UTF8_SIZE + 1];
	uint16_t wch;
} wch_t;

typedef struct {
	uint16_t phone_id;
	int phrase_id;
	int child_begin, child_end;
} TreeType;

typedef struct {
	char chiBuf[ MAX_PHONE_SEQ_LEN * MAX_UTF8_SIZE + 1 ];
	IntervalType dispInterval[ MAX_INTERVAL ];
	int nDispInterval;
	int nNumCut;
} PhrasingOutput;

typedef struct {
    int type;
    char keySeq[ PINYIN_SIZE ];
} PinYinData;

typedef struct {
	int kbtype;
	int pho_inx[ ZUIN_SIZE ];
	int pho_inx_alt[ ZUIN_SIZE ];
	uint16_t phone;
	uint16_t phoneAlt;
	PinYinData pinYinData;
} ZuinData;

typedef struct {
	/** @brief all kinds of lengths of available phrases. */
	struct {
		int len;
		/** @brief phone id. */
		int id;
	} avail[ MAX_PHRASE_LEN ];  
	/** @brief total number of availble lengths. */
	int nAvail;
	/** @brief the current choosing available length. */
	int currentAvail;
} AvailInfo;
/**
 *	@struct AvailInfo
 *	@brief information of available phrases or characters choices.
 */

typedef struct {
	/** @brief total page number. */
	int nPage;
	/** @brief current page number. */
	int pageNo;
	/** @brief number of choices per page. */
	int nChoicePerPage;
	/** @brief store possible phrases for being chosen. */
	char totalChoiceStr[ MAX_CHOICE ][ MAX_PHRASE_LEN * MAX_UTF8_SIZE + 1 ];
	/** @brief number of phrases to choose. */
	int nTotalChoice;
	int oldChiSymbolCursor;
	int isSymbol;
} ChoiceInfo;

/** @brief entry of symbol table */
typedef struct _SymbolEntry {
	/** @brief  nSymnols is total number of symbols in this category.
	 * If nSymbols = 0, category is treat as a symbol, 
	 * which is a zero-terminated utf-8 string. 
	 * In that case, symbols[] is unused and isn't allocated at all.
	 */
	int nSymbols;

	/** @brief  Category name of these symbols */
	char category[ MAX_PHRASE_LEN * MAX_UTF8_SIZE + 1 ];

	/** @brief  Symbols in this category.
	 * This is an char[] array of variable length.
	 * When nSymbols = 0, this array is not allocated.
	 */
	char symbols[][ MAX_UTF8_SIZE + 1 ];
} SymbolEntry;

typedef struct {
	TreeType *tree;
	size_t tree_size;
#ifdef USE_BINARY_DATA
	plat_mmap tree_mmap;
#endif

	uint16_t *arrPhone;
	int *char_begin;
	size_t phone_num;
	void *char_;
	void *char_cur_pos;
	int char_end_pos;
#ifdef USE_BINARY_DATA
	plat_mmap char_mmap;
	plat_mmap char_begin_mmap;
	plat_mmap char_phone_mmap;
#else
	FILE *charfile;
#endif

	int *dict_begin;
	void *dict_cur_pos;
	int dict_end_pos;

	void *dict;

#ifdef USE_BINARY_DATA
	plat_mmap dict_mmap;
	plat_mmap index_mmap;
#else
	FILE *dictfile;
#endif


	int chewing_lifetime;

	char hashfilename[ 200 ];
	struct tag_HASH_ITEM *hashtable[ HASH_TABLE_SIZE ];

	unsigned int n_symbol_entry;
	SymbolEntry ** symbol_table;

	char *g_easy_symbol_value[ EASY_SYMBOL_KEY_TAB_LEN ];
	int g_easy_symbol_num[ EASY_SYMBOL_KEY_TAB_LEN ];

	struct keymap *hanyuInitialsMap;
	struct keymap *hanyuFinalsMap;
	int HANYU_INITIALS;
	int HANYU_FINALS;
} ChewingStaticData;

struct tag_HASH_ITEM;

typedef struct tag_ChewingData {
	AvailInfo availInfo;
	ChoiceInfo choiceInfo;
	PhrasingOutput phrOut;
	ZuinData zuinData;
	ChewingConfigData config;
    /** @brief current input buffer, content==0 means Chinese code */
	wch_t chiSymbolBuf[ MAX_PHONE_SEQ_LEN ];
	int chiSymbolCursor;
	int chiSymbolBufLen;
	int PointStart;
	int PointEnd;
	wch_t showMsg[ MAX_PHONE_SEQ_LEN ];
	int showMsgLen;

	uint16_t phoneSeq[ MAX_PHONE_SEQ_LEN ];
	uint16_t phoneSeqAlt[ MAX_PHONE_SEQ_LEN ];
	int nPhoneSeq;
	char selectStr[ MAX_PHONE_SEQ_LEN ][ MAX_PHONE_SEQ_LEN * MAX_UTF8_SIZE + 1 ];
	IntervalType selectInterval[ MAX_PHONE_SEQ_LEN ];
	int nSelect;
	IntervalType preferInterval[ MAX_INTERVAL ]; /* add connect points */
	int nPrefer;
	int bUserArrCnnct[ MAX_PHONE_SEQ_LEN + 1 ];
	int bUserArrBrkpt[ MAX_PHONE_SEQ_LEN + 1 ];   
	int bArrBrkpt[ MAX_PHONE_SEQ_LEN + 1 ];
	int bSymbolArrBrkpt[ MAX_PHONE_SEQ_LEN + 1 ];
	/* "bArrBrkpt[10]=True" means "it breaks between 9 and 10" */
	int bChiSym, bSelect, bFirstKey, bFullShape;
	/* Symbol Key buffer */
	char symbolKeyBuf[ MAX_PHONE_SEQ_LEN ];

	struct tag_HASH_ITEM *prev_userphrase;
	ChewingStaticData static_data;
	void (*logger)( void *data, int level, const char *fmt, ... );
	void *loggerData;
} ChewingData;

typedef struct {
	/** @brief the content of Edit buffer. */
	wch_t chiSymbolBuf[ MAX_PHONE_SEQ_LEN ];
	/** @brief the length of Edit buffer. */
	int chiSymbolBufLen;
	/** @brief current position of the cursor. */
	long chiSymbolCursor;
	long PointStart;
	long PointEnd;
	/** @brief the zuin-yin symbols have already entered. */
	wch_t zuinBuf[ ZUIN_SIZE ];
	/** @brief indicate the method of showing sentence break. */
	IntervalType dispInterval[ MAX_INTERVAL ]; /* from prefer, considering symbol */
	int nDispInterval;
	/** @brief indicate the break points going to display.*/ 
	int dispBrkpt[ MAX_PHONE_SEQ_LEN + 1 ];
	/** @brief the string going to commit. */
	wch_t commitStr[ MAX_PHONE_SEQ_LEN ];
	int nCommitStr;
	/** @brief information of character selections. */
	ChoiceInfo* pci;
	/** @brief indicate English mode or Chinese mode. */
	int bChiSym;
	int selKey[ MAX_SELKEY ];
	/** @brief return value. */
	int keystrokeRtn;
	int bShowMsg; 
	/** @brief user message. */
	wch_t showMsg[ MAX_PHONE_SEQ_LEN ];
	int showMsgLen;
} ChewingOutput;
/**
 *   @struct ChewingOutput
 *   @brief  information for Chewing output.
 */

struct _ChewingContext {
	ChewingData *data;
	ChewingOutput *output;
	int cand_no;
	int it_no;
	int kb_no;
};
/**
 * @struct ChewingContext
 * @brief context of Chewing IM
 */

typedef struct {
	char phrase[ MAX_PHRASE_LEN * MAX_UTF8_SIZE + 1 ];
	int freq;
} Phrase;

#endif
