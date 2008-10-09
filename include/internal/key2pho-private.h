/**
 * key2pho-private.h
 *
 * Copyright (c) 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_KEY2PHO_PRIVATE_H
#define _CHEWING_KEY2PHO_PRIVATE_H

uint16 UintFromPhone( const char *phone );
uint16 UintFromPhoneInx( const int ph_inx[] );
int PhoneFromKey( char *pho, const char *inputkey, int kbtype, int searchTimes );
int PhoneInxFromKey( int key, int type, int kbtype, int searchTimes );

#endif
