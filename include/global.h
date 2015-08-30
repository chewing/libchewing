/*
 * global.h
 *
 * Copyright (c) 1999, 2000, 2001
 *      Lu-chuan Kung and Kang-pen Chen.
 *      All rights reserved.
 *
 * Copyright (c) 2004, 2005, 2006, 2008, 2011
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* *INDENT-OFF* */
#ifndef _CHEWING_GLOBAL_H
#define _CHEWING_GLOBAL_H
/* *INDENT-ON* */

/*! \file global.h
 *  \brief Chewing Global Definitions
 *  \author libchewing Core Team
*/

#define CHINESE_MODE 1
#define SYMBOL_MODE 0
#define FULLSHAPE_MODE 1
#define HALFSHAPE_MODE 0

/* specified to Chewing API */
#if defined(_WIN32) || defined(_WIN64) || defined(_WIN32_WCE)
#    define CHEWING_DLL_IMPORT __declspec(dllimport)
#    define CHEWING_DLL_EXPORT __declspec(dllexport)
#    ifdef CHEWINGDLL_EXPORTS
#        define CHEWING_API CHEWING_DLL_EXPORT
#        define CHEWING_PRIVATE
#    elif CHEWINGDLL_IMPORTS
#        define CHEWING_API CHEWING_DLL_IMPORT
#        define CHEWING_PRIVATE
#    else
#        define CHEWING_API
#        define CHEWING_PRIVATE
#    endif
#elif (__GNUC__ > 3) && (defined(__ELF__) || defined(__PIC__))
#    define CHEWING_API __attribute__((__visibility__("default")))
#    define CHEWING_PRIVATE __attribute__((__visibility__("hidden")))
#else
#    define CHEWING_API
#    define CHEWING_PRIVATE
#endif

#ifndef UNUSED
#    if defined(__GNUC__)       /* gcc specific */
#        define UNUSED __attribute__((unused))
#    else
#        define UNUSED
#    endif
#endif

#ifndef DEPRECATED
#    if defined(__GNUC__) && __GNUC__ > 3 || \
        (__GNUC__ == 3 && __GNUC_MINOR__ >= 1) /* gcc specific */
#        define DEPRECATED __attribute__((deprecated))
#        if __GNUC__ > 4 || (__GNUC__ == 4 && __GNUC_MINOR__ >= 5)
#            define DEPRECATED_FOR(f) \
                 __attribute__((deprecated("Use " #f " instead")))
#        else
#            define DEPRECATED_FOR(f) DEPRECATED
#        endif
#    else
#        define DEPRECATED
#        define DEPRECATED_FOR(f)
#    endif
#endif

/* The following macros are modified from GLIB.
 * from GNU cpp Manual:
 * C99 introduces the _Pragma operator. This feature addresses a major problem
 * with `#pragma': being a directive, it cannot be produced as the result of
 * macro expansion. _Pragma is an operator, much like sizeof or defined, and
 * can be embedded in a macro.
 */
#if defined(__clang__) || (defined(__GNUC__) && (__GNUC__ > 4 || (__GNUC__ == 4 && __GNUC_MINOR__ >= 6)))
#    define BEGIN_IGNORE_DEPRECATIONS \
         _Pragma ("GCC diagnostic push")                       \
         _Pragma ("GCC diagnostic ignored \"-Wdeprecated-declarations\"")
#    define END_IGNORE_DEPRECATIONS                  \
         _Pragma ("GCC diagnostic pop")
#elif defined (_MSC_VER) && (_MSC_VER >= 1500)
#    define BEGIN_IGNORE_DEPRECATIONS \
         __pragma (warning (push))  \
         __pragma (warning (disable : 4996))
#    define END_IGNORE_DEPRECATIONS \
         __pragma (warning (pop))
#else
#    define BEGIN_IGNORE_DEPRECATIONS
#    define END_IGNORE_DEPRECATIONS
#endif

#define MIN_SELKEY 1
#define MAX_SELKEY 10

#define CHEWING_LOG_VERBOSE 1
#define CHEWING_LOG_DEBUG   2
#define CHEWING_LOG_INFO    3
#define CHEWING_LOG_WARN    4
#define CHEWING_LOG_ERROR   5

/**
 * @deprecated Use chewing_set_ series of functions to set parameters instead.
 */
typedef struct ChewingConfigData {
    int candPerPage;
    int maxChiSymbolLen;
    int selKey[MAX_SELKEY];
    int bAddPhraseForward;
    int bSpaceAsSelection;
    int bEscCleanAllBuf;
    int bAutoShiftCur;
    int bEasySymbolInput;
    int bPhraseChoiceRearward;
    int hsuSelKeyType;          // Deprecated.
} ChewingConfigData;

typedef struct IntervalType {
    /*@{ */
    int from;           /**< starting position of certain interval */
    int to;             /**< ending position of certain interval */
    /*@} */
} IntervalType;

/** @brief context handle used for Chewing IM APIs
 */
typedef struct ChewingContext ChewingContext;

/** @brief use "asdfjkl789" as selection key
 */
#define HSU_SELKEY_TYPE1 1

/** @brief use "asdfzxcv89" as selection key
 */
#define HSU_SELKEY_TYPE2 2

/* *INDENT-OFF* */
#endif
/* *INDENT-ON* */
