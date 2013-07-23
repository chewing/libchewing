/**
 * sort.c
 *
 * Copyright (c) 2012
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#include <ctype.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "chewing-private.h"
#include "chewing-utf8-util.h"
#include "global-private.h"
#include "key2pho-private.h"
#include "zuin-private.h"

/* for ARRAY_SIZE macro */
#include "private.h"

#define CHARDEF_BEGIN		"%chardef  begin"
#define CHARDEF_END		"%chardef  end"
#define MAX_LINE_LEN		(1024)
#define MAX_WORD_DATA		(60000)
#define MAX_PHONE		(12000)
#define MAX_PHRASE_BUF_LEN	(149)
#define MAX_FILE_NAME		(256)
#define MAX_PHRASE_DATA		(420000)
#define PHONEID_FILE		"phoneid.dic"

const char USAGE[] =
	"usage: %s <phone.cin> <tsi.src>\n"
	"This program creates the following new files:\n"
#ifdef USE_BINARY_DATA
	"* " CHAR_INDEX_PHONE_FILE "\n\tindex of word file (phone -> index)\n"
	"* " CHAR_INDEX_BEGIN_FILE "\n\tindex of word file (index -> offset)\n"
#else
	"* " CHAR_INDEX_FILE "\n\tindex of word file\n"
#endif
	"* " CHAR_FILE "\n\tmain word file\n"
	"* " PH_INDEX_FILE "\n\tindex of phrase file\n"
	"* " DICT_FILE "\n\tmain phrase file\n"
	"* " PHONEID_FILE "\n\tintermediate file for make_tree\n"
;

struct WordData {
	int index; /* Used for stable sort */
	uint16_t phone;
	char word[MAX_UTF8_SIZE + 1];
};

struct PhraseData {
	char phrase[MAX_PHRASE_BUF_LEN];
	int freq;
	uint16_t phone[MAX_PHRASE_LEN + 1];
};

struct WordData word_data[MAX_WORD_DATA];
int num_word_data = 0;

struct PhraseData phrase_data[MAX_PHRASE_DATA];
int num_phrase_data = 0;

const struct PhraseData EXCEPTION_PHRASE[] = {
	{ "\xE5\xA5\xBD\xE8\x90\x8A\xE5\xA1\xA2" /* 好萊塢 */ , 0, { 5691, 4138, 256 } /* ㄏㄠˇ ㄌㄞˊ ㄨ */ },
	{ "\xE6\x88\x90\xE6\x97\xA5\xE5\xAE\xB6" /* 成日家 */ , 0, { 8290, 9220, 6281 } /* ㄔㄥˊ ㄖˋ ㄐㄧㄚ˙ */ },
	{ "\xE4\xBF\xBE\xE5\x80\xAA" /* 俾倪 */ , 0, { 644, 3716 } /* ㄅㄧˋ ㄋㄧˋ */ },
	{ "\xE6\x8F\xA9\xE6\xB2\xB9" /* 揩油 */ , 0, { 5128, 194 } /* ㄎㄚ ㄧㄡˊ */ },
	{ "\xE6\x95\x81\xE6\x95\xAA" /* 敁敪 */ , 0, { 2760, 2833 } /* ㄉㄧㄢ ㄉㄨㄛ˙ */ },
	{ "\xE4\xB8\x80\xE9\xAA\xA8\xE7\xA2\x8C" /* 一骨碌 */ , 0, { 128, 4866, 4353 } /* ㄧ ㄍㄨˊ ㄌㄨ˙ */ },
	{ "\xE9\x82\x8B\xE9\x81\xA2" /* 邋遢 */ , 0, { 4106, 3081 } /* ㄌㄚˊ ㄊㄚ˙ */ },

	{ "\xE6\xBA\x9C\xE9\x81\x94" /* 溜達 */ , 0, { 4292, 2569 } /* ㄌㄧㄡˋ ㄉㄚ˙ */ },
	{ "\xE9\x81\x9B\xE9\x81\x94" /* 遛達 */ , 0, { 4292, 2569 } /* ㄌㄧㄡˋ ㄉㄚ˙ */ },
};

/*
 * Some word changes its phone in certain phrases. If it is difficult to list
 * all the phrases to exception phrase list, put the word here so that this
 * won't cause check error.
 */
const struct WordData EXCEPTION_WORD[] = {
	{ 0, 11025 /* ㄙㄨㄛ˙ */ , "\xE5\x97\xA6" /* 嗦 */ },
	{ 0, 521 /* ㄅㄚ˙ */ , "\xE5\xB7\xB4" /* 巴 */ },
	{ 0, 5905 /* ㄏㄨㄛ˙ */ , "\xE4\xBC\x99" /* 伙 */ },
};

void strip(char *line)
{
	char *end;
	size_t i;

	/* remove comment */
	for (i = 0; i < strlen(line); ++i) {
		if (line[i] == '#') {
			line[i] = 0;
			break;
		}
	}

	/* remove tailing space */
	end = line + strlen(line) - 1;
	while (end >= line && isspace((unsigned char)*end)) {
		*end = 0;
		--end;
	}
}

void store_word(const char *line, const int line_num)
{
	char phone_buf[MAX_UTF8_SIZE * ZUIN_SIZE + 1];
	char key_buf[ZUIN_SIZE + 1];
	char buf[MAX_LINE_LEN];

	strncpy(buf, line, sizeof(buf));

	strip(buf);
	if (strlen(buf) == 0)
		return;

	if (num_word_data >= MAX_WORD_DATA) {
		fprintf(stderr, "Need to increase MAX_WORD_DATA to process\n");
		exit(-1);
	}

#define UTF8_FORMAT_STRING(len1, len2) \
	"%" __stringify(len1) "[^ ]" " " \
	"%" __stringify(len2) "[^ ]"
	sscanf(buf, UTF8_FORMAT_STRING(ZUIN_SIZE, MAX_UTF8_SIZE),
		key_buf, word_data[num_word_data].word);

	if (strlen(key_buf) > ZUIN_SIZE) {
		fprintf(stderr, "Error reading line %d, `%s'\n", line_num, line);
		exit(-1);
	}
	PhoneFromKey(phone_buf, key_buf, KB_DEFAULT, 1);
	word_data[num_word_data].phone = UintFromPhone(phone_buf);
	word_data[num_word_data].index = num_word_data;
	++num_word_data;
}

int compare_word_by_phone(const void *x, const void *y)
{
	const struct WordData *a = (struct WordData *)x;
	const struct WordData *b = (struct WordData *)y;

	if (a->phone != b->phone)
		return a->phone - b->phone;

	/* Compare original index for stable sort */
	return a->index - b->index;
}

int compare_word(const void *x, const void *y)
{
	const struct WordData *a = (const struct WordData *)x;
	const struct WordData *b = (const struct WordData *)y;
	int ret;

	ret = strcmp(a->word, b->word);
	if (ret != 0)
		return ret;

	if (a->phone != b->phone)
		return a->phone - b->phone;

	return 0;
}

int compare_word_no_duplicated(const void *x, const void *y)
{
	int ret;

	ret = compare_word(x, y);
	if (!ret) {
		const struct WordData *a = (const struct WordData *)x;
		fprintf(stderr, "Duplicated word found (`%s', %d).\n", a->word, a->phone);
		exit(-1);
	}

	return ret;
}

void read_phone_cin(const char *filename)
{
	FILE *phone_cin;
	char buf[MAX_LINE_LEN];
	char *ret;
	int line_num = 0;

	phone_cin = fopen(filename, "r");
	if (!phone_cin) {
		fprintf(stderr, "Error opening the file %s\n", filename);
		exit(-1);
	}

	/* Find `%chardef  begin' */
	for (;;) {
		ret = fgets(buf, sizeof(buf), phone_cin);
		++line_num;
		if (!ret) {
			fprintf(stderr, "Cannot find %s\n", CHARDEF_BEGIN);
			exit(-1);
		}

		if (strncmp(buf, CHARDEF_BEGIN, strlen(CHARDEF_BEGIN)) == 0) {
			break;
		}
	}

	/* read all words into word_data. */
	for (;;) {
		ret = fgets(buf, sizeof(buf), phone_cin);
		++line_num;
		if (!ret || buf[0] == '%')
			break;

		store_word(buf, line_num);
	}
	fclose(phone_cin);

	qsort(word_data, num_word_data, sizeof(word_data[0]), compare_word_by_phone);

	return;
}

void write_word_data()
{
	FILE *chewing_file;
	FILE *char_file;
#ifdef USE_BINARY_DATA
	FILE *index_begin_file;
	FILE *index_phone_file;
	unsigned char size;
#else
	FILE *index_file;
#endif
	int i;
	uint16_t previous_phone;
	int phone_num;
	int pos;


	chewing_file = fopen(CHEWING_DEFINITION_FILE, "w");
#ifdef USE_BINARY_DATA
	index_begin_file = fopen(CHAR_INDEX_BEGIN_FILE, "wb");
	index_phone_file = fopen(CHAR_INDEX_PHONE_FILE, "wb");
	char_file = fopen(CHAR_FILE, "wb");

	if (!(chewing_file && index_begin_file && index_phone_file && char_file)) {
		fprintf(stderr, "Cannot open output file.\n");
		exit(-1);
	}
#else
	index_file = fopen(CHAR_INDEX_FILE, "w");
	char_file = fopen(CHAR_FILE, "w");
	if (!(chewing_file && index_file && char_file)) {
		fprintf(stderr, "Cannot open output file.\n");
		exit(-1);
	}
#endif

	previous_phone = 0;
	phone_num = 0;
	for (i = 0; i < num_word_data; ++i) {
		if (word_data[i].phone != previous_phone) {
			previous_phone = word_data[i].phone;
			pos = ftell(char_file);
#ifdef USE_BINARY_DATA
			fwrite(&pos, sizeof(pos), 1, index_begin_file);
			fwrite(&previous_phone, sizeof(previous_phone), 1, index_phone_file);
#else
			fprintf(index_file, "%hu %d\n", previous_phone, pos);
#endif
			phone_num++;
		}

#ifdef USE_BINARY_DATA
		size = strlen(word_data[ i ].word);
		fwrite(&size, sizeof(size), 1, char_file);
		fwrite(word_data[i].word, size, 1, char_file);
#else
		fprintf(char_file, "%hu %s\t", word_data[i].phone, word_data[i].word);
#endif
	}
	pos = ftell(char_file);
#ifdef USE_BINARY_DATA
	fwrite(&pos, sizeof(pos), 1, index_begin_file);
	previous_phone = 0;
	fwrite(&previous_phone, sizeof(previous_phone), 1, index_phone_file);
#else
	fprintf(index_file, "0 %d\n", pos);
#endif
	fprintf(chewing_file, "#define PHONE_NUM (%d)\n", phone_num);

	fclose(char_file);
#ifdef USE_BINARY_DATA
	fclose(index_phone_file);
	fclose(index_begin_file);
#else
	fclose(index_file);
#endif
	fclose(chewing_file);
}

void sort_word_for_dictionary()
{
	qsort(word_data, num_word_data, sizeof(word_data[0]), compare_word_no_duplicated);
}

int is_exception_phrase(struct PhraseData *phrase, int pos) {
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
		if (strcmp(word, EXCEPTION_WORD[i].word) == 0 &&
			phrase->phone[pos] == EXCEPTION_WORD[i].phone) {
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
	char *bopomofo;
	size_t phrase_len;
	size_t i;
	size_t j;
	struct WordData word;
	char bopomofo_buf[MAX_UTF8_SIZE * ZUIN_SIZE + 1];

	strncpy(buf, line, sizeof(buf));

	strip(buf);
	if (strlen(buf) == 0)
		return;

	if (num_phrase_data >= MAX_PHRASE_DATA) {
		fprintf(stderr, "Need to increase MAX_PHRASE_DATA to process\n");
		exit(-1);
	}

	/* read phrase */
	phrase = strtok(buf, DELIM);
	if (!phrase) {
		fprintf(stderr, "Error reading line %d, `%s'\n", line_num, line);
		exit(-1);
	}
	strncpy(phrase_data[num_phrase_data].phrase, phrase, sizeof(phrase_data[0].phrase));

	/* read frequency */
	freq = strtok(NULL, DELIM);
	if (!freq) {
		fprintf(stderr, "Error reading line %d, `%s'\n", line_num, line);
		exit(-1);
	}

	errno = 0;
	phrase_data[num_phrase_data].freq = strtol(freq, 0, 0);
	if (errno) {
		fprintf(stderr, "Error reading frequency `%s' in line %d, `%s'\n", freq, line_num, line);
		exit(-1);
	}

	/* read bopomofo */
	for (bopomofo = strtok(NULL, DELIM), phrase_len = 0;
		bopomofo && phrase_len < MAX_PHRASE_LEN;
		bopomofo = strtok(NULL, DELIM), ++phrase_len) {

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
	if ((size_t)ueStrLen(phrase_data[num_phrase_data].phrase) != phrase_len) {
		fprintf(stderr, "Phrase length and bopomofo length mismatch in line %d, `%s'\n", line_num, line);
		exit(-1);
	}

	/* check each word in phrase */
	for (i = 0; i < phrase_len; ++i) {
		ueStrNCpy(word.word, ueStrSeek(phrase_data[num_phrase_data].phrase, i), 1, 1);
		word.phone = phrase_data[num_phrase_data].phone[i];

		if (bsearch(&word, word_data, num_word_data, sizeof(word), compare_word) == NULL &&
			!is_exception_phrase(&phrase_data[num_phrase_data], i)) {

			PhoneFromUint(bopomofo_buf, sizeof(bopomofo_buf), word.phone);

			fprintf(stderr, "Error in phrase `%s'. Word `%s' has no phone %d (%s) in line %d\n", phrase_data[num_phrase_data].phrase ,word.word, word.phone, bopomofo_buf, line_num);
			fprintf(stderr, "\tAdd the following struct to EXCEPTION_PHRASE if this is good phrase\n\t{ \"");
			for (j = 0; j < strlen(phrase_data[num_phrase_data].phrase); ++j) {
				fprintf(stderr, "\\x%02X", (unsigned char)phrase_data[num_phrase_data].phrase[j]);
			}
			fprintf(stderr, "\" /* %s */ , 0, { %d", phrase_data[num_phrase_data].phrase, phrase_data[num_phrase_data].phone[0]);
			for (j = 1; j < phrase_len; ++j) {
				fprintf(stderr, ", %d", phrase_data[num_phrase_data].phone[j]);
			}
			fprintf(stderr, " } /* ");
			for (j = 0; j < phrase_len; ++j) {
				PhoneFromUint(bopomofo_buf, sizeof(bopomofo_buf), phrase_data[num_phrase_data].phone[j]);
				fprintf(stderr, "%s ", bopomofo_buf);
			}
			fprintf(stderr, "*/ },\n");
			exit(-1);
		}
	}

	++num_phrase_data;
}

int compare_phrase(const void *x, const void *y)
{
	const struct PhraseData *a = (const struct PhraseData *) x;
	const struct PhraseData *b = (const struct PhraseData *) y;
	int cmp;
	size_t i;

	for (i = 0; i < ARRAY_SIZE(a->phone); ++i) {
		cmp = a->phone[i] - b->phone[i];
		if (cmp)
			return cmp;
	}

	if (!strcmp(a->phrase, b->phrase)) {
		fprintf(stderr, "Duplicated phrase `%s' found.\n", a->phrase);
		exit(-1);
	}

	if (a->freq == b->freq) {
		/* FIXME: shall exit(-1) when tsi.src is fixed */
		//fprintf(stderr, "Phrase `%s' and `%s' have the same phone and frequency (%d).\n", a->phrase, b->phrase, a->freq);
		//exit(-1);
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
}

int compare_phone_in_phrase(int x, int y)
{
	return memcmp(phrase_data[x].phone, phrase_data[y].phone, sizeof(phrase_data[0].phone));
}

void write_phrase_data()
{
	FILE *dict_file;
	FILE *ph_index_file;
	FILE *phoneid_file;
	int i;
	int j;
	int pos;
#ifdef USE_BINARY_DATA
	unsigned char size;
#endif

#ifdef USE_BINARY_DATA
	dict_file = fopen(DICT_FILE, "wb");
	ph_index_file = fopen(PH_INDEX_FILE, "wb");
#else
	dict_file = fopen(DICT_FILE, "w");
	ph_index_file = fopen(PH_INDEX_FILE, "w");
#endif
	phoneid_file = fopen(PHONEID_FILE, "w");

	if (!(dict_file && ph_index_file && phoneid_file)) {
		fprintf(stderr, "Cannot open output file.\n");
		exit(-1);
	}

	for (i = 0; i < num_phrase_data - 1; ++i) {
		if (i == 0 || compare_phone_in_phrase(i - 1, i)) {
			pos = ftell(dict_file);
#ifdef USE_BINARY_DATA
			fwrite(&pos, sizeof(pos), 1, ph_index_file);
#else
			fprintf(ph_index_file, "%d\n", pos);
#endif
		}
#ifdef USE_BINARY_DATA
		size = strlen(phrase_data[i].phrase);
		fwrite(&size, sizeof(size), 1, dict_file);
		fwrite(phrase_data[i].phrase, size, 1, dict_file);
		fwrite(&phrase_data[i].freq, sizeof(phrase_data[0].freq), 1, dict_file);
#else
		fprintf(dict_file, "%s %d\t", phrase_data[i].phrase, phrase_data[i].freq);
#endif
	}

	pos = ftell(dict_file);
#ifdef USE_BINARY_DATA
	fwrite(&pos, sizeof(pos), 1, ph_index_file);
	size = strlen(phrase_data[i].phrase);
	fwrite(&size, sizeof(size), 1, dict_file);
	fwrite(phrase_data[i].phrase, size, 1, dict_file);
	fwrite(&phrase_data[i].freq, sizeof(phrase_data[0].freq), 1, dict_file);
	pos = ftell(dict_file);
	fwrite(&pos, sizeof(pos), 1, ph_index_file);
#else
	fprintf(ph_index_file, "%d\n", pos);
	fprintf(dict_file, "%s %d", phrase_data[i].phrase, phrase_data[i].freq);
	pos = ftell(dict_file);
	fprintf(ph_index_file, "%d\n", pos);
#endif

	for (i = 0; i < num_phrase_data; ++i) {
		if (i > 0 && !compare_phone_in_phrase(i - 1, i))
			continue;

		for (j = 0; phrase_data[i].phone[j]; ++j) {
			fprintf(phoneid_file, "%hu ", phrase_data[i].phone[j]);
		}
		fprintf(phoneid_file, "0\n");

	}

	fclose(phoneid_file);
	fclose(ph_index_file);
	fclose(dict_file);
}

int main(int argc, char *argv[])
{
	if (argc != 3) {
		printf(USAGE, argv[0]);
		return -1;
	}

	read_phone_cin(argv[1]);
	write_word_data();

	sort_word_for_dictionary();

	read_tsi_src(argv[2]);
	write_phrase_data();
	return 0;
}
