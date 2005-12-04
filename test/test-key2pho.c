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
#include <check.h>
#include "chewing-utf8-util.h"

START_TEST(test_UintFromPhone)
{
	char *u8phone = "ㄆㄣ";
	fail_if(UintFromPhone(u8phone) != 1104, NULL);
	u8phone = "ㄊㄧㄢ";
	fail_if(UintFromPhone(u8phone) != 3272, NULL);
	u8phone = "ㄒㄧㄚˋ";
	fail_if(UintFromPhone(u8phone) != 7308, NULL);
}
END_TEST

START_TEST(test_PhoneFromKey)
{
	char rt[10];
	PhoneFromKey( rt, "dj", 0, 1 );
	fail_if( strcmp(rt, "ㄎㄨ"), NULL);
	PhoneFromKey( rt, "dj6", 0, 1 );
	fail_if( strcmp(rt, "ㄎㄨˊ"), NULL);
	PhoneFromKey( rt, "dj3", 0, 1 );
	fail_if( strcmp(rt, "ㄎㄨˇ"), NULL);
	PhoneFromKey( rt, "dj4", 0, 1 );
	fail_if( strcmp(rt, "ㄎㄨˋ"), NULL);
	PhoneFromKey( rt, "dj7", 0, 1 );
	fail_if( strcmp(rt, "ㄎㄨ˙"), NULL);
}
END_TEST

Suite *key2pho_suite (void)
{
	Suite *s = suite_create("key2pho.c");
	TCase *tc_core = tcase_create("Core");
	suite_add_tcase (s, tc_core);
	tcase_add_test (tc_core, test_UintFromPhone);
	tcase_add_test (tc_core, test_PhoneFromKey);
	return s;
}
