/*
 * chewing-compat.h
 *
 * Copyright (c) 2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_COMPAT_
#define _CHEWING_COMPAT_

/** @brief indicate the internal encoding of data processing.
 *  @since 0.3.0
 */
#define LIBCHEWING_ENCODING "UTF-8"

/* deprecated function. for API compatibility */
CHEWING_API int chewing_zuin_Check(ChewingContext *ctx) DEPRECATED;
CHEWING_API char *chewing_zuin_String(ChewingContext *,
                                      int *zuin_count) DEPRECATED;

CHEWING_API int chewing_Init(const char *dataPath, const char *hashPath) DEPRECATED;
CHEWING_API void chewing_Terminate() DEPRECATED;
CHEWING_API int chewing_Configure(ChewingContext *ctx, ChewingConfigData * pcd) DEPRECATED;

#endif
