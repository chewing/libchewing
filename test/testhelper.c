/**
 * testhelper.c
 *
 * Copyright (c) 2012
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#include "testhelper.h"

#include <assert.h>
#include <errno.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "chewing-private.h"
#include "chewing-utf8-util.h"
#include "key2pho-private.h"
#include "plat_path.h"
#include "userphrase-private.h"

static unsigned int test_run;
static unsigned int test_ok;

TestKeyEntry chewing_test_special_keys[] = {
  { KEY_LEFT,      "<L>",  chewing_handle_Left },
  { KEY_SLEFT,     "<SL>", chewing_handle_ShiftLeft },
  { KEY_RIGHT,     "<R>",  chewing_handle_Right },
  { KEY_SRIGHT,    "<SR>", chewing_handle_ShiftRight },
  { KEY_UP,        "<U>",  chewing_handle_Up },
  { KEY_DOWN,      "<D>",  chewing_handle_Down },
  { KEY_SPACE,     " ",    chewing_handle_Space },
  { KEY_ENTER,     "<E>",  chewing_handle_Enter },
  { KEY_BACKSPACE, "<B>",  chewing_handle_Backspace },
  { KEY_ESC,       "<EE>", chewing_handle_Esc },
  { KEY_DELETE,    "<DC>", chewing_handle_Del },
  { KEY_HOME,      "<H>",  chewing_handle_Home },
  { KEY_END,       "<EN>", chewing_handle_End },
  { KEY_TAB,       "<T>",  chewing_handle_Tab },
  { KEY_CAPSLOCK,  "<CB>", chewing_handle_Capslock },
  { KEY_NPAGE,     "<PD>", chewing_handle_PageDown },
  { KEY_PPAGE,     "<PU>", chewing_handle_PageUp },
  { KEY_SSPACE,    "<SS>", chewing_handle_ShiftSpace },
  { KEY_DBLTAB,    "<TT>", chewing_handle_DblTab },
  { KEY_CTRL_BASE + '0', "<C0>", NULL },
  { KEY_CTRL_BASE + '1', "<C1>", NULL },
  { KEY_CTRL_BASE + '2', "<C2>", NULL },
  { KEY_CTRL_BASE + '3', "<C3>", NULL },
  { KEY_CTRL_BASE + '4', "<C4>", NULL },
  { KEY_CTRL_BASE + '5', "<C5>", NULL },
  { KEY_CTRL_BASE + '6', "<C6>", NULL },
  { KEY_CTRL_BASE + '7', "<C7>", NULL },
  { KEY_CTRL_BASE + '8', "<C8>", NULL },
  { KEY_CTRL_BASE + '9', "<C9>", NULL },
  { KEY_NUMPAD_BASE + '0', "<N0>", NULL },
  { KEY_NUMPAD_BASE + '1', "<N1>", NULL },
  { KEY_NUMPAD_BASE + '2', "<N2>", NULL },
  { KEY_NUMPAD_BASE + '3', "<N3>", NULL },
  { KEY_NUMPAD_BASE + '4', "<N4>", NULL },
  { KEY_NUMPAD_BASE + '5', "<N5>", NULL },
  { KEY_NUMPAD_BASE + '6', "<N6>", NULL },
  { KEY_NUMPAD_BASE + '7', "<N7>", NULL },
  { KEY_NUMPAD_BASE + '8', "<N8>", NULL },
  { KEY_NUMPAD_BASE + '9', "<N9>", NULL },
  { KEY_NUMPAD_BASE + '+', "<N+>", NULL },
  { KEY_NUMPAD_BASE + '-', "<N->", NULL },
  { KEY_NUMPAD_BASE + '*', "<N*>", NULL },
  { KEY_NUMPAD_BASE + '/', "<N/>", NULL },
  { KEY_NUMPAD_BASE + '.', "<N.>", NULL },
  { 0, NULL, NULL },
};

/* We cannot use designated initializer here due to Visual Studio */
BufferType COMMIT_BUFFER = {
    "commit buffer",
    chewing_commit_Check,
    0,
    0,
    chewing_commit_String,
    0,
    chewing_commit_String_static
};

BufferType PREEDIT_BUFFER = {
    "preedit buffer",
    chewing_buffer_Check,
    0,
    chewing_buffer_Len,
    chewing_buffer_String,
    0,
    chewing_buffer_String_static
};

BEGIN_IGNORE_DEPRECATIONS

BufferType BOPOMOFO_BUFFER = {
    "bopomofo buffer",
    chewing_bopomofo_Check,
    chewing_zuin_Check,
    0,
    0,
    chewing_zuin_String,
    chewing_bopomofo_String_static
};

END_IGNORE_DEPRECATIONS

BufferType AUX_BUFFER = {
    "aux buffer",
    chewing_aux_Check,
    0,
    chewing_aux_Length,
    chewing_aux_String,
    0,
    0,
};

int get_keystroke(get_char_func get_char, void *param)
{
    TestKeyEntry *key_entry;
    int ch;
    char current_key[10];
    int current_keylen = 0;
    int partial_match;

    assert(get_char);

    while ((ch = get_char(param)) != END) {
        current_key[current_keylen++] = ch;
        current_key[current_keylen] = '\0';

        partial_match = 0;
        for (key_entry = chewing_test_special_keys; key_entry->key; key_entry++) {
            if (strcmp(key_entry->str, current_key) == 0) {
                current_keylen = 0;
                return key_entry->key;
            }
            if (strncmp(key_entry->str, current_key, current_keylen) == 0)
                partial_match = 1;
        }

        /* special case: partial match but not special key */
        if (strcmp(current_key, "<<") == 0 || strcmp(current_key, "<>") == 0) {
            partial_match = 1;
            continue;
        }
        if (strcmp(current_key, "<<>") == 0 || strcmp(current_key, "<>>") == 0) {
            current_keylen = 0;
            return current_key[1];
        }


        if (partial_match)
            continue;

        if (current_keylen > 1) {
            fprintf(stderr, "unknown key: '%s'\n", current_key);
        }

        return current_key[0];
    }
    return END;
}

void type_single_keystroke(ChewingContext *ctx, int key)
{
    TestKeyEntry *key_entry;

    for (key_entry = chewing_test_special_keys; key_entry->key; key_entry++) {
        if (key_entry->key == key && key_entry->handler) {
            key_entry->handler(ctx);
            return;
        }
    }

    if (KEY_CTRL_BASE <= key && key < KEY_NUMPAD_BASE)
        chewing_handle_CtrlNum(ctx, key - KEY_CTRL_BASE);
    else if (KEY_NUMPAD_BASE <= key)
        chewing_handle_Numlock(ctx, key - KEY_NUMPAD_BASE);
    else
        chewing_handle_Default(ctx, (char) key);
}

static void type_keystroke(ChewingContext *ctx, get_char_func get_char, void *param)
{
    int ch;

    while ((ch = get_keystroke(get_char, param)) != END)
        type_single_keystroke(ctx, ch);
}

int get_char_by_string(void *param)
{
    const char **ptr = param;
    char ch;

    assert(param);

    if (**ptr == 0) {
        return END;
    }

    ch = **ptr;
    ++*ptr;
    return ch;
}

int get_char_from_stdin(void *param UNUSED)
{
    int ch = getchar();

    if (ch == EOF)
        return END;
    return ch;
}

int get_char_from_fp(void *param)
{
    FILE *fp = param;

    assert(fp);
    int ch = fgetc(fp);

    if (ch == EOF)
        return END;
    return ch;
}

void internal_ok(const char *file, int line, int test, const char *test_txt, const char *fmt, ...)
{
    va_list ap;

    ++test_run;
    if (test) {
        ++test_ok;
        printf("ok %u ", test_run);

        va_start(ap, fmt);
        vprintf(fmt, ap);
        va_end(ap);

        printf("\n");
    } else {
        printf("not ok %u ", test_run);

        va_start(ap, fmt);
        vprintf(fmt, ap);
        va_end(ap);

        printf("\n# %s failed in %s:%d\n", test_txt, file, line);
    }
}

void type_keystroke_by_string(ChewingContext *ctx, const char *keystroke)
{
    type_keystroke(ctx, get_char_by_string, &keystroke);
}

void internal_ok_buffer(const char *file, int line, ChewingContext *ctx, const char *expected, const BufferType *buffer)
{
    char *buf;
    const char *const_buf;
    int actual_ret;
    int expected_ret;
    int expected_len;

    assert(ctx);
    assert(expected);
    assert(buffer);

    expected_len = ueStrLen(expected);

    if (buffer->check) {
        actual_ret = buffer->check(ctx);
        expected_ret = ! !expected_len;
        internal_ok(file, line, actual_ret == expected_ret,
                    "actual_ret == expected_ret",
                    "%s check function returned `%d' shall be `%d'", buffer->name, actual_ret, expected_ret);
    }

    if (buffer->check_alt) {
        actual_ret = buffer->check_alt(ctx);
        expected_ret = !expected_len;
        internal_ok(file, line, actual_ret == expected_ret,
                    "actual_ret == expected_ret",
                    "%s check function returned `%d' shall be `%d'", buffer->name, actual_ret, expected_ret);
    }

    if (buffer->get_length) {
        actual_ret = buffer->get_length(ctx);
        expected_ret = expected_len;
        internal_ok(file, line, actual_ret == expected_ret,
                    "actual_ret == expected_ret",
                    "%s get length function returned `%d' shall be `%d'", buffer->name, actual_ret, expected_ret);
    }

    if (buffer->get_string) {
        buf = buffer->get_string(ctx);
        internal_ok(file, line, !strcmp(buf, expected), "!strcmp( buf, expected )",
                    "%s string function returned `%s' shall be `%s'", buffer->name, buf, expected);
        chewing_free(buf);
    }

    if (buffer->get_string_alt) {
        buf = buffer->get_string_alt(ctx, &actual_ret);
        expected_ret = expected_len;
        internal_ok(file, line, actual_ret == expected_ret,
                    "actual_ret == expected_ret",
                    "%s string function returned parameter `%d' shall be `%d'", buffer->name, actual_ret, expected_ret);
        internal_ok(file, line, !strcmp(buf, expected), "!strcmp( buf, expected )",
                    "%s string function returned `%s' shall be `%s'", buffer->name, buf, expected);
        chewing_free(buf);
    }

    if (buffer->get_string_static) {
        const_buf = buffer->get_string_static(ctx);
        internal_ok(file, line, !strcmp(const_buf, expected), "!strcmp( const_buf, expected )",
                    "%s string function returned `%s' shall be `%s'", buffer->name, const_buf, expected);
    }
}

void internal_ok_candidate(const char *file, int line, ChewingContext *ctx, const char *cand[], size_t cand_len)
{
    size_t i;
    char *buf;
    const char *const_buf;

    assert(ctx);

    chewing_cand_Enumerate(ctx);
    for (i = 0; i < cand_len; ++i) {
        internal_ok(file, line, chewing_cand_hasNext(ctx), __func__, "shall has next candidate");

        buf = chewing_cand_String(ctx);
        internal_ok(file, line, strcmp(buf, cand[i]) == 0, __func__, "candidate `%s' shall be `%s'", buf, cand[i]);
        chewing_free(buf);

        const_buf = chewing_cand_string_by_index_static(ctx, i);
        internal_ok(file, line, strcmp(const_buf, cand[i]) == 0, __func__,
                    "candndate `%s' shall be `%s'", const_buf, cand[i]);
    }

    internal_ok(file, line, !chewing_cand_hasNext(ctx), __func__, "shall not have next candidate");
    buf = chewing_cand_String(ctx);

    internal_ok(file, line, strcmp(buf, "") == 0, __func__, "candndate `%s' shall be `%s'", buf, "");

    const_buf = chewing_cand_string_by_index_static(ctx, i);
    internal_ok(file, line, strcmp(const_buf, "") == 0, __func__, "candndate `%s' shall be `%s'", const_buf, "");

    chewing_free(buf);
}

void internal_ok_candidate_len(const char *file, int line, ChewingContext *ctx, size_t expected_len)
{
    const char *buf;
    size_t actual_len;

    assert(ctx);

    buf = chewing_cand_string_by_index_static(ctx, 0);
    actual_len = ueStrLen(buf);
    internal_ok(file, line, actual_len == expected_len, __func__,
                "candidate length `%d' shall be `%d'", actual_len, expected_len);
}

void internal_ok_keystroke_rtn(const char *file, int line, ChewingContext *ctx, int rtn)
{
    const struct {
        int rtn;
        int (*func) (const ChewingContext *ctx);
    } TABLE[] = {
        {KEYSTROKE_IGNORE, chewing_keystroke_CheckIgnore},
        {KEYSTROKE_COMMIT, chewing_commit_Check},
        // No function to check KEYSTROKE_BELL
        {KEYSTROKE_ABSORB, chewing_keystroke_CheckAbsorb},
    };
    size_t i;
    int actual;
    int expected;

    assert(ctx);

    for (i = 0; i < ARRAY_SIZE(TABLE); ++i) {
        actual = TABLE[i].func(ctx);
        expected = ! !(rtn & TABLE[i].rtn);

        internal_ok(file, line, actual == expected, __func__, "keystroke rtn `%d' shall be `%d'", actual, expected);
    }
}

void logger(void *data, int level UNUSED, const char *fmt, ...)
{
    va_list ap;
    FILE *fd = (FILE *) data;

    va_start(ap, fmt);
    vfprintf(fd, fmt, ap);
    va_end(ap);
}

void internal_start_testcase(const char *func, ChewingContext *ctx, FILE * file)
{
    assert(func);

    printf("#\n# %s\n#\n", func);
    fprintf(file, "#\n# %s\n#\n", func);
    chewing_set_logger(ctx, logger, file);
}

int exit_status()
{
    return test_run == test_ok ? 0 : -1;
}

char *get_test_userphrase_path()
{
    char *userphrase_path = getenv("TEST_USERPHRASE_PATH");

    if (userphrase_path)
        return userphrase_path;
    else
        return TEST_HASH_DIR PLAT_SEPARATOR DB_NAME;
}

void clean_userphrase()
{
    char *userphrase_path = get_test_userphrase_path();

    if (remove(userphrase_path) != 0 && errno != ENOENT)
        fprintf(stderr, "remove fails at %s:%d\n", __FILE__, __LINE__);
}
