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
#include <check.h>
#include "chewing-utf8-util.h"

START_TEST(test_strlen1)
{
	char *u8string = "HelloWorld";
	int u8len = ueStrLen(u8string);
	int len = strlen(u8string);
	fail_unless(u8len == len, NULL);
}
END_TEST

START_TEST(test_strlen2)
{
	char *u8string = "測試計算長度";
	int u8len = ueStrLen(u8string);
	fail_unless(u8len == 6, NULL);
}
END_TEST

START_TEST(test_strncpy)
{
	char *u8string = "測試計算長度";
	char u8string2[16];
	ueStrNCpy(u8string2, u8string, 3, STRNCPY_CLOSE);
	fail_if(strcmp(u8string2, "測試計"), NULL);
}
END_TEST

START_TEST(test_strseek)
{
	char *u8string = "測試計算長度";
	u8string = ueStrSeek(u8string, 3);
	fail_if(strcmp(u8string, "算長度"), NULL);
}
END_TEST

START_TEST(test_strseek2)
{
	char *u8string = "測試計算長度";
	u8string = ueStrSeek(u8string, 0);
	fail_if(strcmp(u8string, "測試計算長度"), NULL);
}
END_TEST

Suite *utf8_suite (void)
{
	Suite *s = suite_create("UTF-8_Util");
	TCase *tc_core = tcase_create("Core");
	suite_add_tcase (s, tc_core);
	tcase_add_test (tc_core, test_strlen1);
	tcase_add_test (tc_core, test_strlen2);
	tcase_add_test (tc_core, test_strncpy);
	tcase_add_test (tc_core, test_strseek);
	tcase_add_test (tc_core, test_strseek2);
	return s;
}
