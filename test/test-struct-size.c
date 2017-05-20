/**
 * test-struct-size.c
 *
 * Copyright (c) 2014
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */
#include "testhelper.h"

typedef struct OrigianlChewingConfigData {
    int candPerPage;
    int maxChiSymbolLen;
    int selKey[MAX_SELKEY];
    int bAddPhraseForward;
    int bSpaceAsSelection;
    int bEscCleanAllBuf;
    int bAutoShiftCur;
    int bEasySymbolInput;
    int bPhraseChoiceRearward;
    int bAutoLearn;
    int hsuSelKeyType;
} OrigianlChewingConfigData;

typedef struct OrigianlIntervalType {
    int from;
    int to;
} OrigianlIntervalType;

void test_ChewingConfigData()
{
    size_t expect = sizeof(OrigianlChewingConfigData);
    size_t actual = sizeof(ChewingConfigData);
    ok(actual == expect,
        "sizeof(ChewingConfigData) = %d shall be %d for ABI compatibility", actual, expect);
}

void test_IntervalType()
{
    size_t expect = sizeof(OrigianlIntervalType);
    size_t actual = sizeof(IntervalType);
    ok(actual == expect,
        "sizeof(IntervalType) = %d shall be %d for ABI compatibility", actual, expect);
}

int main()
{
    test_ChewingConfigData();
    test_IntervalType();

    return exit_status();
}
