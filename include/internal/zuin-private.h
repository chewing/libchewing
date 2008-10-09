/**
 * zuin-private.h
 *
 * Copyright (c) 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_ZUIN_PRIVATE_H
#define _CHEWING_ZUIN_PRIVATE_H

#include "chewing-private.h"

int ZuinPhoInput( ZuinData *,int key );  /* assume `key' is "ascii" code. */
int ZuinRemoveLast( ZuinData * );
int ZuinRemoveAll( ZuinData * );
int ZuinIsEntering( ZuinData * );
         
#endif
