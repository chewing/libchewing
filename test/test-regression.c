/**
 * test-regression.c
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

#include <stdio.h>
#include <stdlib.h>

#include "chewing.h"
#include "plat_types.h"
#include "hash-private.h"
#include "testhelper.h"

void test_libchewing_googlecode_issue_472()
{
	/* FIXME: Add const cause gcc warning */
	static char *INPUT[] = {
		"<T>|&Wt<H>mrJY)G<C2>OqJ<H><H>Yl<R>p0<EE>QE[^<C1>k",
		"+F<C9>hQ$UIICMr!X8/9<C3>(N<T>yU2!-LUI<D>`CS<D>jShm9SF}<EN>[`QYu<C8>k",
		"hk4`2<D>jk",
		"hk4`j 0",
		"hk4<C0>j 0",
	};
	size_t i;
	ChewingContext *ctx;

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_autoShiftCur( ctx, 1 );

	for (i = 0; i < ARRAY_SIZE(INPUT); ++i) {
		chewing_Reset( ctx );
		type_keystroke_by_string( ctx, INPUT[i] );
	}

	chewing_delete( ctx );
}

void test_libchewing_googlecode_issue_473()
{
	/* FIXME: Add const cause gcc warning */
	static char *INPUT[] = {
		"t<N->_ M1<N+>H[Ls3<L><N1>PL+Z]4<C1>&(^H*H<TT>Sc<N->P]!|<CB>-<C6>S<H><N1><C0>U<B>d}P!f<EN><N.><C7>V!U!w|4-=S<C1>b<N2>Q",
		"wv<C0><C5><N9>$FIF<D><N4>B *<C2>E4*<C2>q)Kf)<SS><TT>4=<N5>%<R>mN4<EN>H<N9><N.>8s{XTD<N6>jZV(y3G`9<C6>JTy<B>J<C1>SNc<E>hC<SL><N/><R><C6>@an<C3><N7>wzF<C3>P*<N*><B>l<C3><N6>W<N*> $<SR><N.><N1><E><E><N0><N6>Y",
		
	};
	size_t i;
	ChewingContext *ctx;

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_autoShiftCur( ctx, 1 );
	chewing_set_candPerPage( ctx, 9 );
	chewing_set_addPhraseDirection( ctx, 1 );
	chewing_set_spaceAsSelection( ctx, 1 );

	for (i = 0; i < ARRAY_SIZE(INPUT); ++i) {
		chewing_Reset( ctx );
		type_keystroke_by_string( ctx, INPUT[i] );
	}

	chewing_delete( ctx );
}

void test_libchewing_issue_30()
{
	ChewingContext *ctx;
	int cursor;

	remove( TEST_HASH_DIR PLAT_SEPARATOR HASH_FILE );

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );
	chewing_set_autoShiftCur( ctx, 1 );
	chewing_set_spaceAsSelection( ctx, 1 );
	chewing_set_phraseChoiceRearward( ctx, 1 );

	type_keystroke_by_string( ctx, "hk4g4<H> 3 1" );
	cursor = chewing_cursor_Current( ctx );
	ok( cursor == 2, "cursor position `%d' shall be `2'", cursor);

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_libchewing_data_issue_1()
{
	const TestData DATA = { "e03y.3", "\xE8\xB6\x95\xE8\xB5\xB0" /* 趕走*/ };
	ChewingContext *ctx;

	remove( TEST_HASH_DIR PLAT_SEPARATOR HASH_FILE );

	chewing_Init( NULL, NULL );

	ctx = chewing_new();
	chewing_set_maxChiSymbolLen( ctx, 16 );
	type_keystroke_by_string( ctx, DATA.token );
	ok_preedit_buffer( ctx, DATA.expected );

	chewing_delete( ctx );
	chewing_Terminate();
}

int main()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_libchewing_data_issue_1();
	test_libchewing_issue_30();
	test_libchewing_googlecode_issue_472();
	test_libchewing_googlecode_issue_473();

	return exit_status();
}
