/**
 * test-mmap.c
 *
 * Copyright (c) 2005
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifdef HAVE_CONFIG_H
#include <config.h>
#endif

#include <stdio.h>
#include <string.h>
#include <check.h>
#include "plat_mmap.h"

START_TEST(test_UnitFromPlatMmap)
{
	unsigned int idx;
	plat_mmap m_mmap;
	size_t offset = 0;
	size_t csize;
	char *data_buf;
	char hard_copy[] = "ji3cp3vu3cj0 vup dj4up <E>";
	int i;
	
	idx = plat_mmap_create(&m_mmap, "default-test.txt", FLAG_ATTRIBUTE_READ);
	fail_if(idx != 28);
	if (idx > 0) {
		csize = idx;
		data_buf = (char *) plat_mmap_set_view(&m_mmap, &offset, &csize);
		for (i = 0; i < 26; i++) {
			fail_if(data_buf[i] != hard_copy[i]);
		}
	}
	plat_mmap_close( &m_mmap );
}
END_TEST


Suite *mmap_suite (void)
{
	Suite *s = suite_create("plat_mmap");
	TCase *tc_core = tcase_create("Core");
	suite_add_tcase (s, tc_core);
	tcase_add_test (tc_core, test_UnitFromPlatMmap);
	return s;
}
