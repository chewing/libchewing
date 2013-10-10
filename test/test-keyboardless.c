/**
 * test-keyboardless-api.c
 *
 * Copyright (c) 2013
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#include <stdlib.h>

#include "testhelper.h"
#include "chewing.h"

void test_cand_open_word()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4" /* ㄘㄜˋ */ );

	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_cand_open_symbol()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "`31" /* ， */ );

	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_cand_open_already_opened()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4" /* ㄘㄜˋ */ );

	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	/* FIXME: Need to ensure the candidate windows does not change */
	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_cand_open_nothing_in_preedit()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	ret = chewing_cand_open( ctx );
	ok( ret == -1, "chewing_cand_open() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_cand_open_during_bopomofo()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	/* FIXME: shall we clean bopomofo when chewing_cand_open is called? */

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk" /* ㄘㄜ */ );

	ret = chewing_cand_open( ctx );
	ok( ret == -1, "chewing_cand_open() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0 );

	chewing_Reset( ctx );

	type_keystroke_by_string( ctx, "hk4g" /* ㄘㄜˋ ㄕ */ );
	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_cand_open()
{
	test_cand_open_word();
	test_cand_open_symbol();
	test_cand_open_already_opened();
	test_cand_open_nothing_in_preedit();
	test_cand_open_during_bopomofo();
}

void test_cand_close_word()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4" /* ㄘㄜˋ */ );

	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	ret = chewing_cand_close( ctx );
	ok( ret == 0, "chewing_cand_close() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be 0 than `%d'", ret, 0 );

	ok_commit_buffer( ctx, "" );

	chewing_delete( ctx );
}


void test_cand_close_symbol()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "`31" /* ， */ );

	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	ret = chewing_cand_close( ctx );
	ok( ret == 0, "chewing_cand_close() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be 0 than `%d'", ret, 0 );

	ok_commit_buffer( ctx, "" );

	chewing_delete( ctx );
}

void test_cand_close_already_closed()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4" /* ㄘㄜˋ */ );

	ret = chewing_cand_close( ctx );
	ok( ret == 0, "chewing_cand_close() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be 0 than `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_cand_close_nothing_in_preedit()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	ret = chewing_cand_close( ctx );
	ok( ret == 0, "chewing_cand_close() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be 0 than `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_cand_close()
{
	test_cand_close_word();
	test_cand_close_symbol();
	test_cand_close_already_closed();
	test_cand_close_nothing_in_preedit();
}

void test_cand_choose_word()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4" /* ㄘㄜˋ */ );

	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );

	ret = chewing_cand_choose_by_index( ctx, 2 );
	ok ( ret == 0, "chewing_cand_choose_by_index() returns `%d' shall be `%d'", ret, 0 );

	ok_preedit_buffer( ctx, "\xE6\xB8\xAC" /* 測 */ );

	chewing_delete( ctx );
}

void test_cand_choose_symbol()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "`" /* ， */ );

	ret = chewing_cand_choose_by_index( ctx, 2 );
	ok ( ret == 0, "chewing_cand_choose_by_index() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_choose_by_index( ctx, 0 );
	ok ( ret == 0, "chewing_cand_choose_by_index() returns `%d' shall be `%d'", ret, 0 );

	ok_preedit_buffer( ctx, "\xEF\xBC\x8C" /* ， */ );

	chewing_delete( ctx );
}

void test_cand_choose_out_of_range()
{
	ChewingContext *ctx;
	int ret;
	int total_choice;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4" /* ㄘㄜˋ */ );

	ret = chewing_cand_open( ctx );
	ok( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );

	total_choice = chewing_cand_TotalChoice( ctx );
	ok ( total_choice > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", total_choice, 0 );

	ret = chewing_cand_choose_by_index( ctx, total_choice );
	ok ( ret == -1, "chewing_cand_choose_by_index() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_cand_choose_by_index( ctx, -1 );
	ok ( ret == -1, "chewing_cand_choose_by_index() returns `%d' shall be `%d'", ret, -1 );

	ok_preedit_buffer( ctx, "\xE5\x86\x8A" /* 冊 */ );

	chewing_delete( ctx );
}

void test_cand_choose_second_layer()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "`" );
	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );
	ret = chewing_cand_choose_by_index( ctx, 2 );
	ok ( ret == 0, "chewing_cand_choose_by_index() returns `%d' shall be `%d'", ret, 0 );
	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret > 0, "chewing_cand_TotalChoice() returns `%d' shall be greater than `%d'", ret, 0 );
	ret = chewing_cand_choose_by_index( ctx, 0 );
	ok ( ret == 0, "chewing_cand_choose_by_index() returns `%d' shall be `%d'", ret, 0 );
	ok_preedit_buffer( ctx, "\xEF\xBC\x8C" /* ， */ );

	chewing_delete( ctx );
}

void test_cand_choose_not_in_select()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	clean_userphrase();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4" /* ㄘㄜˋ */ );

	ret = chewing_cand_TotalChoice( ctx );
	ok ( ret == 0, "chewing_cand_TotalChoice() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_choose_by_index( ctx, 2 );
	ok ( ret == -1, "chewing_cand_choose_by_index() returns `%d' shall be `%d'", ret, -1 );

	ok_preedit_buffer( ctx, "\xE5\x86\x8A" /* 冊 */ );

	chewing_delete( ctx );
}

void test_cand_choose() {
	test_cand_choose_word();
	test_cand_choose_symbol();
	test_cand_choose_second_layer();
	test_cand_choose_out_of_range();
	test_cand_choose_not_in_select();
}

void test_cand_list_word_no_rearward()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();
	chewing_set_phraseChoiceRearward( ctx, 0 );

	type_keystroke_by_string( ctx, "hk4g4<H>" /* 測試 */ );

	ret = chewing_cand_open( ctx );
	ok ( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );
	ok_candidate_len( ctx, 2 );

	ret = chewing_cand_list_has_next( ctx );
	ok ( ret == 1, "chewing_cand_list_has_next() returns `%d' shall be `%d'", ret, 1 );
	ret = chewing_cand_list_next( ctx );
	ok ( ret == 0, "chewing_cand_list_next() returns `%d' shall be `%d'", ret, 0 );
	ok_candidate_len( ctx, 1 );

	ret = chewing_cand_list_has_next( ctx );
	ok ( ret == 0, "chewing_cand_list_has_next() returns `%d' shall be `%d'", ret, 0 );
	ret = chewing_cand_list_next( ctx );
	ok_candidate_len( ctx, 1 );

	ret = chewing_cand_list_has_prev( ctx );
	ok ( ret == 1, "chewing_cand_list_has_prev() returns `%d' shall be `%d'", ret, 1 );
	ret = chewing_cand_list_prev( ctx );
	ok ( ret == 0, "chewing_cand_list_prev() returns `%d' shall be `%d'", ret, 0 );
	ok_candidate_len( ctx, 2 );

	ret = chewing_cand_list_has_prev( ctx );
	ok ( ret == 0, "chewing_cand_list_has_prev() returns `%d' shall be `%d'", ret, 0 );
	ret = chewing_cand_list_prev( ctx );
	ok ( ret == -1, "chewing_cand_list_prev() returns `%d' shall be `%d'", ret, -1 );
	ok_candidate_len( ctx, 2 );

	ret = chewing_cand_list_last( ctx );
	ok ( ret == 0, "chewing_cand_list_last() returns `%d' shall be `%d'", ret, 0 );
	ok_candidate_len( ctx, 1 );

	ret = chewing_cand_list_first( ctx );
	ok ( ret == 0, "chewing_cand_list_first() returns `%d' shall be `%d'", ret, 0 );
	ok_candidate_len( ctx, 2 );

	chewing_delete( ctx );
}

void test_cand_list_word_rearward()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();
	chewing_set_phraseChoiceRearward( ctx, 1 );

	type_keystroke_by_string( ctx, "hk4g4" /* 測試 */ );

	ret = chewing_cand_open( ctx );
	ok ( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );
	ok_candidate_len( ctx, 2 );

	ret = chewing_cand_list_has_next( ctx );
	ok ( ret == 1, "chewing_cand_list_has_next() returns `%d' shall be `%d'", ret, 1 );
	ret = chewing_cand_list_next( ctx );
	ok ( ret == 0, "chewing_cand_list_next() returns `%d' shall be `%d'", ret, 0 );
	ok_candidate_len( ctx, 1 );

	ret = chewing_cand_list_has_next( ctx );
	ok ( ret == 0, "chewing_cand_list_has_next() returns `%d' shall be `%d'", ret, 0 );
	ret = chewing_cand_list_next( ctx );
	ok_candidate_len( ctx, 1 );

	ret = chewing_cand_list_has_prev( ctx );
	ok ( ret == 1, "chewing_cand_list_has_prev() returns `%d' shall be `%d'", ret, 1 );
	ret = chewing_cand_list_prev( ctx );
	ok ( ret == 0, "chewing_cand_list_prev() returns `%d' shall be `%d'", ret, 0 );
	ok_candidate_len( ctx, 2 );

	ret = chewing_cand_list_has_prev( ctx );
	ok ( ret == 0, "chewing_cand_list_has_prev() returns `%d' shall be `%d'", ret, 0 );
	ret = chewing_cand_list_prev( ctx );
	ok ( ret == -1, "chewing_cand_list_prev() returns `%d' shall be `%d'", ret, -1 );
	ok_candidate_len( ctx, 2 );

	ret = chewing_cand_list_last( ctx );
	ok ( ret == 0, "chewing_cand_list_last() returns `%d' shall be `%d'", ret, 0 );
	ok_candidate_len( ctx, 1 );

	ret = chewing_cand_list_first( ctx );
	ok ( ret == 0, "chewing_cand_list_first() returns `%d' shall be `%d'", ret, 0 );
	ok_candidate_len( ctx, 2 );

	chewing_delete( ctx );
}

void test_cand_list_symbol()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();
	type_keystroke_by_string( ctx, "`31" /* ， */ );

	ret = chewing_cand_open( ctx );
	ok ( ret == 0, "chewing_cand_open() returns `%d' shall be `%d'", ret, 0 );
	ok_candidate_len( ctx, 1 );

	ret = chewing_cand_list_has_next( ctx );
	ok ( ret == 0, "chewing_cand_list_has_next() returns `%d' shall be `%d'", ret, 0 );
	ret = chewing_cand_list_next( ctx );
	ok ( ret == -1, "chewing_cand_list_has_next() returns `%d' shall be `%d'", ret, -1 );
	ok_candidate_len( ctx, 1 );

	ret = chewing_cand_list_has_prev( ctx );
	ok ( ret == 0, "chewing_cand_list_has_prev() returns `%d' shall be `%d'", ret, 0 );
	ret = chewing_cand_list_prev( ctx );
	ok ( ret == -1, "chewing_cand_list_prev() returns `%d' shall be `%d'", ret, -1 );
	ok_candidate_len( ctx, 1 );

	ret = chewing_cand_list_first( ctx );
	ok ( ret == 0, "chewing_cand_list_first() returns `%d' shall be `%d'", ret, 0 );
	ok_candidate_len( ctx, 1 );

	ret = chewing_cand_list_last( ctx );
	ok ( ret == 0, "chewing_cand_list_last() returns `%d' shall be `%d'", ret, 0 );
	ok_candidate_len( ctx, 1 );

	chewing_delete( ctx );
}

void test_cand_list_no_cand_windows()
{
	ChewingContext *ctx;
	int ret;

	print_function_name();

	ctx = chewing_new();

	type_keystroke_by_string( ctx, "hk4g4" /* 測試 */ );

	ret = chewing_cand_list_has_next( ctx );
	ok ( ret == 0, "chewing_cand_list_has_next() returns `%d' shall be `%d'", ret, 0 );
	ret = chewing_cand_list_next( ctx );
	ok ( ret == -1, "chewing_cand_list_next() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_cand_list_has_prev( ctx );
	ok ( ret == 0, "chewing_cand_list_has_prev() returns `%d' shall be `%d'", ret, 0 );
	ret = chewing_cand_list_prev( ctx );
	ok ( ret == -1, "chewing_cand_list_prev() returns `%d' shall be `%d'", ret, -1 );

	ret = chewing_cand_list_first( ctx );
	ok ( ret == -1, "chewing_cand_list_first() returns `%d' shall be `%d'", ret, 0 );

	ret = chewing_cand_list_last( ctx );
	ok ( ret == -1, "chewing_cand_list_last() returns `%d' shall be `%d'", ret, 0 );

	chewing_delete( ctx );
}

void test_cand_list()
{
	test_cand_list_word_no_rearward();
	test_cand_list_word_rearward();
	test_cand_list_symbol();
	test_cand_list_no_cand_windows();
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_cand_open();
	test_cand_close();
	test_cand_choose();
	test_cand_list();

	return exit_status();
}
