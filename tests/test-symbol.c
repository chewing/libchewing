/**
 * test-symbol.c
 *
 * Copyright (c) 2012
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#include <assert.h>
#include <stdlib.h>
#include <string.h>

#include "chewing.h"
#include "testhelper.h"

static const TestData SYMBOL[] = {
    {"`1<E>", "\xE2\x80\xA6" /* … */ },
    {"`2<E>", "\xE2\x80\xBB" /* ※ */ },
    {"`31<E>", "\xEF\xBC\x8C" /* ， */ },
    {"`32<E>", "\xE3\x80\x81" /* 、 */ },
    {"`33<E>", "\xE3\x80\x82" /* 。 */ },
    {"`34<E>", "\xEF\xBC\x8E" /* ． */ },
    {"`35<E>", "\xEF\xBC\x9F" /* ？ */ },
    {"`36<E>", "\xEF\xBC\x81" /* ！ */ },
    {"`37<E>", "\xEF\xBC\x9B" /* ； */ },
    {"`38<E>", "\xEF\xBC\x9A" /* ： */ },
    {"`39<E>", "\xE2\x80\xA7" /* ‧ */ },
    {"`30<E>", "\xE2\x80\xA5" /* ‥ */ },
    {"`3<R>1<E>", "\xEF\xB9\x90" /* ﹐ */ },
    {"`3<R>2<E>", "\xEF\xB9\x92" /* ﹒ */ },
    {"`3<R>3<E>", "\xCB\x99" /* ˙ */ },
    {"`3<R>4<E>", "\xC2\xB7" /* · */ },
    {"`3<R>5<E>", "\xE2\x80\x98" /* ‘ */ },
    {"`3<R>6<E>", "\xE2\x80\x99" /* ’ */ },
    {"`3<R>7<E>", "\xE2\x80\x9C" /* “ */ },
    {"`3<R>8<E>", "\xE2\x80\x9D" /* ” */ },
    {"`3<R>9<E>", "\xE3\x80\x9D" /* 〝 */ },
    {"`3<R>0<E>", "\xE3\x80\x9E" /* 〞 */ },
    {"`3<R><R>1<E>", "\xE2\x80\xB5" /* ‵ */ },
    {"`3<R><R>2<E>", "\xE2\x80\xB2" /* ′ */ },
    {"`3<R><R>3<E>", "\xE3\x80\x83" /* 〃 */ },
    {"`3<R><R>4<E>", "\xEF\xBD\x9E" /* ～ */ },
    {"`3<R><R>5<E>", "\xEF\xBC\x84" /* ＄ */ },
    {"`3<R><R>6<E>", "\xEF\xBC\x85" /* ％ */ },
    {"`3<R><R>7<E>", "\xEF\xBC\xA0" /* ＠ */ },
    {"`3<R><R>8<E>", "\xEF\xBC\x86" /* ＆ */ },
    {"`3<R><R>9<E>", "\xEF\xBC\x83" /* ＃ */ },
    {"`3<R><R>0<E>", "\xEF\xBC\x8A" /* ＊ */ },
    {"`41<E>", "\xEF\xBC\x88" /* （ */ },
    {"`42<E>", "\xEF\xBC\x89" /* ） */ },
    {"`43<E>", "\xE3\x80\x8C" /* 「 */ },
    {"`44<E>", "\xE3\x80\x8D" /* 」 */ },
    {"`45<E>", "\xE3\x80\x94" /* 〔 */ },
    {"`46<E>", "\xE3\x80\x95" /* 〕 */ },
    {"`47<E>", "\xEF\xBD\x9B" /* ｛ */ },
    {"`48<E>", "\xEF\xBD\x9D" /* ｝ */ },
    {"`49<E>", "\xE3\x80\x88" /* 〈 */ },
    {"`40<E>", "\xE3\x80\x89" /* 〉 */ },
    {"`4<R>1<E>", "\xE3\x80\x8E" /* 『 */ },
    {"`4<R>2<E>", "\xE3\x80\x8F" /* 』 */ },
    {"`4<R>3<E>", "\xE3\x80\x8A" /* 《 */ },
    {"`4<R>4<E>", "\xE3\x80\x8B" /* 》 */ },
    {"`4<R>5<E>", "\xE3\x80\x90" /* 【 */ },
    {"`4<R>6<E>", "\xE3\x80\x91" /* 】 */ },
    {"`4<R>7<E>", "\xEF\xB9\x99" /* ﹙ */ },
    {"`4<R>8<E>", "\xEF\xB9\x9A" /* ﹚ */ },
    {"`4<R>9<E>", "\xEF\xB9\x9D" /* ﹝ */ },
    {"`4<R>0<E>", "\xEF\xB9\x9E" /* ﹞ */ },
    {"`4<R><R>1<E>", "\xEF\xB9\x9B" /* ﹛ */ },
    {"`4<R><R>2<E>", "\xEF\xB9\x9C" /* ﹜ */ },
    {"`51<E>", "\xEF\xB8\xB5" /* ︵ */ },
    {"`52<E>", "\xEF\xB8\xB6" /* ︶ */ },
    {"`53<E>", "\xEF\xB9\x81" /* ﹁ */ },
    {"`54<E>", "\xEF\xB9\x82" /* ﹂ */ },
    {"`55<E>", "\xEF\xB8\xB9" /* ︹ */ },
    {"`56<E>", "\xEF\xB8\xBA" /* ︺ */ },
    {"`57<E>", "\xEF\xB8\xB7" /* ︷ */ },
    {"`58<E>", "\xEF\xB8\xB8" /* ︸ */ },
    {"`59<E>", "\xEF\xB8\xBF" /* ︿ */ },
    {"`50<E>", "\xEF\xB9\x80" /* ﹀ */ },
    {"`5<R>1<E>", "\xEF\xB9\x83" /* ﹃ */ },
    {"`5<R>2<E>", "\xEF\xB9\x84" /* ﹄ */ },
    {"`5<R>3<E>", "\xEF\xB8\xBD" /* ︽ */ },
    {"`5<R>4<E>", "\xEF\xB8\xBE" /* ︾ */ },
    {"`5<R>5<E>", "\xEF\xB8\xBB" /* ︻ */ },
    {"`5<R>6<E>", "\xEF\xB8\xBC" /* ︼ */ },
    {"`61<E>", "\xCE\xB1" /* α */ },
    {"`62<E>", "\xCE\xB2" /* β */ },
    {"`63<E>", "\xCE\xB3" /* γ */ },
    {"`64<E>", "\xCE\xB4" /* δ */ },
    {"`65<E>", "\xCE\xB5" /* ε */ },
    {"`66<E>", "\xCE\xB6" /* ζ */ },
    {"`67<E>", "\xCE\xB7" /* η */ },
    {"`68<E>", "\xCE\xB8" /* θ */ },
    {"`69<E>", "\xCE\xB9" /* ι */ },
    {"`60<E>", "\xCE\xBA" /* κ */ },
    {"`6<R>1<E>", "\xCE\xBB" /* λ */ },
    {"`6<R>2<E>", "\xCE\xBC" /* μ */ },
    {"`6<R>3<E>", "\xCE\xBD" /* ν */ },
    {"`6<R>4<E>", "\xCE\xBE" /* ξ */ },
    {"`6<R>5<E>", "\xCE\xBF" /* ο */ },
    {"`6<R>6<E>", "\xCF\x80" /* π */ },
    {"`6<R>7<E>", "\xCF\x81" /* ρ */ },
    {"`6<R>8<E>", "\xCF\x83" /* σ */ },
    {"`6<R>9<E>", "\xCF\x84" /* τ */ },
    {"`6<R>0<E>", "\xCF\x85" /* υ */ },
    {"`6<R><R>1<E>", "\xCF\x86" /* φ */ },
    {"`6<R><R>2<E>", "\xCF\x87" /* χ */ },
    {"`6<R><R>3<E>", "\xCF\x88" /* ψ */ },
    {"`6<R><R>4<E>", "\xCF\x89" /* ω */ },
    {"`6<R><R>5<E>", "\xCE\x91" /* Α */ },
    {"`6<R><R>6<E>", "\xCE\x92" /* Β */ },
    {"`6<R><R>7<E>", "\xCE\x93" /* Γ */ },
    {"`6<R><R>8<E>", "\xCE\x94" /* Δ */ },
    {"`6<R><R>9<E>", "\xCE\x95" /* Ε */ },
    {"`6<R><R>0<E>", "\xCE\x96" /* Ζ */ },
    {"`6<R><R><R>1<E>", "\xCE\x97" /* Η */ },
    {"`6<R><R><R>2<E>", "\xCE\x98" /* Θ */ },
    {"`6<R><R><R>3<E>", "\xCE\x99" /* Ι */ },
    {"`6<R><R><R>4<E>", "\xCE\x9A" /* Κ */ },
    {"`6<R><R><R>5<E>", "\xCE\x9B" /* Λ */ },
    {"`6<R><R><R>6<E>", "\xCE\x9C" /* Μ */ },
    {"`6<R><R><R>7<E>", "\xCE\x9D" /* Ν */ },
    {"`6<R><R><R>8<E>", "\xCE\x9E" /* Ξ */ },
    {"`6<R><R><R>9<E>", "\xCE\x9F" /* Ο */ },
    {"`6<R><R><R>0<E>", "\xCE\xA0" /* Π */ },
    {"`6<R><R><R><R>1<E>", "\xCE\xA1" /* Ρ */ },
    {"`6<R><R><R><R>2<E>", "\xCE\xA3" /* Σ */ },
    {"`6<R><R><R><R>3<E>", "\xCE\xA4" /* Τ */ },
    {"`6<R><R><R><R>4<E>", "\xCE\xA5" /* Υ */ },
    {"`6<R><R><R><R>5<E>", "\xCE\xA6" /* Φ */ },
    {"`6<R><R><R><R>6<E>", "\xCE\xA7" /* Χ */ },
    {"`6<R><R><R><R>7<E>", "\xCE\xA8" /* Ψ */ },
    {"`6<R><R><R><R>8<E>", "\xCE\xA9" /* Ω */ },
    {"`71<E>", "\xEF\xBC\x8B" /* ＋ */ },
    {"`72<E>", "\xEF\xBC\x8D" /* － */ },
    {"`73<E>", "\xC3\x97" /* × */ },
    {"`74<E>", "\xC3\xB7" /* ÷ */ },
    {"`75<E>", "\xEF\xBC\x9D" /* ＝ */ },
    {"`76<E>", "\xE2\x89\xA0" /* ≠ */ },
    {"`77<E>", "\xE2\x89\x92" /* ≒ */ },
    {"`78<E>", "\xE2\x88\x9E" /* ∞ */ },
    {"`79<E>", "\xC2\xB1" /* ± */ },
    {"`70<E>", "\xE2\x88\x9A" /* √ */ },
    {"`7<R>1<E>", "\xEF\xBC\x9C" /* ＜ */ },
    {"`7<R>2<E>", "\xEF\xBC\x9E" /* ＞ */ },
    {"`7<R>3<E>", "\xEF\xB9\xA4" /* ﹤ */ },
    {"`7<R>4<E>", "\xEF\xB9\xA5" /* ﹥ */ },
    {"`7<R>5<E>", "\xE2\x89\xA6" /* ≦ */ },
    {"`7<R>6<E>", "\xE2\x89\xA7" /* ≧ */ },
    {"`7<R>7<E>", "\xE2\x88\xA9" /* ∩ */ },
    {"`7<R>8<E>", "\xE2\x88\xAA" /* ∪ */ },
    {"`7<R>9<E>", "\xCB\x87" /* ˇ */ },
    {"`7<R>0<E>", "\xE2\x8A\xA5" /* ⊥ */ },
    {"`7<R><R>1<E>", "\xE2\x88\xA0" /* ∠ */ },
    {"`7<R><R>2<E>", "\xE2\x88\x9F" /* ∟ */ },
    {"`7<R><R>3<E>", "\xE2\x8A\xBF" /* ⊿ */ },
    {"`7<R><R>4<E>", "\xE3\x8F\x92" /* ㏒ */ },
    {"`7<R><R>5<E>", "\xE3\x8F\x91" /* ㏑ */ },
    {"`7<R><R>6<E>", "\xE2\x88\xAB" /* ∫ */ },
    {"`7<R><R>7<E>", "\xE2\x88\xAE" /* ∮ */ },
    {"`7<R><R>8<E>", "\xE2\x88\xB5" /* ∵ */ },
    {"`7<R><R>9<E>", "\xE2\x88\xB4" /* ∴ */ },
    {"`7<R><R>0<E>", "\xE2\x95\xB3" /* ╳ */ },
    {"`7<R><R><R>1<E>", "\xEF\xB9\xA2" /* ﹢ */ },
    {"`81<E>", "\xE2\x86\x91" /* ↑ */ },
    {"`82<E>", "\xE2\x86\x93" /* ↓ */ },
    {"`83<E>", "\xE2\x86\x90" /* ← */ },
    {"`84<E>", "\xE2\x86\x92" /* → */ },
    {"`85<E>", "\xE2\x86\x96" /* ↖ */ },
    {"`86<E>", "\xE2\x86\x97" /* ↗ */ },
    {"`87<E>", "\xE2\x86\x99" /* ↙ */ },
    {"`88<E>", "\xE2\x86\x98" /* ↘ */ },
    {"`89<E>", "\xE3\x8A\xA3" /* ㊣ */ },
    {"`80<E>", "\xE2\x97\x8E" /* ◎ */ },
    {"`8<R>1<E>", "\xE2\x97\x8B" /* ○ */ },
    {"`8<R>2<E>", "\xE2\x97\x8F" /* ● */ },
    {"`8<R>3<E>", "\xE2\x8A\x95" /* ⊕ */ },
    {"`8<R>4<E>", "\xE2\x8A\x99" /* ⊙ */ },
    {"`8<R>5<E>", "\xE2\x97\x8B" /* ○ */ },
    {"`8<R>6<E>", "\xE2\x97\x8F" /* ● */ },
    {"`8<R>7<E>", "\xE2\x96\xB3" /* △ */ },
    {"`8<R>8<E>", "\xE2\x96\xB2" /* ▲ */ },
    {"`8<R>9<E>", "\xE2\x98\x86" /* ☆ */ },
    {"`8<R>0<E>", "\xE2\x98\x85" /* ★ */ },
    {"`8<R><R>1<E>", "\xE2\x97\x87" /* ◇ */ },
    {"`8<R><R>2<E>", "\xE2\x97\x86" /* ◆ */ },
    {"`8<R><R>3<E>", "\xE2\x96\xA1" /* □ */ },
    {"`8<R><R>4<E>", "\xE2\x96\xA0" /* ■ */ },
    {"`8<R><R>5<E>", "\xE2\x96\xBD" /* ▽ */ },
    {"`8<R><R>6<E>", "\xE2\x96\xBC" /* ▼ */ },
    {"`8<R><R>7<E>", "\xC2\xA7" /* § */ },
    {"`8<R><R>8<E>", "\xEF\xBF\xA5" /* ￥ */ },
    {"`8<R><R>9<E>", "\xE3\x80\x92" /* 〒 */ },
    {"`8<R><R>0<E>", "\xEF\xBF\xA0" /* ￠ */ },
    {"`8<R><R><R>1<E>", "\xEF\xBF\xA1" /* ￡ */ },
    {"`8<R><R><R>2<E>", "\xE2\x80\xBB" /* ※ */ },
    {"`8<R><R><R>3<E>", "\xE2\x99\x80" /* ♀ */ },
    {"`8<R><R><R>4<E>", "\xE2\x99\x82" /* ♂ */ },
    {"`91<E>", "\xE2\x99\xA8" /* ♨ */ },
    {"`92<E>", "\xE2\x98\x80" /* ☀ */ },
    {"`93<E>", "\xE2\x98\x81" /* ☁ */ },
    {"`94<E>", "\xE2\x98\x82" /* ☂ */ },
    {"`95<E>", "\xE2\x98\x83" /* ☃ */ },
    {"`96<E>", "\xE2\x99\xA0" /* ♠ */ },
    {"`97<E>", "\xE2\x99\xA5" /* ♥ */ },
    {"`98<E>", "\xE2\x99\xA3" /* ♣ */ },
    {"`99<E>", "\xE2\x99\xA6" /* ♦ */ },
    {"`90<E>", "\xE2\x99\xA9" /* ♩ */ },
    {"`9<R>1<E>", "\xE2\x99\xAA" /* ♪ */ },
    {"`9<R>2<E>", "\xE2\x99\xAB" /* ♫ */ },
    {"`9<R>3<E>", "\xE2\x99\xAC" /* ♬ */ },
    {"`9<R>4<E>", "\xE2\x98\xBA" /* ☺ */ },
    {"`9<R>5<E>", "\xE2\x98\xBB" /* ☻ */ },
    {"`01<E>", "\xE2\x94\x9C" /* ├ */ },
    {"`02<E>", "\xE2\x94\x80" /* ─ */ },
    {"`03<E>", "\xE2\x94\xBC" /* ┼ */ },
    {"`04<E>", "\xE2\x94\xB4" /* ┴ */ },
    {"`05<E>", "\xE2\x94\xAC" /* ┬ */ },
    {"`06<E>", "\xE2\x94\xA4" /* ┤ */ },
    {"`07<E>", "\xE2\x94\x8C" /* ┌ */ },
    {"`08<E>", "\xE2\x94\x90" /* ┐ */ },
    {"`09<E>", "\xE2\x95\x9E" /* ╞ */ },
    {"`00<E>", "\xE2\x95\x90" /* ═ */ },
    {"`0<R>1<E>", "\xE2\x95\xAA" /* ╪ */ },
    {"`0<R>2<E>", "\xE2\x95\xA1" /* ╡ */ },
    {"`0<R>3<E>", "\xE2\x94\x82" /* │ */ },
    {"`0<R>4<E>", "\xE2\x96\x95" /* ▕ */ },
    {"`0<R>5<E>", "\xE2\x94\x94" /* └ */ },
    {"`0<R>6<E>", "\xE2\x94\x98" /* ┘ */ },
    {"`0<R>7<E>", "\xE2\x95\xAD" /* ╭ */ },
    {"`0<R>8<E>", "\xE2\x95\xAE" /* ╮ */ },
    {"`0<R>9<E>", "\xE2\x95\xB0" /* ╰ */ },
    {"`0<R>0<E>", "\xE2\x95\xAF" /* ╯ */ },
    {"`<R>11<E>", "\xE2\x95\x94" /* ╔ */ },
    {"`<R>12<E>", "\xE2\x95\xA6" /* ╦ */ },
    {"`<R>13<E>", "\xE2\x95\x97" /* ╗ */ },
    {"`<R>14<E>", "\xE2\x95\xA0" /* ╠ */ },
    {"`<R>15<E>", "\xE2\x95\x90" /* ═ */ },
    {"`<R>16<E>", "\xE2\x95\xAC" /* ╬ */ },
    {"`<R>17<E>", "\xE2\x95\xA3" /* ╣ */ },
    {"`<R>18<E>", "\xE2\x95\x93" /* ╓ */ },
    {"`<R>19<E>", "\xE2\x95\xA5" /* ╥ */ },
    {"`<R>10<E>", "\xE2\x95\x96" /* ╖ */ },
    {"`<R>1<R>1<E>", "\xE2\x95\x92" /* ╒ */ },
    {"`<R>1<R>2<E>", "\xE2\x95\xA4" /* ╤ */ },
    {"`<R>1<R>3<E>", "\xE2\x95\x95" /* ╕ */ },
    {"`<R>1<R>4<E>", "\xE2\x95\x91" /* ║ */ },
    {"`<R>1<R>5<E>", "\xE2\x95\x9A" /* ╚ */ },
    {"`<R>1<R>6<E>", "\xE2\x95\xA9" /* ╩ */ },
    {"`<R>1<R>7<E>", "\xE2\x95\x9D" /* ╝ */ },
    {"`<R>1<R>8<E>", "\xE2\x95\x9F" /* ╟ */ },
    {"`<R>1<R>9<E>", "\xE2\x95\xAB" /* ╫ */ },
    {"`<R>1<R>0<E>", "\xE2\x95\xA2" /* ╢ */ },
    {"`<R>1<R><R>1<E>", "\xE2\x95\x99" /* ╙ */ },
    {"`<R>1<R><R>2<E>", "\xE2\x95\xA8" /* ╨ */ },
    {"`<R>1<R><R>3<E>", "\xE2\x95\x9C" /* ╜ */ },
    {"`<R>1<R><R>4<E>", "\xE2\x95\x9E" /* ╞ */ },
    {"`<R>1<R><R>5<E>", "\xE2\x95\xAA" /* ╪ */ },
    {"`<R>1<R><R>6<E>", "\xE2\x95\xA1" /* ╡ */ },
    {"`<R>1<R><R>7<E>", "\xE2\x95\x98" /* ╘ */ },
    {"`<R>1<R><R>8<E>", "\xE2\x95\xA7" /* ╧ */ },
    {"`<R>1<R><R>9<E>", "\xE2\x95\x9B" /* ╛ */ },
    {"`<R>21<E>", "\xEF\xBC\xBF" /* ＿ */ },
    {"`<R>22<E>", "\xCB\x8D" /* ˍ */ },
    {"`<R>23<E>", "\xE2\x96\x81" /* ▁ */ },
    {"`<R>24<E>", "\xE2\x96\x82" /* ▂ */ },
    {"`<R>25<E>", "\xE2\x96\x83" /* ▃ */ },
    {"`<R>26<E>", "\xE2\x96\x84" /* ▄ */ },
    {"`<R>27<E>", "\xE2\x96\x85" /* ▅ */ },
    {"`<R>28<E>", "\xE2\x96\x86" /* ▆ */ },
    {"`<R>29<E>", "\xE2\x96\x87" /* ▇ */ },
    {"`<R>20<E>", "\xE2\x96\x88" /* █ */ },
    {"`<R>2<R>1<E>", "\xE2\x96\x8F" /* ▏ */ },
    {"`<R>2<R>2<E>", "\xE2\x96\x8E" /* ▎ */ },
    {"`<R>2<R>3<E>", "\xE2\x96\x8D" /* ▍ */ },
    {"`<R>2<R>4<E>", "\xE2\x96\x8C" /* ▌ */ },
    {"`<R>2<R>5<E>", "\xE2\x96\x8B" /* ▋ */ },
    {"`<R>2<R>6<E>", "\xE2\x96\x8A" /* ▊ */ },
    {"`<R>2<R>7<E>", "\xE2\x96\x89" /* ▉ */ },
    {"`<R>2<R>8<E>", "\xE2\x97\xA2" /* ◢ */ },
    {"`<R>2<R>9<E>", "\xE2\x97\xA3" /* ◣ */ },
    {"`<R>2<R>0<E>", "\xE2\x97\xA5" /* ◥ */ },
    {"`<R>2<R><R>1<E>", "\xE2\x97\xA4" /* ◤ */ },
    {"`<R>31<E>", "\xEF\xB9\xA3" /* ﹣ */ },
    {"`<R>32<E>", "\xEF\xB9\xA6" /* ﹦ */ },
    {"`<R>33<E>", "\xE2\x89\xA1" /* ≡ */ },
    {"`<R>34<E>", "\xEF\xBD\x9C" /* ｜ */ },
    {"`<R>35<E>", "\xE2\x88\xA3" /* ∣ */ },
    {"`<R>36<E>", "\xE2\x88\xA5" /* ∥ */ },
    {"`<R>37<E>", "\xE2\x80\x93" /* – */ },
    {"`<R>38<E>", "\xEF\xB8\xB1" /* ︱ */ },
    {"`<R>39<E>", "\xE2\x80\x94" /* — */ },
    {"`<R>30<E>", "\xEF\xB8\xB3" /* ︳ */ },
    {"`<R>3<R>1<E>", "\xE2\x95\xB4" /* ╴ */ },
    {"`<R>3<R>2<E>", "\xC2\xAF" /* ¯ */ },
    {"`<R>3<R>3<E>", "\xEF\xBF\xA3" /* ￣ */ },
    {"`<R>3<R>4<E>", "\xEF\xB9\x89" /* ﹉ */ },
    {"`<R>3<R>5<E>", "\xEF\xB9\x8A" /* ﹊ */ },
    {"`<R>3<R>6<E>", "\xEF\xB9\x8D" /* ﹍ */ },
    {"`<R>3<R>7<E>", "\xEF\xB9\x8E" /* ﹎ */ },
    {"`<R>3<R>8<E>", "\xEF\xB9\x8B" /* ﹋ */ },
    {"`<R>3<R>9<E>", "\xEF\xB9\x8C" /* ﹌ */ },
    {"`<R>3<R>0<E>", "\xEF\xB9\x8F" /* ﹏ */ },
    {"`<R>3<R><R>1<E>", "\xEF\xB8\xB4" /* ︴ */ },
    {"`<R>3<R><R>2<E>", "\xE2\x88\x95" /* ∕ */ },
    {"`<R>3<R><R>3<E>", "\xEF\xB9\xA8" /* ﹨ */ },
    {"`<R>3<R><R>4<E>", "\xE2\x95\xB1" /* ╱ */ },
    {"`<R>3<R><R>5<E>", "\xE2\x95\xB2" /* ╲ */ },
    {"`<R>3<R><R>6<E>", "\xEF\xBC\x8F" /* ／ */ },
    {"`<R>3<R><R>7<E>", "\xEF\xBC\xBC" /* ＼ */ },
};

static const char *CAND[] = {
    "\xE2\x80\xA6" /* … */ ,
    "\xE2\x80\xBB" /* ※ */ ,
    "\xE5\xB8\xB8\xE7\x94\xA8\xE7\xAC\xA6\xE8\x99\x9F" /* 常用符號 */ ,
    "\xE5\xB7\xA6\xE5\x8F\xB3\xE6\x8B\xAC\xE8\x99\x9F" /* 左右括號 */ ,
    "\xE4\xB8\x8A\xE4\xB8\x8B\xE6\x8B\xAC\xE8\x99\x9F" /* 上下括號 */ ,
    "\xE5\xB8\x8C\xE8\x87\x98\xE5\xAD\x97\xE6\xAF\x8D" /* 希臘字母 */ ,
    "\xE6\x95\xB8\xE5\xAD\xB8\xE7\xAC\xA6\xE8\x99\x9F" /* 數學符號 */ ,
    "\xE7\x89\xB9\xE6\xAE\x8A\xE5\x9C\x96\xE5\xBD\xA2" /* 特殊圖形 */ ,
    "\x55\x6E\x69\x63\x6F\x64\x65" /* Unicode */ ,
    "\xE5\x96\xAE\xE7\xB7\x9A\xE6\xA1\x86" /* 單線框 */ ,
    "\xE9\x9B\x99\xE7\xB7\x9A\xE6\xA1\x86" /* 雙線框 */ ,
    "\xE5\xA1\xAB\xE8\x89\xB2\xE6\x96\xB9\xE5\xA1\x8A" /* 填色方塊 */ ,
    "\xE7\xB7\x9A\xE6\xAE\xB5" /* 線段 */ ,
};

FILE *fd;

void test_type_symbol()
{
    ChewingContext *ctx;
    size_t i;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_candPerPage(ctx, 10);
    chewing_set_maxChiSymbolLen(ctx, 16);

    for (i = 0; i < ARRAY_SIZE(SYMBOL); ++i) {
        type_keystroke_by_string(ctx, SYMBOL[i].token);
        ok_preedit_buffer(ctx, "");
        ok_commit_buffer(ctx, SYMBOL[i].expected);
    }

    chewing_delete(ctx);
}

void test_symbol_cand_page()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_candPerPage(ctx, 10);
    chewing_set_maxChiSymbolLen(ctx, 16);

    chewing_handle_Default(ctx, '`');
    ok(chewing_cand_CurrentPage(ctx) == 0, "current page shall be 0");
    ok(chewing_cand_TotalPage(ctx) == 2, "total page shall be 2");

    ok_candidate(ctx, CAND, ARRAY_SIZE(CAND));

    chewing_delete(ctx);
}

void test_symbol_count()
{
    ChewingContext *ctx;
    int total;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    type_keystroke_by_string(ctx, "`3");
    total = chewing_cand_TotalChoice(ctx);
    ok(total == 30, "total candidate for `3 is %d, shall be %d", total, 30);

    chewing_delete(ctx);
}

void test_symbol()
{
    test_symbol_cand_page();
    test_symbol_count();
}

void test_nocand_symbol()
{
    ChewingContext *ctx;

    ctx = chewing_new();
    start_testcase(ctx, fd);

    chewing_set_candPerPage(ctx, 10);
    chewing_set_maxChiSymbolLen(ctx, 16);

    type_keystroke_by_string(ctx, "`<R>20");
    ok_preedit_buffer(ctx, "\xE2\x96\x88"); /* █ */

    type_keystroke_by_string(ctx, "<D>");
    ok_candidate(ctx, CAND, ARRAY_SIZE(CAND));

    type_keystroke_by_string(ctx, "1<E>"); /* select … */
    ok_commit_buffer(ctx, "\xE2\x80\xA6");

    chewing_delete(ctx);
}

int main(int argc, char *argv[])
{
    char *logname;
    int ret;

    putenv("CHEWING_PATH=" CHEWING_DATA_PREFIX);
    putenv("CHEWING_USER_PATH=" TEST_HASH_DIR);

    ret = asprintf(&logname, "%s.log", argv[0]);
    if (ret == -1)
        return -1;
    fd = fopen(logname, "w");
    assert(fd);
    free(logname);

    test_type_symbol();
    test_symbol();
    test_nocand_symbol();

    fclose(fd);

    return exit_status();
}
