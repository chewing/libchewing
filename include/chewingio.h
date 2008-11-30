/*
 * chewingio.h
 *
 * Copyright (c) 1999, 2000, 2001
 *	Lu-chuan Kung and Kang-pen Chen.
 *	All rights reserved.
 *
 * Copyright (c) 2004, 2005, 2008
 *	libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

#ifndef _CHEWING_IO_H
#define _CHEWING_IO_H

/*! \file chewingio.h
 *  \brief Chewing I/O module
 *  \author libchewing Core Team
 */

#include "global.h"

#define KEYSTROKE_IGNORE 1
#define KEYSTROKE_COMMIT 2
#define KEYSTROKE_BELL 4
#define KEYSTROKE_ABSORB 8

/*! \name Series of functions handling key stroke.
 */

/*@{*/
/**
 * @brief Handle the input key stroke: Space
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_Space( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Escape
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_Esc( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Enter/Return
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_Enter( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Delete
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_Del( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Backspace
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_Backspace( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Tab
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_Tab( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Shift + Left
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_ShiftLeft( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Left
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_Left( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Shift + Right
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_ShiftRight( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Right
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_Right( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Up
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_Up( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Home
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_Home( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: End
 * @param ctx Chewing IM context
 */

CHEWING_API int chewing_handle_End( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: PageUp
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_PageUp( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: PageDown
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_PageDown( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Down
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_Down( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Capslock
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_Capslock( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: casual key
 * @param ctx Chewing IM context
 * @param key scan code of key stroke
 */
CHEWING_API int chewing_handle_Default( ChewingContext *ctx, int key );

/**
 * @brief Handle the input key stroke: Ctrl + Number-key
 * @param ctx Chewing IM context
 * @param key scan code of number key
 */
CHEWING_API int chewing_handle_CtrlNum( ChewingContext *ctx, int key );

/**
 * @brief Handle the input key stroke: Shift + Space
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_ShiftSpace( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: double Tab
 * @param ctx Chewing IM context
 */
CHEWING_API int chewing_handle_DblTab( ChewingContext *ctx );

/**
 * @brief Handle the input key stroke: Numlock (keypad)
 * @param ctx Chewing IM context
 * @param key scan code of number key
 */
CHEWING_API int chewing_handle_Numlock( ChewingContext *ctx, int key);
/*@}*/


/*! \name Chewing IM Instance Management
 */

/*@{*/
/**
 * @brief Create new handle of the instance for Chewing IM
 * @see chewing_delete()
 */
CHEWING_API ChewingContext *chewing_new();

/**
 * @brief Release the handle and internal memory by given Chewing instance
 * @see chewing_new()
 *
 * @param ctx Chewing IM context
 */
CHEWING_API void chewing_delete( ChewingContext *ctx );

/**
 * @brief Release memory allocated used by given pointer used in APIs
 */
CHEWING_API void chewing_free( void * );
/*@}*/


/*! \name Chewing IM Setup
 */

/*@{*/
/**
 * @brief Initialize directory data used by Chewing IM
 * @see chewing_Terminate()
 *
 * @param dataPath (read-only) system path of Chewing IM data
 * @param hashPath (read-write) the path where user-defined hash data stores
 * @retval 0 if succeed
 */
CHEWING_API int chewing_Init( const char *dataPath, const char *hashPath );

/**
 * @brief Reset all settings
 *
 * @param ctx
 * @return If successed than return 0
 */
CHEWING_API int chewing_Reset( ChewingContext *ctx );

/**
 * @brief Terminate the I/O routines of Chewing IM
 * @see chewing_Init()
 */
CHEWING_API void chewing_Terminate();

/**
 * @brief Set selectAreaLen, maxChiSymbolLen, selKey in pcd.
 * @deprecated Use chewing_set_ series of functions to set parameters instead.
 *
 * @param ctx Chewing IM context
 * @param pcd pointer to Chewing configuration data structure
 */
CHEWING_API int chewing_Configure( ChewingContext *ctx, ChewingConfigData *pcd );

/*@}*/

/*! \name Keyboard mapping
 */

/*@{*/
/**
 * @brief Set keyboard mapping type
 *
 * @param ctx
 * @param kbtype index number of keyboard mapping type from KBStr2Num
 * @return If successed then return 0
 */
CHEWING_API int chewing_set_KBType( ChewingContext *ctx, int kbtype );

/**
 * @brief Get keyboard mapping type
 *
 * @param ctx
 * @return If successed then return keyboard mapping type from KBStr2Num
 */
CHEWING_API int chewing_get_KBType( ChewingContext *ctx );

/**
 * @brief Get keyboard mapping type in C-style string format
 *
 * @param ctx
 * @return If successed then return kbtype from KBStr2Num
 */
CHEWING_API char* chewing_get_KBString( ChewingContext *ctx );

/**
 * @brief Get the index number of keyboard mapping type from given string
 *
 * @param str[] name of kbtype eg. "KB_HSU"
 */
CHEWING_API int chewing_KBStr2Num( char str[] );
/*@}*/


/*! \name Operating language mode of Chewing IM
 */

/*@{*/
/**
 * @brief Set the operating language mode of Chewing IM.
 *
 * @param ctx
 * @param mode CHINESE_MODE or ENGLISH_MODE
 */
CHEWING_API void chewing_set_ChiEngMode( ChewingContext *ctx, int mode );

/**
 * @brief Get current operating language mode: English / Chinese
 *
 * @param ctx
 * 
 * @return CHINESE_MODE or ENGLISH_MODE
 */
CHEWING_API int chewing_get_ChiEngMode( ChewingContext *ctx );
/*@*/


/*! \name Shape mode of output symbols
 */

/*@{*/
/**
 * @brief Set the shape mode of output symbols: full-shape / half-shape
 *
 * @param ctx
 * @param mode FULLSHAPE_MODE or HALFSHAPE_MODE
 */
CHEWING_API void chewing_set_ShapeMode( ChewingContext *ctx, int mode );

/**
 * @brief Get current shape mode of output symbols
 *
 * @param ctx
 * 
 * @return FULLSHAPE_MODE or HALFSHAPE_MODE
 */
CHEWING_API int chewing_get_ShapeMode( ChewingContext *ctx );
/*@}*/


/*! \name Number of selection candidate per page
 */

/*@{*/
/**
 * @brief Set the number of selection candidate per page
 *
 * @param ctx
 * @param n number of selection candidate
 */
CHEWING_API void chewing_set_candPerPage( ChewingContext *ctx, int n );

/**
 * @brief Get the number of selection candidate per page
 *
 * @param ctx
 */
CHEWING_API int chewing_get_candPerPage( ChewingContext *ctx );
/*@}*/


/*! \name Maximum length of Chinese symbols
 */

/*@{*/
/**
 * @brief Set the maximum length of Chinese symbols
 *
 * @param ctx
 * @param n maximum length
 */
CHEWING_API void chewing_set_maxChiSymbolLen( ChewingContext *ctx, int n );

/**
 * @brief Get the maximum length of Chinese symbols
 *
 * @param ctx
 */
CHEWING_API int chewing_get_maxChiSymbolLen( ChewingContext *ctx );
/*@}*/


/*! \name Key sequence for selecting phrases
 */

/*@{*/
/**
 * @brief Set the key sequence for selecting phrases
 *
 * @param ctx
 * @param selkeys
 * @param len
 */
CHEWING_API void chewing_set_selKey( ChewingContext *ctx, int *selkeys, int len );

/**
 * @brief Get the key sequence for selecting phrases
 *
 * @param ctx
 */
CHEWING_API int* chewing_get_selKey( ChewingContext *ctx );
/*@}*/


/*! \name Direction of adding new user-defined phrases
 */

/*@{*/
/**
 * @brief Set the direction of adding new user-defined phrases
 *
 * @param ctx
 * @param direction
 */
CHEWING_API void chewing_set_addPhraseDirection( ChewingContext *ctx, int direction );

/**
 * @brief Get the direction of adding new user-defined phrases
 *
 * @param ctx
 */
CHEWING_API int chewing_get_addPhraseDirection( ChewingContext *ctx );
/*@}*/


/*! \name Behavior whether if space key is regarded as selection key
 */

/*@{*/
/**
 * @brief Set the behavior whether if space key is regarded as selection key
 *
 * @param ctx
 * @param mode
 */
CHEWING_API void chewing_set_spaceAsSelection( ChewingContext *ctx, int mode );

/**
 * @brief Get the behavior if space key is regarded as selection key or not
 *
 * @param ctx
 */
CHEWING_API int chewing_get_spaceAsSelection( ChewingContext *ctx );
/*@}*/


/*! \name Behavior whether if Escape should clean all buffer
 */

/*@{*/
/**
 * @brief Set the behavior whether if Escape key should clean all buffer
 *
 * @param ctx
 * @param mode
 */
CHEWING_API void chewing_set_escCleanAllBuf( ChewingContext *ctx, int mode );

/**
 * @brief Get the behavior whether if Escape key should clean all buffer
 *
 * @param ctx
 */
CHEWING_API int chewing_get_escCleanAllBuf( ChewingContext *ctx );
/*@}*/


/*! \name Type of selection keys in Hsu's keyboard mapping
 */

/*@{*/
/**
 * @brief Set the type of selection keys in Hsu's keyboard mapping
 *
 * @param ctx
 * @param mode
 */
CHEWING_API void chewing_set_hsuSelKeyType( ChewingContext *ctx, int mode );

/**
 * @brief Get the type of selection keys in Hsu's keyboard mapping
 *
 * @param ctx
 */
CHEWING_API int chewing_get_hsuSelKeyType( ChewingContext *ctx );
/*@}*/


/*! \name Behavior of automatically shifting cursor after selection
 */

/*@{*/
/**
 * @brief Set the behavior of automatically shifting cursor after selection
 *
 * @param ctx
 * @param mode
 */
CHEWING_API void chewing_set_autoShiftCur( ChewingContext *ctx, int mode );

/**
 * @brief Get the behavior of automatically shifting cursor after selection
 *
 * @param ctx
 */
CHEWING_API int chewing_get_autoShiftCur( ChewingContext *ctx );
/*@}*/


/*! \name Operating mode during easy symbol input
 */

/*@{*/
/**
 * @brief Set the operating mode during easy symbol input
 *
 * @param ctx
 * @param mode
 */
CHEWING_API void chewing_set_easySymbolInput( ChewingContext *ctx, int mode );

/**
 * @brief Get the operating mode during easy symbol input
 *
 * @param ctx
 */
CHEWING_API int chewing_get_easySymbolInput( ChewingContext *ctx );
/*@}*/


/*! \name Behavior for phrase choice to be rearward or not
 */

/*@{*/
/**
 * @brief Set the behavior for phrase choice to be rearward or not
 *
 * @param ctx
 * @param mode
 */
CHEWING_API void chewing_set_phraseChoiceRearward( ChewingContext *ctx, int mode );

/**
 * @brief Get the behavior for phrase choice to be rearward or not
 *
 * @param ctx
 */
CHEWING_API int chewing_get_phraseChoiceRearward( ChewingContext *ctx );
/*@}*/


/*! \name Phonetic sequence in Chewing internal state machine
 */

/*@{*/
/**
 * @brief Get phonetic sequence in Chewing internal state machine
 *
 * @param ctx
 */
CHEWING_API uint16 *chewing_get_phoneSeq( ChewingContext *ctx );

/**
 * @brief Get the length of phonetic sequence in Chewing internal state machine
 *
 * @param ctx
 */
CHEWING_API int chewing_get_phoneSeqLen( ChewingContext *ctx );
/*@}*/

#endif /* _CHEWING_IO_H */
