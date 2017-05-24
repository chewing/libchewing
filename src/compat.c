/**
 * compat.c
 *
 * Copyright (c) 2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#include <string.h>

#include "chewing-utf8-util.h"

/* for compatibility */

#include "chewing.h"

CHEWING_API int chewing_zuin_Check(const ChewingContext *ctx)
{
    return !chewing_bopomofo_Check(ctx);
}

CHEWING_API char *chewing_zuin_String(const ChewingContext *ctx, int *bopomofo_count)
{
    char *s = strdup(chewing_bopomofo_String_static(ctx));

    if (bopomofo_count) {
        *bopomofo_count = ueStrLen(s);
    }

    return s;
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
    chewing_set_autoLearn(ctx, pcd->bAutoLearn);

    return 0;
}

CHEWING_API void chewing_set_hsuSelKeyType(ChewingContext *ctx UNUSED, int mode UNUSED)
{
}

CHEWING_API int chewing_get_hsuSelKeyType(ChewingContext *ctx UNUSED)
{
    return 0;
}
