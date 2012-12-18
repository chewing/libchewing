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

#include "testhelper.h"
#include "global.h"
#include "chewing-utf8-util.h"
#include "key2pho-private.h"

int main (int argc, char *argv[])
{
	char *u8phone;
	char rt[10];

	u8phone = "\xE3\x84\x86\xE3\x84\xA3" /* ㄆㄣ */;
	ok (UintFromPhone(u8phone) == 1104, "UintFromPhone");

	u8phone = "\xE3\x84\x8A\xE3\x84\xA7\xE3\x84\xA2" /* ㄊㄧㄢ */;
	ok (UintFromPhone(u8phone) == 3272, "UintFromPhone");

	u8phone = "\xE3\x84\x92\xE3\x84\xA7\xE3\x84\x9A\xCB\x8B" /* ㄒㄧㄚˋ */;
	ok (UintFromPhone(u8phone) == 7308, "UintFromPhone");

	PhoneFromKey( rt, "dj", 0, 1 );
	ok (!strcmp(rt, "\xE3\x84\x8E\xE3\x84\xA8" /* ㄎㄨ */ ), "dj");

	PhoneFromKey( rt, "dj6", 0, 1 );
	ok (!strcmp(rt, "\xE3\x84\x8E\xE3\x84\xA8\xCB\x8A" /* ㄎㄨˊ */ ), "dj6");

	PhoneFromKey( rt, "dj3", 0, 1 );
	ok (!strcmp(rt, "\xE3\x84\x8E\xE3\x84\xA8\xCB\x87" /* ㄎㄨˇ */ ), "dj3");

	PhoneFromKey( rt, "dj4", 0, 1 );
	ok (!strcmp(rt, "\xE3\x84\x8E\xE3\x84\xA8\xCB\x8B" /* ㄎㄨˋ */ ), "dj4");

	PhoneFromKey( rt, "dj7", 0, 1 );
	ok (!strcmp(rt, "\xE3\x84\x8E\xE3\x84\xA8\xCB\x99" /* ㄎㄨ˙ */ ), "dj7");

	return exit_status();
}
