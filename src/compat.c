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
