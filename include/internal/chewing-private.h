/**
 * chewing-private.h
 *
 * Copyright (c) 2008, 2010
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* *INDENT-OFF* */
#ifndef _CHEWING_CORE_PRIVATE_H
#define _CHEWING_CORE_PRIVATE_H
/* *INDENT-ON* */

#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#ifdef HAVE_INTTYPES_H
#    include <inttypes.h>
#elif defined HAVE_STDINT_H
#    include <stdint.h>
#endif

/* visual C++ does not have ssize_t type */
#if defined(_MSC_VER)
#    include <BaseTsd.h>
typedef SSIZE_T ssize_t;
#endif

#include "global.h"
#include "plat_mmap.h"

#include "userphrase-private.h"
#if WITH_SQLITE3
#    include "sqlite3.h"
#    include "chewing-sql.h"
#endif

#define MAX_UTF8_SIZE 4
#define BOPOMOFO_SIZE 4
#define PINYIN_SIZE 10
#define MAX_PHRASE_LEN 11
#define MAX_PHONE_SEQ_LEN 50
#define MIN_CHI_SYMBOL_LEN 0
#define MAX_CHI_SYMBOL_LEN (MAX_PHONE_SEQ_LEN - MAX_PHRASE_LEN)
#define MAX_INTERVAL ( ( MAX_PHONE_SEQ_LEN + 1 ) * MAX_PHONE_SEQ_LEN / 2 )
#define MAX_CHOICE (567)
#define MAX_CHOICE_BUF (50)     /* max length of the choise buffer */
#define N_HASH_BIT (14)
#define HASH_TABLE_SIZE (1<<N_HASH_BIT)
#define EASY_SYMBOL_KEY_TAB_LEN (36)
#define AUX_PREFIX_LEN (3)

/* For isSymbol */
#define WORD_CHOICE            (0)
#define SYMBOL_CATEGORY_CHOICE (1)
#define SYMBOL_CHOICE_INSERT   (2)
#define SYMBOL_CHOICE_UPDATE   (3)

#ifndef _MSC_VER
#    undef max
static inline int max(int a, int b)
{
    return a > b ? a : b;
}

#    undef min
static inline int min(int a, int b)
{
    return a < b ? a : b;
}
#endif

typedef enum KBTYPE {
    KBTYPE_STANDARD,
    KBTYPE_HSU,
    KBTYPE_IBM,
    KBTYPE_GIN_YIEH,
    KBTYPE_ET,
    KBTYPE_ET26,
    KBTYPE_DVORAK,
    KBTYPE_DVORAK_HSU,
    KBTYPE_DACHEN_CP26,
    KBTYPE_HANYU_PINYIN,
    KBTYPE_LUOMA_PINYIN,
    KBTYPE_MSP2,            /* Mandarin Phonetic Symbols II */
    KBTYPE_CARPALX,
    KBTYPE_COUNT
} KBTYPE;

/**
 * @struct TreeType
 * @brief node type of the system index tree
 *
 * This structure may represent both internal nodes and leaf nodes of a phrase
 * tree. Two kinds are distinguished by whether key is 0. For an internal node,
 * child.begin and child.end give a list of children in the position
 * [child.begin, child.end). For a leaf node, phrase.pos offers the position
 * of the phrase in system dictionary, and phrase.freq offers frequency of this
 * phrase using a specific input method (may be bopomofo or non-phone). Note
 * that key in root represents the number of total elements(nodes) in the tree.
 */

typedef struct TreeType {
    unsigned char key[2];
    union {
        struct {
            unsigned char begin[3];
            unsigned char end[3];
        } child;
        struct {
            unsigned char pos[3];
            unsigned char freq[3];
        } phrase;
    };
} TreeType;

typedef struct PhrasingOutput {
    IntervalType dispInterval[MAX_INTERVAL];
    int nDispInterval;
    int nNumCut;
} PhrasingOutput;

typedef struct PinYinData {
    int type;
    char keySeq[PINYIN_SIZE];
} PinYinData;

typedef struct BopomofoData {
    int kbtype;
    int pho_inx[BOPOMOFO_SIZE];
    int pho_inx_alt[BOPOMOFO_SIZE];
    uint16_t phone;
    uint16_t phoneAlt;
    PinYinData pinYinData;
} BopomofoData;

/**
 * @struct AvailInfo
 * @brief information of available phrases or characters choices.
 */

typedef struct AvailInfo {
        /** @brief all kinds of lengths of available phrases. */
    struct {
        int len;
                /** @brief phone id. */
        const TreeType *id;
    } avail[MAX_PHRASE_LEN];
        /** @brief total number of available lengths. */
    int nAvail;
        /** @brief the current choosing available length. */
    int currentAvail;
} AvailInfo;

typedef struct ChoiceInfo {
        /** @brief total page number. */
    int nPage;
        /** @brief current page number. */
    int pageNo;
        /** @brief number of choices per page. */
    int nChoicePerPage;
        /** @brief store possible phrases for being chosen. */
    char totalChoiceStr[MAX_CHOICE][MAX_PHRASE_LEN * MAX_UTF8_SIZE + 1];
        /** @brief number of phrases to choose. */
    int nTotalChoice;
    int oldChiSymbolCursor;
    int isSymbol;
} ChoiceInfo;

/**
 * @struct SymbolEntry
 * @brief entry of symbol table
 */
typedef struct SymbolEntry {
        /** @brief  nSymnols is total number of symbols in this category.
         * If nSymbols = 0, category is treat as a symbol,
         * which is a zero-terminated utf-8 string.
         * In that case, symbols[] is unused and isn't allocated at all.
         */
    int nSymbols;

        /** @brief  Category name of these symbols */
    char category[MAX_PHRASE_LEN * MAX_UTF8_SIZE + 1];

        /** @brief  Symbols in this category.
         * This is an char[] array of variable length.
         * When nSymbols = 0, this array is not allocated.
         */
    char symbols[][MAX_UTF8_SIZE + 1];
} SymbolEntry;

typedef struct ChewingStaticData {
    const TreeType *tree;
    size_t tree_size;
    plat_mmap tree_mmap;
    const TreeType *tree_cur_pos, *tree_end_pos;

    const char *dict;
    plat_mmap dict_mmap;

#if WITH_SQLITE3
    sqlite3 *db;
    sqlite3_stmt *stmt_config[STMT_CONFIG_COUNT];
    sqlite3_stmt *stmt_userphrase[STMT_USERPHRASE_COUNT];

    unsigned int original_lifetime;
    unsigned int new_lifetime;
#else
    int chewing_lifetime;

    char hashfilename[200];
    struct HASH_ITEM *hashtable[HASH_TABLE_SIZE];
    struct HASH_ITEM *userphrase_enum;  /* FIXME: Shall be in ChewingData? */
#endif

    unsigned int n_symbol_entry;
    SymbolEntry **symbol_table;

    char *g_easy_symbol_value[EASY_SYMBOL_KEY_TAB_LEN];
    int g_easy_symbol_num[EASY_SYMBOL_KEY_TAB_LEN];

    struct keymap *hanyuInitialsMap;
    struct keymap *hanyuFinalsMap;
    int HANYU_INITIALS;
    int HANYU_FINALS;
} ChewingStaticData;

typedef enum Category {
    CHEWING_NONE,
    CHEWING_CHINESE,
    CHEWING_SYMBOL,
} Category;

typedef struct PreeditBuf {
    Category category;
    char char_[MAX_UTF8_SIZE + 1];
} PreeditBuf;

typedef struct ChewingData {
    AvailInfo availInfo;
    ChoiceInfo choiceInfo;
    PhrasingOutput phrOut;
    BopomofoData bopomofoData;
    ChewingConfigData config;
        /** @brief current input buffer, content==0 means Chinese code */
    PreeditBuf preeditBuf[MAX_PHONE_SEQ_LEN];
    int chiSymbolCursor;
    int chiSymbolBufLen;
    int PointStart;
    int PointEnd;

    int bShowMsg;
    char showMsg[MAX_UTF8_SIZE * (MAX_PHRASE_LEN + AUX_PREFIX_LEN) + 1];
    int showMsgLen;

    uint16_t phoneSeq[MAX_PHONE_SEQ_LEN];
    uint16_t phoneSeqAlt[MAX_PHONE_SEQ_LEN];
    int nPhoneSeq;
    char selectStr[MAX_PHONE_SEQ_LEN][MAX_PHONE_SEQ_LEN * MAX_UTF8_SIZE + 1];
    IntervalType selectInterval[MAX_PHONE_SEQ_LEN];
    int nSelect;
    IntervalType preferInterval[MAX_INTERVAL];  /* add connect points */
    int nPrefer;
    int bUserArrCnnct[MAX_PHONE_SEQ_LEN + 1];
    int bUserArrBrkpt[MAX_PHONE_SEQ_LEN + 1];
    int bArrBrkpt[MAX_PHONE_SEQ_LEN + 1];
    int bSymbolArrBrkpt[MAX_PHONE_SEQ_LEN + 1];
    /* "bArrBrkpt[10]=True" means "it breaks between 9 and 10" */
    int bChiSym, bSelect, bFirstKey, bFullShape;
    /* Symbol Key buffer */
    char symbolKeyBuf[MAX_PHONE_SEQ_LEN];

#if WITH_SQLITE3
    UserPhraseData userphrase_data;
#else
    struct HASH_ITEM *prev_userphrase;
#endif

    ChewingStaticData static_data;
    void (*logger) (void *data, int level, const char *fmt, ...);
    void *loggerData;
} ChewingData;

/**
 * @struct ChewingOutput
 * @brief information for Chewing output.
 */

typedef struct ChewingOutput {
        /** @brief the content of Edit buffer. */
    char preeditBuf[MAX_PHONE_SEQ_LEN * MAX_UTF8_SIZE + 1];
        /** @brief the length of Edit buffer. */
    int chiSymbolBufLen;
        /** @brief current position of the cursor. */
    long chiSymbolCursor;
    long PointStart;
    long PointEnd;
    char bopomofoBuf[BOPOMOFO_SIZE * MAX_UTF8_SIZE + 1];
        /** @brief indicate the method of showing sentence break. */
    IntervalType dispInterval[MAX_INTERVAL];    /* from prefer, considering symbol */
    int nDispInterval;
        /** @brief indicate the break points going to display.*/
    int dispBrkpt[MAX_PHONE_SEQ_LEN + 1];
        /** @brief the string going to commit. */

    char commitBuf[MAX_PHONE_SEQ_LEN * MAX_UTF8_SIZE + 1];
    int commitBufLen;
        /** @brief information of character selections. */
    ChoiceInfo *pci;
        /** @brief indicate English mode or Chinese mode. */
    int bChiSym;
    int selKey[MAX_SELKEY];
        /** @brief return value. */
    int keystrokeRtn;
        /** @brief user message. */
} ChewingOutput;

/**
 * @struct ChewingContext
 * @brief context of Chewing IM
 */

struct ChewingContext {
    ChewingData *data;
    ChewingOutput *output;
    int cand_no;
    int it_no;
    int kb_no;
};

typedef struct Phrase {
    char phrase[MAX_PHRASE_LEN * MAX_UTF8_SIZE + 1];
    int freq;
} Phrase;

/* *INDENT-OFF* */
#endif
/* *INDENT-ON* */
