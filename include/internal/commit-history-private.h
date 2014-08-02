/**
 * commit-history-private.h
 *
 * Copyright (c) 2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* *INDENT-OFF* */
#ifndef _CHEWING_COMMIT_HISTORY_PRIVATE_H
#define _CHEWING_COMMIT_HISTORY_PRIVATE_H
/* *INDENT-ON* */

#ifdef HAVE_CONFIG_H
#    include <config.h>
#endif

#ifdef HAVE_INTTYPES_H
#    include <inttypes.h>
#elif defined HAVE_STDINT_H
#    include <stdint.h>
#endif

#define COMMIT_INSERT_SUCCESS (1)
#define COMMIT_INSERT_FAIL    (0)

/* Forward declaration */
struct ChewingData;

int CommitHistoryInsert(struct ChewingData *pgdata, const uint16_t phoneSeq[], const char wordSeq[]);

/* *INDENT-OFF* */
#endif
/* *INDENT-ON* */
