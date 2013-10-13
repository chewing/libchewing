/**
 * chewing-sql.c
 *
 * Copyright (c)
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2013
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#ifndef CHEWING_SQL_H
#define CHEWING_SQL_H

#include "chewing-private.h"

int InitSql( struct tag_ChewingData *pgdata );
void TerminateHash( struct tag_ChewingData *pgdata );

#endif
