/**
 * do-test.c
 *
 * Copyright (c) 2005
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#include <check.h>

extern Suite *key2pho_suite (void);
extern Suite *utf8_suite (void);
extern Suite *mmap_suite(void);

int main (void)
{
	int nf;
	SRunner *sr = srunner_create(key2pho_suite());
	srunner_add_suite(sr, utf8_suite());
	srunner_add_suite(sr, mmap_suite());
	srunner_set_xml(sr, "do-test.xml");
	srunner_run_all (sr, CK_NORMAL);
	nf = srunner_ntests_failed(sr);
	srunner_free(sr);
	return (nf == 0) ? 0 : 1;
}
