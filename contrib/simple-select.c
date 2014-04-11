/*
 * original contributor: StarForcefield
 * http://starforcefield.wordpress.com/2012/08/13/
 */

#include <chewing.h>
#include <stdio.h>
#include <stdlib.h>

static int selKeys[] = { '1', '2', '3', '4', '5', '6', '7', '8', '9', 0 };

int main()
{
    ChewingContext *ctx;
    char *buf;
    int counter;

    /*
     * Initialize Chewing input method engine
     */
    ctx = chewing_new2(TEST_DATA_DIR, TEST_HASH_DIR "/test.sqlite3",
                       NULL, 0);

    /* Chinese mode by default */
    if (chewing_get_ChiEngMode(ctx) == CHINESE_MODE)
        printf("Chinese mode!\n");

    /* Set the selection keys, otherwise you can not select candidates. */
    chewing_set_selKey(ctx, selKeys, 9);
    /* Set the legth of maximum Chinese symbol */
    chewing_set_maxChiSymbolLen(ctx, 10);
    /* Set the amount of candidates per page */
    chewing_set_candPerPage(ctx, 9);

    /*
     * Sequence 1：'綠茶'
     */
    chewing_handle_Default(ctx, 'x');    /* ㄌ */
    chewing_handle_Default(ctx, 'm');    /* ㄩ */
    chewing_handle_Default(ctx, '4');    /* ˋ */
    chewing_handle_Default(ctx, 't');    /* ㄔ */
    chewing_handle_Default(ctx, '8');    /* ㄚ */
    chewing_handle_Default(ctx, '6');    /* ˊ */
    /* commit buffer to output area */
    chewing_handle_Enter(ctx);

    /* Copy stribf from output area */
    buf = chewing_commit_String(ctx);
    printf("%s\n", buf);
    free(buf);

    /*
     * Sequence 2：Input 'ㄓ' and select candidates
     */
    chewing_handle_Default(ctx, '5');
    chewing_handle_Space(ctx);
    /*
     * The expected key to enter candidate selection is 'Down'.
     * If 'Down' is not triggered, we can not use the further
     * chewing_cand_Enumerate() to get the detailed listing.
     */
    chewing_handle_Down(ctx);

    /*
     * Start the enumeration of candidate.  It follows the typical iterator
     * design.
     * (1) chewing_cand_Enumerate(): specify the iterator by ChewingContext
     * (2) chewing_cand_hasNext(): find out the next element in iterator
     * (3) chewing_cand_String(): get the current element and advance
     */
    chewing_cand_Enumerate(ctx);
    counter = 0;
    while (chewing_cand_hasNext(ctx)) {
        counter += 1;
        char *s = chewing_cand_String(ctx);

        printf("%s ", s);
        free(s);
        if (counter == 5) {
            counter = 0;
            printf("\n");
        }
    }

    printf("\nSelecting 13rd: ");
    /*
     * 剛才按下了↓，目前正在選字。
     * 我想選第13個字，那就必須換頁，然後選第二頁的第4個字（9+4=13）
     * 換頁的按鍵是空白鍵
     * (一頁有多少候選字的設定，在 chewing_set_candPerPage(ctx 9) 這個呼叫中)
     */
    chewing_handle_Space(ctx);
    chewing_handle_Default(ctx, '4');
    chewing_handle_Enter(ctx);

    buf = chewing_commit_String(ctx);
    printf("%s\n", buf);
    free(buf);

    /* Finalize Chewing input method engine */
    chewing_delete(ctx);
    return 0;
}
