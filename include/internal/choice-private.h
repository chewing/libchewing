/**
 * choice-private.h
 *
 * Copyright (c) 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_CHOICE_PRIVATE_H
#define _CHEWING_CHOICE_PRIVATE_H

int ChoiceFirstAvail( ChewingData * );
int ChoiceNextAvail( ChewingData * );
int ChoicePrevAvail( ChewingData * );
int ChoiceSelect( ChewingData *, int selectNo );
int ChoiceEndChoice( ChewingData * );

#endif
