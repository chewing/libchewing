/**
 * test-key2pho.c
 *
 * Copyright (c) 2005
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include <stdio.h>
#include <string.h>

#include "test.h"
#include "global.h"
#include "chewing-utf8-util.h"
#include "key2pho-private.h"

int main (int argc, char *argv[])
{
	char *u8phone;

	u8phone = "ㄆㄣ";
	ok (UintFromPhone(u8phone) == 1104, "UintFromPhone");

	u8phone = "ㄊㄧㄢ";
	ok (UintFromPhone(u8phone) == 3272, "UintFromPhone");

	u8phone = "ㄒㄧㄚˋ";
	ok (UintFromPhone(u8phone) == 7308, "UintFromPhone");

	char rt[10];

	PhoneFromKey( rt, "dj", 0, 1 );
	ok (!strcmp(rt, "ㄎㄨ"), "dj");

	PhoneFromKey( rt, "dj6", 0, 1 );
	ok (!strcmp(rt, "ㄎㄨˊ"), "dj6");

	PhoneFromKey( rt, "dj3", 0, 1 );
	ok (!strcmp(rt, "ㄎㄨˇ"), "dj3");

	PhoneFromKey( rt, "dj4", 0, 1 );
	ok (!strcmp(rt, "ㄎㄨˋ"), "dj4");

	PhoneFromKey( rt, "dj7", 0, 1 );
	ok (!strcmp(rt, "ㄎㄨ˙"), "dj7");

	return exit_status();
}
