/* for compatibility */

#include "chewing.h"

CHEWING_API int chewing_zuin_Check(ChewingContext *ctx)
{
    return !chewing_bopomofo_Check(ctx);
}

CHEWING_API char *chewing_zuin_String(ChewingContext *ctx, int *zuin_count)
{
    return chewing_bopomofo_String(ctx, zuin_count);
}

CHEWING_API int chewing_Init(const char *dataPath UNUSED, const char *hashPath UNUSED)
{
    return 0;
}

CHEWING_API void chewing_Terminate()
{
}

CHEWING_API int chewing_Configure(ChewingContext *ctx, ChewingConfigData * pcd)
{
    chewing_set_candPerPage(ctx, pcd->candPerPage);
    chewing_set_maxChiSymbolLen(ctx, pcd->maxChiSymbolLen);
    chewing_set_selKey(ctx, pcd->selKey, MAX_SELKEY);
    chewing_set_addPhraseDirection(ctx, pcd->bAddPhraseForward);
    chewing_set_spaceAsSelection(ctx, pcd->bSpaceAsSelection);
    chewing_set_escCleanAllBuf(ctx, pcd->bEscCleanAllBuf);
    chewing_set_autoShiftCur(ctx, pcd->bAutoShiftCur);
    chewing_set_easySymbolInput(ctx, pcd->bEasySymbolInput);
    chewing_set_phraseChoiceRearward(ctx, pcd->bPhraseChoiceRearward);

    return 0;
}
