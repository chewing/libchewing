/**
 * test-utf8.c
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
#include "chewing-utf8-util.h"

int main (int argc, char *argv[])
{
	char *u8string;
	int u8len;

	u8string = "HelloWorld";
	u8len = ueStrLen(u8string);
	int len = strlen(u8string);
	ok (u8len == len, "ueStrLen");

	u8string = "測試計算長度";
	u8len = ueStrLen(u8string);
	ok (u8len == 6, "ueStrLen");

	u8string = "測試計算長度";
	char u8string2[16];
	ueStrNCpy(u8string2, u8string, 3, STRNCPY_CLOSE);
	ok (!strcmp(u8string2, "測試計"), "ueStrNCpy");

	u8string = "測試計算長度";
	u8string = ueStrSeek(u8string, 3);
	ok (!strcmp(u8string, "算長度"), "ueStrSeek");

	u8string = "測試計算長度";
	u8string = ueStrSeek(u8string, 0);
	ok (!strcmp(u8string, "測試計算長度"), "ueStrSeek");

	return exit_status();
}
