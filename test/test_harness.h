/**
 * test_harness.h
 *
 * Copyright (c) 2012
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/*
 * A minimum TAP, Test Anything Protocol, like test harness framework
 *
 * Simple test case should look like:
 *
 * #include "test_harness.h"
 * int main (int argc, char *argv[])
 * {
 *         ok (1 + 1 == 2, "1 + 1 is 2");
 *         return exit_status();
 * }
 */

#ifndef TEST_HARNESS_H
#define TEST_HARNESS_H

static unsigned int _test_run_;
static unsigned int _test_ok_;

#define ok(test, message)                                               \
        do {                                                            \
                _test_run_++;                                           \
                if (test) {                                             \
                        _test_ok_++;                                    \
                        printf("ok %d %s\n", _test_run_, message);      \
                } else {                                                \
                        printf("not ok %d %s\n", _test_run_,            \
                               message);                                \
                        printf("# %s failed in %s:%d\n",                \
                               #test, __FILE__, __LINE__);              \
                }                                                       \
        } while (0)

#define exit_status()                           \
        (_test_run_ == _test_ok_ ? 0 : -1)

#endif
