/**
 * init_database.c
 *
 * Copyright (c) 2013, 2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/**
 * @file init_database.c
 *
 * @brief Initialization of system dictionary and phone phrase tree.\n
 *
 *      This program reads in source of dictionary.\n
 *      Output a database file containing a phone phrase tree, and a dictionary file\n
 * filled with non-duplicate phrases.\n
 *      Each node represents a single phone.\n
 *      The output file contains a random access array, where each record includes:\n
 *      \code{
 *            [16-bit uint] key; may be phone data or record of input keys
 *            [24-bit uint] child.begin, child.end; for internal nodes (key != 0)
 *            [24-bit uint] phrase.pos; for leaf nodes (key == 0), position of phrase in dictionary
 *            [24-bit uint] phrase.freq; for leaf nodes (key == 0), frequency of the phrase
 *      }\endcode
 */

#include <assert.h>
#include <ctype.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "chewing-private.h"
#include "chewing-utf8-util.h"
#include "global-private.h"
#include "key2pho-private.h"
#include "memory-private.h"
#include "bopomofo-private.h"

/* For ALC macro */
#include "private.h"

#define CHARDEF               "%chardef"
#define BEGIN                 "begin"
#define END                   "end"
#define MAX_LINE_LEN          (1024)
#define MAX_WORD_DATA         (60000)
#define MAX_PHRASE_BUF_LEN    (149)
#define MAX_PHRASE_DATA       (420000)

const char USAGE[] =
    "Usage: %s <phone.cin> <tsi.src>\n"
    "This program creates the following new files:\n"
    "* " PHONE_TREE_FILE "\n\tindex to phrase file (dictionary)\n" "* " DICT_FILE "\n\tmain phrase file\n";

/* An additional pos helps avoid duplicate Chinese strings. */
typedef struct {
    char phrase[MAX_PHRASE_BUF_LEN];
    uint32_t freq;
    uint16_t phone[MAX_PHRASE_LEN + 1];
    long pos;
} PhraseData;

typedef struct {
    PhraseData *text;           /* Common part shared with PhraseData. */
    int index;                  /* For stable sorting. */
} WordData;

const PhraseData EXCEPTION_PHRASE[] = {
    {"\xE5\xA5\xBD\xE8\x90\x8A\xE5\xA1\xA2" /* 好萊塢 */ , 0, {5691, 4138, 256} /* ㄏㄠˇ ㄌㄞˊ ㄨ */ , 0},
    {"\xE6\x88\x90\xE6\x97\xA5\xE5\xAE\xB6" /* 成日家 */ , 0, {8290, 9220, 6281} /* ㄔㄥˊ ㄖˋ ㄐㄧㄚ˙ */ ,
     0},
    {"\xE4\xBF\xBE\xE5\x80\xAA" /* 俾倪 */ , 0, {644, 3716} /* ㄅㄧˋ ㄋㄧˋ */ , 0},
    {"\xE6\x8F\xA9\xE6\xB2\xB9" /* 揩油 */ , 0, {5128, 194} /* ㄎㄚ ㄧㄡˊ */ , 0},
    {"\xE6\x95\x81\xE6\x95\xAA" /* 敁敪 */ , 0, {2760, 2833} /* ㄉㄧㄢ ㄉㄨㄛ˙ */ , 0},
    {"\xE4\xB8\x80\xE9\xAA\xA8\xE7\xA2\x8C" /* 一骨碌 */ , 0, {128, 4866, 4353} /* ㄧ ㄍㄨˊ ㄌㄨ˙ */ , 0},
    {"\xE9\x82\x8B\xE9\x81\xA2" /* 邋遢 */ , 0, {4106, 3081} /* ㄌㄚˊ ㄊㄚ˙ */ , 0},

    {"\xE6\xBA\x9C\xE9\x81\x94" /* 溜達 */ , 0, {4292, 2569} /* ㄌㄧㄡˋ ㄉㄚ˙ */ , 0},
    {"\xE9\x81\x9B\xE9\x81\x94" /* 遛達 */ , 0, {4292, 2569} /* ㄌㄧㄡˋ ㄉㄚ˙ */ , 0},

    {"\xE5\xA4\xA7\xE5\xA4\xAB" /* 大夫 */ , 0, {2604, 2305} /* ㄉㄞˋ ㄈㄨ˙ */ , 0},

    {"\xE5\x92\x96\xE5\x96\xB1" /* 咖喱 */ , 0, {4616, 4226} /* ㄍㄚ ㄌㄧˊ */, 0},
    {"\xE5\x92\x96\xE5\x96\xB1\xE6\xB1\x81" /* 咖喱汁 */ , 0, {4616, 4226, 7680} /* ㄍㄚ ㄌㄧˊ ㄓ */, 0},
    {"\xE5\x92\x96\xE5\x96\xB1\xE7\xB2\x89" /* 咖喱粉 */ , 0, {4616, 4226, 2131} /* ㄍㄚ ㄌㄧˊ ㄈㄣˇ */, 0},
    {"\xE5\x92\x96\xE5\x96\xB1\xE9\x9B\x9E" /* 咖喱雞 */ , 0, {4616, 4226, 6272} /* ㄍㄚ ㄌㄧˊ ㄐㄧ */, 0},
    {"\xE5\x92\x96\xE5\x96\xB1\xE9\xA3\xAF" /* 咖喱飯 */ , 0, {4616, 4226, 2124} /* ㄍㄚ ㄌㄧˊ ㄈㄢˋ */, 0},
};

/*
 * Some word changes its phone in certain phrases. If it is difficult to list
 * all the phrases to exception phrase list, put the word here so that this
 * won't cause check error.
 */
const PhraseData EXCEPTION_WORD[] = {
    {"\xE5\x97\xA6" /* 嗦 */ , 0, {11025} /* ㄙㄨㄛ˙ */ , 0},
    {"\xE5\xB7\xB4" /* 巴 */ , 0, {521} /* ㄅㄚ˙ */ , 0},
    {"\xE4\xBC\x99" /* 伙 */ , 0, {5905} /* ㄏㄨㄛ˙ */ , 0},
};

/*
 * Please see TreeType for data field. pFirstChild points to the first of its
 * child list. pNextSibling points to its right sibling, where it and its right
 * sibling are both in the child list of its parent. However, pNextSibling will
 * become next-pointer like linked list, which makes writing of index-tree file
 * become a sequential traversal rather than BFS.
 */
typedef struct _tNODE {
    TreeType data;
    struct _tNODE *pFirstChild;
    struct _tNODE *pNextSibling;
} NODE;

WordData word_data[MAX_WORD_DATA];
char word_matched[MAX_WORD_DATA];
int num_word_data = 0;

PhraseData phrase_data[MAX_PHRASE_DATA];
int num_phrase_data = 0;
int top_phrase_data = MAX_PHRASE_DATA;

NODE *root;

void strip(char *line)
{
    char *end;
    size_t i;

    /* remove comment */
    for (i = 0; i < strlen(line); ++i) {
        if (line[i] == '#') {
            line[i] = '\0';
            break;
        }
    }

    /* remove tailing space */
    end = line + strlen(line) - 1;
    while (end >= line && isspace((unsigned char) *end)) {
        *end = 0;
        --end;
    }
}

/* word_data is sorted reversely, for stack-like push operation. */
int compare_word_by_phone(const void *x, const void *y)
{
    const WordData *a = (const WordData *) x;
    const WordData *b = (const WordData *) y;

    if (a->text->phone[0] != b->text->phone[0])
        return b->text->phone[0] - a->text->phone[0];

    /* Compare original index for stable sort */
    return b->index - a->index;
}

int compare_word_by_text(const void *x, const void *y)
{
    const WordData *a = (const WordData *) x;
    const WordData *b = (const WordData *) y;
    int ret = strcmp(a->text->phrase, b->text->phrase);

    if (ret != 0)
        return ret;

    if (a->text->phone[0] != b->text->phone[0])
        return a->text->phone[0] - b->text->phone[0];

    return 0;
}

int compare_word_no_duplicated(const void *x, const void *y)
{
    int ret = compare_word_by_text(x, y);

    if (!ret) {
        const WordData *a = (const WordData *) x;

        fprintf(stderr, "Duplicated word found (`%s', %d).\n", a->text->phrase, a->text->phone[0]);
        exit(-1);
    }

    return ret;
}

int is_exception_phrase(PhraseData *phrase, int pos)
{
    size_t i;
    char word[MAX_UTF8_SIZE + 1];

    ueStrNCpy(word, ueStrSeek(phrase->phrase, pos), 1, 1);

    /*
     * Check if the phrase is an exception phrase.
     */
    for (i = 0; i < sizeof(EXCEPTION_PHRASE) / sizeof(EXCEPTION_PHRASE[0]); ++i) {
        if (strcmp(phrase->phrase, EXCEPTION_PHRASE[i].phrase) == 0 &&
            memcmp(phrase->phone, EXCEPTION_PHRASE[i].phone, sizeof(phrase->phone)) == 0) {
            return 1;
        }
    }

    /*
     * Check if the word in phrase is an exception word.
     */
    for (i = 0; i < sizeof(EXCEPTION_WORD) / sizeof(EXCEPTION_WORD[0]); ++i) {
        if (strcmp(word, EXCEPTION_WORD[i].phrase) == 0 && phrase->phone[pos] == EXCEPTION_WORD[i].phone[0]) {
            return 1;
        }
    }

    /*
     * If the same word appears continuous in a phrase (疊字), the second
     * word can change to light tone.
     * ex:
     * 爸爸 -> ㄅㄚˋ ㄅㄚ˙
     */
    if (pos > 0) {
        char previous[MAX_UTF8_SIZE + 1];

        ueStrNCpy(previous, ueStrSeek(phrase->phrase, pos - 1), 1, 1);

        if (strcmp(previous, word) == 0) {
            if (((phrase->phone[pos - 1] & ~0x7) | 0x1) == phrase->phone[pos]) {
                return 1;
            }
        }
    }

    return 0;
}

void store_phrase(const char *line, int line_num)
{
    const char DELIM[] = " \t\n";
    char buf[MAX_LINE_LEN];
    char *phrase;
    char *freq;
    char *endptr = NULL;
    char *bopomofo;
    char bopomofo_buf[MAX_UTF8_SIZE * BOPOMOFO_SIZE + 1];
    size_t phrase_len;
    WordData word;              /* For check. */
    WordData *found_word = NULL;
    size_t i, j;

    snprintf(buf, sizeof(buf), "%s", line);
    strip(buf);
    if (strlen(buf) == 0)
        return;

    if (num_phrase_data >= top_phrase_data) {
        fprintf(stderr, "Need to increase MAX_PHRASE_DATA to process\n");
        exit(-1);
    }

    /* read phrase */
    phrase = strtok(buf, DELIM);
    if (!phrase) {
        fprintf(stderr, "Error reading line %d, `%s'\n", line_num, line);
        exit(-1);
    }
    strncpy(phrase_data[num_phrase_data].phrase, phrase, sizeof(phrase_data[0].phrase) - 1);

    /* read frequency */
    freq = strtok(NULL, DELIM);
    if (!freq) {
        fprintf(stderr, "Error reading line %d, `%s'\n", line_num, line);
        exit(-1);
    }

    phrase_data[num_phrase_data].freq = strtoul(freq, &endptr, 0);
    if ((*freq == '\0' || *endptr != '\0') ||
        (phrase_data[num_phrase_data].freq == UINT32_MAX && errno == ERANGE)) {
        fprintf(stderr, "Error reading frequency `%s' in line %d, `%s'\n", freq, line_num, line);
        exit(-1);
    }

    /* read bopomofo */
    for (bopomofo = strtok(NULL, DELIM), phrase_len = 0;
         bopomofo && phrase_len < MAX_PHRASE_LEN; bopomofo = strtok(NULL, DELIM), ++phrase_len) {

        phrase_data[num_phrase_data].phone[phrase_len] = UintFromPhone(bopomofo);
        if (phrase_data[num_phrase_data].phone[phrase_len] == 0) {
            fprintf(stderr, "Error reading bopomofo `%s' in line %d, `%s'\n", bopomofo, line_num, line);
            exit(-1);
        }
    }
    if (bopomofo) {
        fprintf(stderr, "Phrase `%s' too long in line %d\n", phrase, line_num);
    }

    /* check phrase length & bopomofo length */
    if ((size_t) ueStrLen(phrase_data[num_phrase_data].phrase) != phrase_len) {
        fprintf(stderr, "Phrase length and bopomofo length mismatch in line %d, `%s'\n", line_num, line);
        exit(-1);
    }

    /* Check that each word in phrase can be found in word list. */
    word.text = ALC(PhraseData, 1);

    assert(word.text);
    for (i = 0; i < phrase_len; ++i) {
        ueStrNCpy(word.text->phrase, ueStrSeek(phrase_data[num_phrase_data].phrase, i), 1, 1);
        word.text->phone[0] = phrase_data[num_phrase_data].phone[i];
        found_word = bsearch(&word, word_data, num_word_data, sizeof(word), compare_word_by_text);
        if ((found_word == NULL ||
             (phrase_len == 1 &&
              word_matched[found_word - word_data])) && !is_exception_phrase(&phrase_data[num_phrase_data], i)) {

            PhoneFromUint(bopomofo_buf, sizeof(bopomofo_buf), word.text->phone[0]);

            fprintf(stderr, "Error in phrase `%s'. Word `%s' has no phone %d (%s) in line %d\n",
                    phrase_data[num_phrase_data].phrase, word.text->phrase, word.text->phone[0], bopomofo_buf,
                    line_num);
            fprintf(stderr, "\tAdd the following struct to EXCEPTION_PHRASE if this is good phrase\n\t{\"");
            for (j = 0; j < strlen(phrase_data[num_phrase_data].phrase); ++j) {
                fprintf(stderr, "\\x%02X", (unsigned char) phrase_data[num_phrase_data].phrase[j]);
            }
            fprintf(stderr, "\" /* %s */ , 0, {%d", phrase_data[num_phrase_data].phrase,
                    phrase_data[num_phrase_data].phone[0]);
            for (j = 1; j < phrase_len; ++j) {
                fprintf(stderr, ", %d", phrase_data[num_phrase_data].phone[j]);
            }
            fprintf(stderr, "} /* ");
            for (j = 0; j < phrase_len; ++j) {
                PhoneFromUint(bopomofo_buf, sizeof(bopomofo_buf), phrase_data[num_phrase_data].phone[j]);
                fprintf(stderr, "%s ", bopomofo_buf);
            }
            fprintf(stderr, "*/, 0},\n");
            exit(-1);
        }
    }
    free(word.text);

    if (phrase_len >= 2)
        ++num_phrase_data;
    else
        word_matched[found_word - word_data] = 1;
}

int compare_phrase(const void *x, const void *y)
{
    const PhraseData *a = (const PhraseData *) x;
    const PhraseData *b = (const PhraseData *) y;
    int cmp = strcmp(a->phrase, b->phrase);

    /* If phrases are different, it returns the result of strcmp(); else it
     * reports an error when the same phone sequence is found.
     */
    if (cmp)
        return cmp;
    if (!memcmp(a->phone, b->phone, sizeof(a->phone))) {
        fprintf(stderr, "Duplicated phrase `%s' found.\n", a->phrase);
        exit(-1);
    }
    return b->freq - a->freq;
}

void read_tsi_src(const char *filename)
{
    FILE *tsi_src;
    char buf[MAX_LINE_LEN];
    int line_num = 0;

    tsi_src = fopen(filename, "r");
    if (!tsi_src) {
        fprintf(stderr, "Error opening the file %s\n", filename);
        exit(-1);
    }

    while (fgets(buf, sizeof(buf), tsi_src)) {
        ++line_num;
        store_phrase(buf, line_num);
    }

    qsort(phrase_data, num_phrase_data, sizeof(phrase_data[0]), compare_phrase);
    fclose(tsi_src);
}

void store_word(const char *line, const int line_num)
{
    char phone_buf[MAX_UTF8_SIZE * BOPOMOFO_SIZE + 1];
    char key_buf[BOPOMOFO_SIZE + 1];
    char buf[MAX_LINE_LEN + 1] = {0};

    strncpy(buf, line, sizeof(buf) - 1);

    strip(buf);
    if (strlen(buf) == 0)
        return;

    if (num_word_data >= MAX_WORD_DATA) {
        fprintf(stderr, "Need to increase MAX_WORD_DATA to process\n");
        exit(-1);
    }
    if (top_phrase_data <= num_phrase_data) {
        fprintf(stderr, "Need to increase MAX_PHRASE_DATA to process\n");
        exit(-1);
    }
    word_data[num_word_data].text = &phrase_data[--top_phrase_data];

#define UTF8_FORMAT_STRING(len1, len2) \
    "%" __stringify(len1) "[^ ]" " " \
    "%" __stringify(len2) "[^ ]"
    sscanf(buf, UTF8_FORMAT_STRING(BOPOMOFO_SIZE, MAX_UTF8_SIZE), key_buf, word_data[num_word_data].text->phrase);

    if (strlen(key_buf) > BOPOMOFO_SIZE) {
        fprintf(stderr, "Error reading line %d, `%s'\n", line_num, line);
        exit(-1);
    }
    PhoneFromKey(phone_buf, key_buf, KB_DEFAULT, 1);
    word_data[num_word_data].text->phone[0] = UintFromPhone(phone_buf);

    word_data[num_word_data].index = num_word_data;
    ++num_word_data;
}

void read_phone_cin(const char *filename)
{
    FILE *phone_cin;
    char buf[MAX_LINE_LEN];
    char *ret;
    int line_num = 0;
    enum { INIT, HAS_CHARDEF_BEGIN, HAS_CHARDEF_END } status;

    phone_cin = fopen(filename, "r");
    if (!phone_cin) {
        fprintf(stderr, "Error opening the file %s\n", filename);
        exit(-1);
    }

    for (status = INIT; status != HAS_CHARDEF_BEGIN;) {
        ret = fgets(buf, sizeof(buf), phone_cin);
        ++line_num;
        if (!ret) {
            fprintf(stderr, "%s: No expected %s %s\n", filename, CHARDEF, BEGIN);
            exit(-1);
        }

        strip(buf);
        ret = strtok(buf, " \t");
        if (!strcmp(ret, CHARDEF)) {
            ret = strtok(NULL, " \t");
            if (!strcmp(ret, BEGIN))
                status = HAS_CHARDEF_BEGIN;
            else {
                fprintf(stderr, "%s:%d: Unexpected %s %s\n", filename, line_num, CHARDEF, ret);
                exit(-1);
            }
        }
    }

    while (status != HAS_CHARDEF_END) {
        ret = fgets(buf, sizeof(buf), phone_cin);
        ++line_num;
        if (!ret) {
            fprintf(stderr, "%s: No expected %s %s\n", filename, CHARDEF, END);
            exit(-1);
        }

        strip(buf);
        if (!strncmp(buf, CHARDEF, strlen(CHARDEF))) {
            strtok(buf, " \t");
            ret = strtok(NULL, " \t");
            if (!strcmp(ret, END))
                status = HAS_CHARDEF_END;
            else {
                fprintf(stderr, "%s:%d: Unexpected %s %s\n", filename, line_num, CHARDEF, ret);
                exit(-1);
            }
        } else
            store_word(buf, line_num);
    }

    fclose(phone_cin);

    qsort(word_data, num_word_data, sizeof(word_data[0]), compare_word_no_duplicated);
}

NODE *new_node(uint32_t key)
{
    NODE *pnew = ALC(NODE, 1);

    if (pnew == NULL) {
        fprintf(stderr, "Memory allocation failed on constructing phrase tree.\n");
        exit(-1);
    }

    memset(&pnew->data, 0, sizeof(pnew->data));
    PutUint16(key, pnew->data.key);
    pnew->pFirstChild = NULL;
    pnew->pNextSibling = NULL;
    return pnew;
}

/*
 * This function puts FindKey() and Insert() together. It first searches for the
 * specified key and performs FindKey() on hit. Otherwise, it inserts a new node
 * at proper position and returns the newly inserted child.
 */
NODE *find_or_insert(NODE * parent, uint32_t key)
{
    NODE *prev = NULL;
    NODE *p;
    NODE *pnew;

    for (p = parent->pFirstChild; p && GetUint16(p->data.key) <= key; prev = p, p = p->pNextSibling)
        if (GetUint16(p->data.key) == key)
            return p;

    pnew = new_node(key);
    pnew->pNextSibling = p;
    if (prev == NULL)
        parent->pFirstChild = pnew;
    else
        prev->pNextSibling = pnew;
    pnew->pNextSibling = p;
    return pnew;
}

void insert_leaf(NODE * parent, long phr_pos, uint32_t freq)
{
    NODE *prev = NULL;
    NODE *p;
    NODE *pnew;

    for (p = parent->pFirstChild; p && GetUint16(p->data.key) == 0; prev = p, p = p->pNextSibling)
        if (GetUint16(p->data.phrase.freq) <= freq)
            break;

    pnew = new_node(0);
    PutUint24((uint32_t) phr_pos, pnew->data.phrase.pos);
    PutUint24(freq, pnew->data.phrase.freq);
    if (prev == NULL)
        parent->pFirstChild = pnew;
    else
        prev->pNextSibling = pnew;
    pnew->pNextSibling = p;
}

void construct_phrase_tree()
{
    NODE *levelPtr;
    int i;
    int j;

    /* First, assume that words are in order of their phones and indices. */
    qsort(word_data, num_word_data, sizeof(word_data[0]), compare_word_by_phone);

    /* Key value of root will become tree_size later. */
    root = new_node(1);

    /* Second, insert word_data as the first level of children. */
    for (i = 0; i < num_word_data; i++) {
        if (i == 0 || word_data[i].text->phone[0] != word_data[i - 1].text->phone[0]) {
            levelPtr = new_node(word_data[i].text->phone[0]);
            levelPtr->pNextSibling = root->pFirstChild;
            root->pFirstChild = levelPtr;
        }
        levelPtr = new_node(0);
        PutUint24((uint32_t) word_data[i].text->pos, levelPtr->data.phrase.pos);
        PutUint24(word_data[i].text->freq, levelPtr->data.phrase.freq);
        levelPtr->pNextSibling = root->pFirstChild->pFirstChild;
        root->pFirstChild->pFirstChild = levelPtr;
    }

    /* Third, insert phrases having length at least 2. */
    for (i = 0; i < num_phrase_data; ++i) {
        levelPtr = root;
        for (j = 0; phrase_data[i].phone[j] != 0; ++j)
            levelPtr = find_or_insert(levelPtr, phrase_data[i].phone[j]);
        insert_leaf(levelPtr, phrase_data[i].pos, phrase_data[i].freq);
    }
}

void write_phrase_data()
{
    FILE *dict_file;
    PhraseData *cur_phr;
    PhraseData *last_phr = NULL;
    int i;
    int j;

    dict_file = fopen(DICT_FILE, "wb");

    if (!dict_file) {
        fprintf(stderr, "Cannot open output file.\n");
        exit(-1);
    }

    /*
     * Duplicate Chinese strings with common pronunciation are detected and
     * not written into system dictionary. Written phrases are separated by
     * '\0', for convenience of mmap usage.
     * Note: word_data and phrase_data have been qsorted by strcmp in
     *       reading.
     */
    for (i = j = 0; i < num_word_data || j < num_phrase_data; last_phr = cur_phr) {
        if (i == num_word_data)
            cur_phr = &phrase_data[j++];
        else if (j == num_phrase_data)
            cur_phr = word_data[i++].text;
        else if (strcmp(word_data[i].text->phrase, phrase_data[j].phrase) < 0)
            cur_phr = word_data[i++].text;
        else
            cur_phr = &phrase_data[j++];

        if (last_phr && !strcmp(cur_phr->phrase, last_phr->phrase))
            cur_phr->pos = last_phr->pos;
        else {
            cur_phr->pos = ftell(dict_file);
            fwrite(cur_phr->phrase, strlen(cur_phr->phrase) + 1, 1, dict_file);
        }
    }

    fclose(dict_file);
}

/*
 * This function performs BFS to compute child.begin and child.end of each node.
 * It sponteneously converts tree structure into a linked list. Writing the tree
 * into index file is then implemented by pure sequential traversal.
 */
void write_index_tree()
{
    /* (Circular) queue implementation is hidden within this function. */
    NODE **queue;
    NODE *p;
    NODE *pNext;
    size_t head = 0, tail = 0;
    size_t tree_size = 1;
    size_t q_len = num_word_data + num_phrase_data + 1;

    FILE *output = fopen(PHONE_TREE_FILE, "wb");

    if (!output) {
        fprintf(stderr, "Error opening file " PHONE_TREE_FILE " for output.\n");
        exit(-1);
    }

    queue = ALC(NODE *, q_len);
    assert(queue);

    queue[head++] = root;
    while (head != tail) {
        p = queue[tail++];
        if (tail >= q_len)
            tail = 0;
        if (GetUint16(p->data.key) != 0) {
            PutUint24(tree_size, p->data.child.begin);

            /*
             * The latest inserted element must have a NULL
             * pNextSibling value, and the following code let
             * it point to the next child list to serialize
             * them.
             */
            if (head == 0)
                queue[q_len - 1]->pNextSibling = p->pFirstChild;
            else
                queue[head - 1]->pNextSibling = p->pFirstChild;

            for (pNext = p->pFirstChild; pNext; pNext = pNext->pNextSibling) {
                queue[head++] = pNext;
                if (head == q_len)
                    head = 0;
                tree_size++;
            }

            PutUint24(tree_size, p->data.child.end);
        }
    }
    PutUint16(tree_size, root->data.key);

    for (p = root; p; p = pNext) {
        fwrite(&p->data, sizeof(TreeType), 1, output);
        pNext = p->pNextSibling;
        free(p);
    }
    free(queue);

    fclose(output);
}

int main(int argc, char *argv[])
{
    if (argc != 3) {
        printf(USAGE, argv[0]);
        return -1;
    }

    read_phone_cin(argv[1]);
    read_tsi_src(argv[2]);
    write_phrase_data();
    construct_phrase_tree();
    write_index_tree();
    return 0;
}
