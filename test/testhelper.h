/**
 * testhelper.h
 * * Copyright (c) 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#include <stddef.h>
#include <stdio.h>

#include "chewing.h"
#include "chewingio.h"
#include "chewing-utf8-util.h"

#include "plat_path.h"

#define KEY_DBLTAB	892     // <TT>
#define KEY_SSPACE	893     // <SS>
#define KEY_PPAGE	894     // <PU>
#define KEY_NPAGE	895     // <PD>
#define KEY_SLEFT	896     // <SL>
#define KEY_SRIGHT	897     // <SR>
#define KEY_LEFT	898     // <L>
#define KEY_RIGHT	899     // <R>
#define KEY_UP		990     // <U>
#define KEY_DOWN	991     // <D>
#define KEY_SPACE	' '
#define KEY_ENTER	992     // <E>
#define KEY_BACKSPACE	993     // <B>
#define KEY_ESC		994     // <EE>
#define KEY_DELETE	995     // <DC>
#define KEY_HOME	996     // <H>
#define KEY_END		997     // <EN>
#define KEY_TAB		998     // <T>
#define KEY_CAPSLOCK	999     // <CB>
#define KEY_CTRL_BASE	1000    // <C0>..<C9>
#define KEY_NUMPAD_BASE	1100    // <N0>..<N9>,<N+>,<N->,<N*>,<N/>,<N.>
#define END 2000

#ifdef __GNUC__
#    define ARRAY_SIZE( array ) ( sizeof(array) / sizeof(((typeof(array)){})[0]) )
#else
#    define ARRAY_SIZE( array ) ( sizeof(array) / sizeof(array[0] ) )
#endif

#define ok(test, fmt, ...) \
    internal_ok(__FILE__, __LINE__, !!(test), #test, fmt, ##__VA_ARGS__)
#define ok_commit_buffer(ctx, expected) \
    internal_ok_buffer(__FILE__, __LINE__, ctx, expected, &COMMIT_BUFFER)
#define ok_preedit_buffer(ctx, expected) \
    internal_ok_buffer(__FILE__, __LINE__, ctx, expected, &PREEDIT_BUFFER)
#define ok_bopomofo_buffer(ctx, expected) \
    internal_ok_buffer(__FILE__, __LINE__, ctx, expected, &BOPOMOFO_BUFFER)
#define ok_aux_buffer(ctx, expected) \
    internal_ok_buffer(__FILE__, __LINE__, ctx, expected, &AUX_BUFFER)
#define ok_candidate(ctx, cand, cand_len) \
    internal_ok_candidate(__FILE__, __LINE__, ctx, cand, cand_len)
#define ok_candidate_len(ctx, expected_len) \
    internal_ok_candidate_len(__FILE__, __LINE__, ctx, expected_len)
#define ok_keystroke_rtn(ctx, rtn) \
    internal_ok_keystroke_rtn(__FILE__, __LINE__, ctx, rtn)
#define has_userphrase(ctx, bopomofo, phrase) \
    chewing_userphrase_lookup(ctx, phrase, bopomofo)
#define start_testcase(ctx, file) \
    internal_start_testcase(__func__, ctx, file)

typedef struct {
    /* Key code. This code is only valid in testing framework. */
    int key;

    /* key serialed as string */
    const char *str;

    /* chewing handling function. NULL for special functions */
    int (*handler) (ChewingContext *ctx);
} TestKeyEntry;

typedef struct TestData {
    char *token;
    char *expected;
} TestData;

typedef struct BufferType {
    const char *name;
    int (*check) (const ChewingContext *ctx);
    int (*check_alt) (const ChewingContext *ctx);
    int (*get_length) (const ChewingContext *ctx);
    char *(*get_string) (const ChewingContext *ctx);
    char *(*get_string_alt) (const ChewingContext *ctx, int *len);
    const char *(*get_string_static) (const ChewingContext *ctx);
} BufferType;

extern BufferType COMMIT_BUFFER;
extern BufferType PREEDIT_BUFFER;
extern BufferType BOPOMOFO_BUFFER;
extern BufferType AUX_BUFFER;
extern TestKeyEntry chewing_test_special_keys[];

typedef int (*get_char_func) (void *param);

int get_char_by_string(void *param);
int get_char_from_stdin(void *param);
int get_char_from_fp(void *param);
int get_keystroke(get_char_func get_char, void *param);
void type_keystroke_by_string(ChewingContext *ctx, const char *keystroke);
void type_single_keystroke(ChewingContext *ctx, int ch);
int exit_status();
char *get_test_userphrase_path();
void clean_userphrase();

// The internal_xxx function shall be used indirectly by macro in order to
// get correct __FILE__ and __LINE__ information.
void internal_ok_buffer(const char *file, int line, ChewingContext *ctx,
                        const char *expected, const BufferType *buffer);
void internal_ok(const char *file, int line, int test, const char *test_txt, const char *message, ...);
void internal_ok_candidate(const char *file, int line, ChewingContext *ctx, const char *cand[], size_t cand_len);
void internal_ok_candidate_len(const char *file, int line, ChewingContext *ctx, size_t expected_len);
void internal_ok_keystroke_rtn(const char *file, int line, ChewingContext *ctx, int rtn);
void internal_start_testcase(const char *func, ChewingContext *ctx, FILE * file);
void logger(void *data, int level, const char *fmt, ...);
