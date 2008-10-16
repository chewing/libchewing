/**
 * global.h
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004, 2005, 2006, 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_GLOBAL_H
#define _CHEWING_GLOBAL_H

#include <stdio.h>
#include <stdlib.h>

#define IS_USER_PHRASE 1
#define IS_DICT_PHRASE 0
#define CHINESE_MODE 1
#define SYMBOL_MODE 0
#define FULLSHAPE_MODE 1
#define HALFSHAPE_MODE 0

/* specified to Chewing API */
#ifdef WIN32
    #define CHEWING_DLL_IMPORT __declspec(dllimport)
    #define CHEWING_DLL_EXPORT __declspec(dllexport)
#   ifdef CHEWINGDLL_EXPORTS
#      define CHEWING_API CHEWING_DLL_EXPORT
#      define CHEWING_PRIVATE
#   else
#      define CHEWING_API CHEWING_DLL_IMPORT
#      define CHEWING_PRIVATE
#   endif
#elif (__GNUC__ > 3) && defined(__ELF__)
#   define CHEWING_API __attribute__((__visibility__("default")))
#   define CHEWING_PRIVATE __attribute__((__visibility__("hidden")))
#else
#  define CHEWING_API
#  define CHEWING_PRIVATE
#endif

#ifndef UNUSED
#if defined(__GNUC__) /* gcc specific */
#   define UNUSED __attribute__((unused))
#else
#   define UNUSED
#endif
#endif

#define MAX_SELKEY 10

typedef struct {
	int candPerPage;
	int maxChiSymbolLen;
	int selKey[ MAX_SELKEY ];
	int bAddPhraseForward;
	int bSpaceAsSelection;
	int bEscCleanAllBuf;
	/** @brief
            HSU_SELKEY_TYPE1 = asdfjkl789,
            HSU_SELKEY_TYPE2 = asdfzxcv89.
         */
	int hsuSelKeyType;
} ChewingConfigData;
/** @brief deprecated */

typedef struct {
	int from, to;
} IntervalType;

typedef unsigned short uint16;
typedef struct _ChewingContext ChewingContext;

/** @brief use "asdfjkl789" as selection key */
#define HSU_SELKEY_TYPE1 1
/** @brief use "asdfzxcv89" as selection key */
#define HSU_SELKEY_TYPE2 2

#endif
