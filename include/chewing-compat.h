/*
 * chewing-compat.h
 *
 * Copyright (c) 2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* *INDENT-OFF* */
#ifndef _CHEWING_COMPAT_
#define _CHEWING_COMPAT_
/* *INDENT-ON* */

/** @brief indicate the internal encoding of data processing.
 *  @since 0.3.0
 */
#define LIBCHEWING_ENCODING "UTF-8"

/* deprecated function. for API compatibility */
CHEWING_API int chewing_zuin_Check(const ChewingContext *ctx)
    DEPRECATED_FOR(chewing_bopomofo_Check);
CHEWING_API char *chewing_zuin_String(const ChewingContext *, int *zuin_count)
    DEPRECATED_FOR(chewing_bopomofo_String_static);

CHEWING_API int chewing_Init(const char *dataPath, const char *hashPath)
    DEPRECATED;
CHEWING_API void chewing_Terminate() DEPRECATED;
CHEWING_API int chewing_Configure(ChewingContext *ctx, ChewingConfigData * pcd)
    DEPRECATED_FOR(chewing_set_*);
CHEWING_API void chewing_set_hsuSelKeyType(ChewingContext *ctx, int mode)
    DEPRECATED_FOR(chewing_set_selKey);
CHEWING_API int chewing_get_hsuSelKeyType(ChewingContext *ctx)
    DEPRECATED_FOR(chewing_get_selKey);

/* *INDENT-OFF* */
#endif
/* *INDENT-ON* */
