/**
 * choice-private.h
 *
 * Copyright (c) 2008, 2010
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* *INDENT-OFF* */
#ifndef _CHEWING_CHOICE_PRIVATE_H
#define _CHEWING_CHOICE_PRIVATE_H
/* *INDENT-ON* */

int ChoiceInitAvail(ChewingData *);
int ChoiceFirstAvail(ChewingData *pgdata);
int ChoiceLastAvail(ChewingData *pgdata);
int ChoiceHasNextAvail(ChewingData *pgdata);
int ChoiceHasPrevAvail(ChewingData *pgdata);
int ChoiceNextAvail(ChewingData *pgdata);
int ChoicePrevAvail(ChewingData *pgdata);
int ChoiceSelect(ChewingData *, int selectNo);
int ChoiceEndChoice(ChewingData *);

/* *INDENT-OFF* */
#endif
/* *INDENT-ON* */
