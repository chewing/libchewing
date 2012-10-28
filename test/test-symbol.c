/**
 * test-symbol.c
 *
 * Copyright (c) 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#include <config.h>
#endif

#include <stdlib.h>
#include <string.h>

#include "chewing.h"
#include "test.h"

static const TestData SYMBOL[] = {
	{ .token = "`1<E>", .expected = "…" },
	{ .token = "`2<E>", .expected = "※" },
	{ .token = "`31<E>", .expected = "，" },
	{ .token = "`32<E>", .expected = "、" },
	{ .token = "`33<E>", .expected = "。" },
	{ .token = "`34<E>", .expected = "．" },
	{ .token = "`35<E>", .expected = "？" },
	{ .token = "`36<E>", .expected = "！" },
	{ .token = "`37<E>", .expected = "；" },
	{ .token = "`38<E>", .expected = "︰" },
	{ .token = "`39<E>", .expected = "‧" },
	{ .token = "`30<E>", .expected = "‥" },
	{ .token = "`3<R>1<E>", .expected = "﹐" },
	{ .token = "`3<R>2<E>", .expected = "﹒" },
	{ .token = "`3<R>3<E>", .expected = "˙" },
	{ .token = "`3<R>4<E>", .expected = "·" },
	{ .token = "`3<R>5<E>", .expected = "‘" },
	{ .token = "`3<R>6<E>", .expected = "’" },
	{ .token = "`3<R>7<E>", .expected = "“" },
	{ .token = "`3<R>8<E>", .expected = "”" },
	{ .token = "`3<R>9<E>", .expected = "〝" },
	{ .token = "`3<R>0<E>", .expected = "〞" },
	{ .token = "`3<R><R>1<E>", .expected = "‵" },
	{ .token = "`3<R><R>2<E>", .expected = "′" },
	{ .token = "`3<R><R>3<E>", .expected = "〃" },
	{ .token = "`3<R><R>4<E>", .expected = "～" },
	{ .token = "`3<R><R>5<E>", .expected = "＄" },
	{ .token = "`3<R><R>6<E>", .expected = "％" },
	{ .token = "`3<R><R>7<E>", .expected = "＠" },
	{ .token = "`3<R><R>8<E>", .expected = "＆" },
	{ .token = "`3<R><R>9<E>", .expected = "＃" },
	{ .token = "`3<R><R>0<E>", .expected = "＊" },
	{ .token = "`41<E>", .expected = "（" },
	{ .token = "`42<E>", .expected = "）" },
	{ .token = "`43<E>", .expected = "「" },
	{ .token = "`44<E>", .expected = "」" },
	{ .token = "`45<E>", .expected = "〔" },
	{ .token = "`46<E>", .expected = "〕" },
	{ .token = "`47<E>", .expected = "｛" },
	{ .token = "`48<E>", .expected = "｝" },
	{ .token = "`49<E>", .expected = "〈" },
	{ .token = "`40<E>", .expected = "〉" },
	{ .token = "`4<R>1<E>", .expected = "『" },
	{ .token = "`4<R>2<E>", .expected = "』" },
	{ .token = "`4<R>3<E>", .expected = "《" },
	{ .token = "`4<R>4<E>", .expected = "》" },
	{ .token = "`4<R>5<E>", .expected = "【" },
	{ .token = "`4<R>6<E>", .expected = "】" },
	{ .token = "`4<R>7<E>", .expected = "﹙" },
	{ .token = "`4<R>8<E>", .expected = "﹚" },
	{ .token = "`4<R>9<E>", .expected = "﹝" },
	{ .token = "`4<R>0<E>", .expected = "﹞" },
	{ .token = "`4<R><R>1<E>", .expected = "﹛" },
	{ .token = "`4<R><R>2<E>", .expected = "﹜" },
	{ .token = "`51<E>", .expected = "︵" },
	{ .token = "`52<E>", .expected = "︶" },
	{ .token = "`53<E>", .expected = "﹁" },
	{ .token = "`54<E>", .expected = "﹂" },
	{ .token = "`55<E>", .expected = "︹" },
	{ .token = "`56<E>", .expected = "︺" },
	{ .token = "`57<E>", .expected = "︷" },
	{ .token = "`58<E>", .expected = "︸" },
	{ .token = "`59<E>", .expected = "︿" },
	{ .token = "`50<E>", .expected = "﹀" },
	{ .token = "`5<R>1<E>", .expected = "﹃" },
	{ .token = "`5<R>2<E>", .expected = "﹄" },
	{ .token = "`5<R>3<E>", .expected = "︽" },
	{ .token = "`5<R>4<E>", .expected = "︾" },
	{ .token = "`5<R>5<E>", .expected = "︻" },
	{ .token = "`5<R>6<E>", .expected = "︼" },
	{ .token = "`61<E>", .expected = "α" },
	{ .token = "`62<E>", .expected = "β" },
	{ .token = "`63<E>", .expected = "γ" },
	{ .token = "`64<E>", .expected = "δ" },
	{ .token = "`65<E>", .expected = "ε" },
	{ .token = "`66<E>", .expected = "ζ" },
	{ .token = "`67<E>", .expected = "η" },
	{ .token = "`68<E>", .expected = "θ" },
	{ .token = "`69<E>", .expected = "ι" },
	{ .token = "`60<E>", .expected = "κ" },
	{ .token = "`6<R>1<E>", .expected = "λ" },
	{ .token = "`6<R>2<E>", .expected = "μ" },
	{ .token = "`6<R>3<E>", .expected = "ν" },
	{ .token = "`6<R>4<E>", .expected = "ξ" },
	{ .token = "`6<R>5<E>", .expected = "ο" },
	{ .token = "`6<R>6<E>", .expected = "π" },
	{ .token = "`6<R>7<E>", .expected = "ρ" },
	{ .token = "`6<R>8<E>", .expected = "σ" },
	{ .token = "`6<R>9<E>", .expected = "τ" },
	{ .token = "`6<R>0<E>", .expected = "υ" },
	{ .token = "`6<R><R>1<E>", .expected = "φ" },
	{ .token = "`6<R><R>2<E>", .expected = "χ" },
	{ .token = "`6<R><R>3<E>", .expected = "ψ" },
	{ .token = "`6<R><R>4<E>", .expected = "ω" },
	{ .token = "`6<R><R>5<E>", .expected = "Α" },
	{ .token = "`6<R><R>6<E>", .expected = "Β" },
	{ .token = "`6<R><R>7<E>", .expected = "Γ" },
	{ .token = "`6<R><R>8<E>", .expected = "Δ" },
	{ .token = "`6<R><R>9<E>", .expected = "Ε" },
	{ .token = "`6<R><R>0<E>", .expected = "Ζ" },
	{ .token = "`6<R><R><R>1<E>", .expected = "Η" },
	{ .token = "`6<R><R><R>2<E>", .expected = "Θ" },
	{ .token = "`6<R><R><R>3<E>", .expected = "Ι" },
	{ .token = "`6<R><R><R>4<E>", .expected = "Κ" },
	{ .token = "`6<R><R><R>5<E>", .expected = "Λ" },
	{ .token = "`6<R><R><R>6<E>", .expected = "Μ" },
	{ .token = "`6<R><R><R>7<E>", .expected = "Ν" },
	{ .token = "`6<R><R><R>8<E>", .expected = "Ξ" },
	{ .token = "`6<R><R><R>9<E>", .expected = "Ο" },
	{ .token = "`6<R><R><R>0<E>", .expected = "Π" },
	{ .token = "`6<R><R><R><R>1<E>", .expected = "Ρ" },
	{ .token = "`6<R><R><R><R>2<E>", .expected = "Σ" },
	{ .token = "`6<R><R><R><R>3<E>", .expected = "Τ" },
	{ .token = "`6<R><R><R><R>4<E>", .expected = "Υ" },
	{ .token = "`6<R><R><R><R>5<E>", .expected = "Φ" },
	{ .token = "`6<R><R><R><R>6<E>", .expected = "Χ" },
	{ .token = "`6<R><R><R><R>7<E>", .expected = "Ψ" },
	{ .token = "`6<R><R><R><R>8<E>", .expected = "Ω" },
	{ .token = "`71<E>", .expected = "＋" },
	{ .token = "`72<E>", .expected = "－" },
	{ .token = "`73<E>", .expected = "×" },
	{ .token = "`74<E>", .expected = "÷" },
	{ .token = "`75<E>", .expected = "＝" },
	{ .token = "`76<E>", .expected = "≠" },
	{ .token = "`77<E>", .expected = "≒" },
	{ .token = "`78<E>", .expected = "∞" },
	{ .token = "`79<E>", .expected = "±" },
	{ .token = "`70<E>", .expected = "√" },
	{ .token = "`7<R>1<E>", .expected = "＜" },
	{ .token = "`7<R>2<E>", .expected = "＞" },
	{ .token = "`7<R>3<E>", .expected = "﹤" },
	{ .token = "`7<R>4<E>", .expected = "﹥" },
	{ .token = "`7<R>5<E>", .expected = "≦" },
	{ .token = "`7<R>6<E>", .expected = "≧" },
	{ .token = "`7<R>7<E>", .expected = "∩" },
	{ .token = "`7<R>8<E>", .expected = "∪" },
	{ .token = "`7<R>9<E>", .expected = "ˇ" },
	{ .token = "`7<R>0<E>", .expected = "⊥" },
	{ .token = "`7<R><R>1<E>", .expected = "∠" },
	{ .token = "`7<R><R>2<E>", .expected = "∟" },
	{ .token = "`7<R><R>3<E>", .expected = "⊿" },
	{ .token = "`7<R><R>4<E>", .expected = "㏒" },
	{ .token = "`7<R><R>5<E>", .expected = "㏑" },
	{ .token = "`7<R><R>6<E>", .expected = "∫" },
	{ .token = "`7<R><R>7<E>", .expected = "∮" },
	{ .token = "`7<R><R>8<E>", .expected = "∵" },
	{ .token = "`7<R><R>9<E>", .expected = "∴" },
	{ .token = "`7<R><R>0<E>", .expected = "╳" },
	{ .token = "`7<R><R><R>1<E>", .expected = "﹢" },
	{ .token = "`81<E>", .expected = "↑" },
	{ .token = "`82<E>", .expected = "↓" },
	{ .token = "`83<E>", .expected = "←" },
	{ .token = "`84<E>", .expected = "→" },
	{ .token = "`85<E>", .expected = "↖" },
	{ .token = "`86<E>", .expected = "↗" },
	{ .token = "`87<E>", .expected = "↙" },
	{ .token = "`88<E>", .expected = "↘" },
	{ .token = "`89<E>", .expected = "㊣" },
	{ .token = "`80<E>", .expected = "◎" },
	{ .token = "`8<R>1<E>", .expected = "○" },
	{ .token = "`8<R>2<E>", .expected = "●" },
	{ .token = "`8<R>3<E>", .expected = "⊕" },
	{ .token = "`8<R>4<E>", .expected = "⊙" },
	{ .token = "`8<R>5<E>", .expected = "○" },
	{ .token = "`8<R>6<E>", .expected = "●" },
	{ .token = "`8<R>7<E>", .expected = "△" },
	{ .token = "`8<R>8<E>", .expected = "▲" },
	{ .token = "`8<R>9<E>", .expected = "☆" },
	{ .token = "`8<R>0<E>", .expected = "★" },
	{ .token = "`8<R><R>1<E>", .expected = "◇" },
	{ .token = "`8<R><R>2<E>", .expected = "◆" },
	{ .token = "`8<R><R>3<E>", .expected = "□" },
	{ .token = "`8<R><R>4<E>", .expected = "■" },
	{ .token = "`8<R><R>5<E>", .expected = "▽" },
	{ .token = "`8<R><R>6<E>", .expected = "▼" },
	{ .token = "`8<R><R>7<E>", .expected = "§" },
	{ .token = "`8<R><R>8<E>", .expected = "￥" },
	{ .token = "`8<R><R>9<E>", .expected = "〒" },
	{ .token = "`8<R><R>0<E>", .expected = "￠" },
	{ .token = "`8<R><R><R>1<E>", .expected = "￡" },
	{ .token = "`8<R><R><R>2<E>", .expected = "※" },
	{ .token = "`8<R><R><R>3<E>", .expected = "♀" },
	{ .token = "`8<R><R><R>4<E>", .expected = "♂" },
	{ .token = "`91<E>", .expected = "♨" },
	{ .token = "`92<E>", .expected = "☀" },
	{ .token = "`93<E>", .expected = "☁" },
	{ .token = "`94<E>", .expected = "☂" },
	{ .token = "`95<E>", .expected = "☃" },
	{ .token = "`96<E>", .expected = "♠" },
	{ .token = "`97<E>", .expected = "♥" },
	{ .token = "`98<E>", .expected = "♣" },
	{ .token = "`99<E>", .expected = "♦" },
	{ .token = "`90<E>", .expected = "♩" },
	{ .token = "`9<R>1<E>", .expected = "♪" },
	{ .token = "`9<R>2<E>", .expected = "♫" },
	{ .token = "`9<R>3<E>", .expected = "♬" },
	{ .token = "`9<R>4<E>", .expected = "☺" },
	{ .token = "`9<R>5<E>", .expected = "☻" },
	{ .token = "`01<E>", .expected = "├" },
	{ .token = "`02<E>", .expected = "─" },
	{ .token = "`03<E>", .expected = "┼" },
	{ .token = "`04<E>", .expected = "┴" },
	{ .token = "`05<E>", .expected = "┬" },
	{ .token = "`06<E>", .expected = "┤" },
	{ .token = "`07<E>", .expected = "┌" },
	{ .token = "`08<E>", .expected = "┐" },
	{ .token = "`09<E>", .expected = "╞" },
	{ .token = "`00<E>", .expected = "═" },
	{ .token = "`0<R>1<E>", .expected = "╪" },
	{ .token = "`0<R>2<E>", .expected = "╡" },
	{ .token = "`0<R>3<E>", .expected = "│" },
	{ .token = "`0<R>4<E>", .expected = "▕" },
	{ .token = "`0<R>5<E>", .expected = "└" },
	{ .token = "`0<R>6<E>", .expected = "┘" },
	{ .token = "`0<R>7<E>", .expected = "╭" },
	{ .token = "`0<R>8<E>", .expected = "╮" },
	{ .token = "`0<R>9<E>", .expected = "╰" },
	{ .token = "`0<R>0<E>", .expected = "╯" },
	{ .token = "`<R>11<E>", .expected = "╔" },
	{ .token = "`<R>12<E>", .expected = "╦" },
	{ .token = "`<R>13<E>", .expected = "╗" },
	{ .token = "`<R>14<E>", .expected = "╠" },
	{ .token = "`<R>15<E>", .expected = "═" },
	{ .token = "`<R>16<E>", .expected = "╬" },
	{ .token = "`<R>17<E>", .expected = "╣" },
	{ .token = "`<R>18<E>", .expected = "╓" },
	{ .token = "`<R>19<E>", .expected = "╥" },
	{ .token = "`<R>10<E>", .expected = "╖" },
	{ .token = "`<R>1<R>1<E>", .expected = "╒" },
	{ .token = "`<R>1<R>2<E>", .expected = "╤" },
	{ .token = "`<R>1<R>3<E>", .expected = "╕" },
	{ .token = "`<R>1<R>4<E>", .expected = "║" },
	{ .token = "`<R>1<R>5<E>", .expected = "╚" },
	{ .token = "`<R>1<R>6<E>", .expected = "╩" },
	{ .token = "`<R>1<R>7<E>", .expected = "╝" },
	{ .token = "`<R>1<R>8<E>", .expected = "╟" },
	{ .token = "`<R>1<R>9<E>", .expected = "╫" },
	{ .token = "`<R>1<R>0<E>", .expected = "╢" },
	{ .token = "`<R>1<R><R>1<E>", .expected = "╙" },
	{ .token = "`<R>1<R><R>2<E>", .expected = "╨" },
	{ .token = "`<R>1<R><R>3<E>", .expected = "╜" },
	{ .token = "`<R>1<R><R>4<E>", .expected = "╞" },
	{ .token = "`<R>1<R><R>5<E>", .expected = "╪" },
	{ .token = "`<R>1<R><R>6<E>", .expected = "╡" },
	{ .token = "`<R>1<R><R>7<E>", .expected = "╘" },
	{ .token = "`<R>1<R><R>8<E>", .expected = "╧" },
	{ .token = "`<R>1<R><R>9<E>", .expected = "╛" },
	{ .token = "`<R>21<E>", .expected = "＿" },
	{ .token = "`<R>22<E>", .expected = "ˍ" },
	{ .token = "`<R>23<E>", .expected = "▁" },
	{ .token = "`<R>24<E>", .expected = "▂" },
	{ .token = "`<R>25<E>", .expected = "▃" },
	{ .token = "`<R>26<E>", .expected = "▄" },
	{ .token = "`<R>27<E>", .expected = "▅" },
	{ .token = "`<R>28<E>", .expected = "▆" },
	{ .token = "`<R>29<E>", .expected = "▇" },
	{ .token = "`<R>20<E>", .expected = "█" },
	{ .token = "`<R>2<R>1<E>", .expected = "▏" },
	{ .token = "`<R>2<R>2<E>", .expected = "▎" },
	{ .token = "`<R>2<R>3<E>", .expected = "▍" },
	{ .token = "`<R>2<R>4<E>", .expected = "▌" },
	{ .token = "`<R>2<R>5<E>", .expected = "▋" },
	{ .token = "`<R>2<R>6<E>", .expected = "▊" },
	{ .token = "`<R>2<R>7<E>", .expected = "▉" },
	{ .token = "`<R>2<R>8<E>", .expected = "◢" },
	{ .token = "`<R>2<R>9<E>", .expected = "◣" },
	{ .token = "`<R>2<R>0<E>", .expected = "◥" },
	{ .token = "`<R>2<R><R>1<E>", .expected = "◤" },
	{ .token = "`<R>31<E>", .expected = "﹣" },
	{ .token = "`<R>32<E>", .expected = "﹦" },
	{ .token = "`<R>33<E>", .expected = "≡" },
	{ .token = "`<R>34<E>", .expected = "｜" },
	{ .token = "`<R>35<E>", .expected = "∣" },
	{ .token = "`<R>36<E>", .expected = "∥" },
	{ .token = "`<R>37<E>", .expected = "–" },
	{ .token = "`<R>38<E>", .expected = "︱" },
	{ .token = "`<R>39<E>", .expected = "—" },
	{ .token = "`<R>30<E>", .expected = "︳" },
	{ .token = "`<R>3<R>1<E>", .expected = "╴" },
	{ .token = "`<R>3<R>2<E>", .expected = "¯" },
	{ .token = "`<R>3<R>3<E>", .expected = "￣" },
	{ .token = "`<R>3<R>4<E>", .expected = "﹉" },
	{ .token = "`<R>3<R>5<E>", .expected = "﹊" },
	{ .token = "`<R>3<R>6<E>", .expected = "﹍" },
	{ .token = "`<R>3<R>7<E>", .expected = "﹎" },
	{ .token = "`<R>3<R>8<E>", .expected = "﹋" },
	{ .token = "`<R>3<R>9<E>", .expected = "﹌" },
	{ .token = "`<R>3<R>0<E>", .expected = "﹏" },
	{ .token = "`<R>3<R><R>1<E>", .expected = "︴" },
	{ .token = "`<R>3<R><R>2<E>", .expected = "∕" },
	{ .token = "`<R>3<R><R>3<E>", .expected = "﹨" },
	{ .token = "`<R>3<R><R>4<E>", .expected = "╱" },
	{ .token = "`<R>3<R><R>5<E>", .expected = "╲" },
	{ .token = "`<R>3<R><R>6<E>", .expected = "／" },
	{ .token = "`<R>3<R><R>7<E>", .expected = "＼" },
};

static const char *CAND[] = {
	"…",
	"※",
	"常用符號",
	"左右括號",
	"上下括號",
	"希臘字母",
	"數學符號",
	"特殊圖形",
	"Unicode",
	"單線框",
	"雙線框",
	"填色方塊",
	"線段",
};

void test_type_symbol()
{
	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_candPerPage( ctx, 10 );
	chewing_set_maxChiSymbolLen( ctx, 16 );

	for (int i = 0; i < ARRAY_SIZE(SYMBOL); ++i ) {
		verify_keystoke( ctx, SYMBOL[i].token, SYMBOL[i].expected );
	}

	chewing_delete( ctx );
	chewing_Terminate();
}

void test_symbol_cand_page()
{
	char *buf;

	chewing_Init( NULL, NULL );

	ChewingContext *ctx = chewing_new();
	ok( ctx, "chewing_new shall not return NULL" );

	chewing_set_candPerPage( ctx, 10 );
	chewing_set_maxChiSymbolLen( ctx, 16 );

	chewing_handle_Default( ctx, '`' );
	ok( chewing_cand_CurrentPage( ctx ) == 0, "current page shall be 0" );
	ok( chewing_cand_TotalPage( ctx ) == 2, "total page shall be 2" );

	chewing_cand_Enumerate( ctx );
	for (int i = 0; i < ARRAY_SIZE( CAND ); ++i ) {
		ok( chewing_cand_hasNext( ctx ), "shall has next candidate" );
		buf = chewing_cand_String( ctx );
		ok( strcmp( buf, CAND[i] ) == 0,
			"candidate string shall match expected value");
		chewing_free( buf );
	}

	buf = chewing_cand_String( ctx );
	ok( strcmp( buf, "" ) == 0,
		"cand string shall be empty when out of range");
	chewing_free( buf );

	ok( !chewing_cand_hasNext( ctx ), "shall not have next candidate" );

	chewing_delete( ctx );
	chewing_Terminate();
}

int main ()
{
	putenv( "CHEWING_PATH=" CHEWING_DATA_PREFIX );
	putenv( "CHEWING_USER_PATH=" TEST_HASH_DIR );

	test_type_symbol();
	test_symbol_cand_page();

	return exit_status();
}
