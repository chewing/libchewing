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

int main ()
{
	char *u8phone;
	char rt[10];
	uint16_t phone;
	uint16_t expect;

	u8phone = "\xE3\x84\x86\xE3\x84\xA3" /* ㄆㄣ */;
	phone = UintFromPhone(u8phone);
	expect = 1104;
	ok (phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'",
		u8phone, phone, expect);

	u8phone = "\xE3\x84\x8A\xE3\x84\xA7\xE3\x84\xA2" /* ㄊㄧㄢ */;
	phone = UintFromPhone(u8phone);
	expect = 3272;
	ok (phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'",
		u8phone, phone, expect);

	u8phone = "\xE3\x84\x92\xE3\x84\xA7\xE3\x84\x9A\xCB\x8B" /* ㄒㄧㄚˋ */;
	phone = UintFromPhone(u8phone);
	expect = 7308;
	ok (phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'",
		u8phone, phone, expect);

	u8phone = "\xE3\x84\x8A\xE3\x84\xA7\xE6\xB8\xAC" /* ㄊㄧ測 */;
	phone = UintFromPhone(u8phone);
	expect = 0;
	ok (phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'",
		u8phone, phone, expect);

	u8phone = "\xE3\x84\x8E\xE3\x84\x8E" /* ㄎㄎ */;
	phone = UintFromPhone(u8phone);
	expect = 0;
	ok (phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'",
		u8phone, phone, expect);

	u8phone = "\xE3\x84\xA8\xE3\x84\x8E" /* ㄨㄎ */;
	phone = UintFromPhone(u8phone);
	expect = 0;
	ok (phone == expect, "UintFromPhone `%s' shall be `%d', got `%d'",
		u8phone, phone, expect);

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
