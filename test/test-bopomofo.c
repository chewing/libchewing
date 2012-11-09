/**
 * test-bopomofo.c
 *
 * Copyright (c) 2012
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#include <config.h>
#endif

#include <stdlib.h>
#include <stdio.h>

#include "chewing.h"
#include "plat_types.h"
#include "hash-private.h"
#include "test.h"

void test_select_candidate()
{
	// The following phrases are in dict
	// 一上來
	// 上來
	// 移上來
	// 移上
	// 快上

	static const char *CAND_1[] = {
		"一上來",
		"移上來",
	};

	static const char *CAND_2[] = {
		"移上",
	};

	remove( TEST_HASH_DIR PLAT_SEPARATOR HASH_FILE );

	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_maxChiSymbolLen( ctx, 16 );

	type_keystoke_by_string( ctx, "u6g;4x96<L><L><L>" ); // ㄧˊㄕㄤˋㄌㄞˊ

	// ㄧˊㄕㄤˋㄌㄞˊ
	type_keystoke_by_string( ctx, "<D>" );
	ok_candidate( ctx, CAND_1, ARRAY_SIZE( CAND_1 ) );

	// ㄕㄤˋㄌㄞˊ
	type_keystoke_by_string( ctx, "<D>" );
	ok_candidate( ctx, CAND_2, ARRAY_SIZE( CAND_2 ) );

	// select 移上來
	ok_keystoke( ctx, "<D><D>2<E>", CAND_1[1] );

	chewing_Terminate();
}

void test_select_candidate_phrase_choice_rearward()
{
	// The following phrases are in dict
	// 一上來
	// 上來
	// 移上來
	// 移上
	// 快上

	static const char *CAND_1[] = {
		"一上來",
		"移上來",
	};

	static const char *CAND_2[] = {
		"上來",
		"快上", // XXX: bug?
	};

	remove( TEST_HASH_DIR PLAT_SEPARATOR HASH_FILE );

	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_phraseChoiceRearward( ctx, 1 );

	type_keystoke_by_string( ctx, "u6g;4x96" ); // ㄧˊㄕㄤˋㄌㄞˊ

	// ㄧˊㄕㄤˋㄌㄞˊ
	type_keystoke_by_string( ctx, "<D>" );
	ok_candidate( ctx, CAND_1, ARRAY_SIZE( CAND_1 ) );

	// ㄕㄤˋㄌㄞˊ
	type_keystoke_by_string( ctx, "<D>" );
	ok_candidate( ctx, CAND_2, ARRAY_SIZE( CAND_2 ) );

	// select 移上來
	ok_keystoke( ctx, "<D><D>2<E>", CAND_1[1] );

	chewing_Terminate();
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_select_candidate();
	test_select_candidate_phrase_choice_rearward();

	return exit_status();
}
