/**
 * hanyupinying.c
 *
 * Copyright (c) 2005
 *      libchewing Core Team. See ChangeLog for details.
 *
 * See the file "COPYING" for information on usage and redistribution
 * of this file.
 */

/* @(#)hanyupinying.c
 */

#include <stdio.h>
#include <string.h>
#include "hanyupinying.h"

/*
  according to http://www.arts.cuhk.edu.hk/Lexis/Lindict/
 */
#define HANYU_INITIALS 26
PinYingZuinMap hanyuInitialsMap[HANYU_INITIALS] = {
    {"b" , "1"}, {"p" , "q"}, {"m" , "a"}, {"f" ,"z"},
    {"d" , "2"}, {"t" , "w"}, {"n" , "s"}, {"l" ,"x"},
    {"g" , "e"}, {"k" , "d"}, {"h" , "c"},
    {"j" , "r"}, {"q" , "f"}, {"x" , "v"},
    {"zhi" , "5"}, {"zh" , "5"}, {"chi" , "t"}, {"ch" , "t"},
    {"shi" , "g"}, {"sh" , "g"}, {"ri" , "b"}, {"r" , "b"},
    {"z" , "y"}, {"c" , "h"}, {"s" , "n"}
};

#define HANYU_FINALS 70
PinYingZuinMap hanyuFinalsMap[HANYU_FINALS] = {
    {"uang","j;"}, {"wang","j;"},
    {"wong","j/"}, {"weng","j/"},
    {"ying","u/"},
    {"iong","m/"}, {"yong","m/"}, {"iung","m/"}, {"yung","m/"},
    {"iang","u;"}, {"yang","u;"},
    {"iuan","m0"}, {"yuan","m0"},
    {"ing","u/"},
    {"iao","ul"}, {"yao","ul"},
    {"iun","mp"}, {"yun","mp"},
    {"iou","u."}, {"you","u."},
    {"ian","u0"}, {"yan","u0"},
    {"yin","up"},
    {"ang",";"},
    {"eng","/"},
    {"iue","m,"}, {"yue","m,"},
    {"uai","j9"}, {"wai","j9"},
    {"wei","jo"},
    {"uan","j0"}, {"wan","j0"},
    {"wen","jp"},
    {"ong","j/"},
    {"van","m0"},
    {"ven","mp"},
    {"er","-"},
    {"ai","9"},
    {"ei","o"},
    {"ao","l"},
    {"ou","."},
    {"an","0"},
    {"en","p"},
    {"yi","u"},
    {"ia","u8"}, {"ya","u8"},
    {"ie","u,"}, {"ye","u,"},
    {"in","up"},
    {"io","m."},
    {"wu","j"},
    {"ua","j8"}, {"wa","j8"},
    {"uo","ji"}, {"wo","ji"},
    {"ui","jo"}, 
    {"iu","m"}, {"yu","m"},
    {"ue","m,"}, {"ve","m,"},
    {"un","mp"}, {"vn","mp"},
    {"a","8"},
    {"e","k"},
    {"i","u"},
    {"o","i"},
    {"v","m"},
    {"u","j"},
    {"E",","}
};

/*
  0: Success
  Non-Zero: Fail to fully convert
  1: Failed fo lookup initals
  2: Failed fo lookup finals
  Map pinyin key-sequence to Zuin key-sequence.
  Caller should allocate char zuin[4].
 */

int HanyuPinYingToZuin( char *pinyingKeySeq, char *zuinKeySeq )
{
	/*
	 * pinyinKeySeq[] should have at most 6 letters (Shuang)
	 * zuinKeySeq[] has at most 3 letters.
	 */
	char *p, *cursor;
	char *initial = 0;
	char *final   = 0;
	int i;
	for (i = 0; i < HANYU_INITIALS; i++ ) {
		p = strstr( pinyingKeySeq, hanyuInitialsMap[ i ].pinying );
		if ( p == pinyingKeySeq ) {
			initial = hanyuInitialsMap[ i ].zuin;
			cursor = pinyingKeySeq +
				strlen( hanyuInitialsMap[ i ].pinying );
			break;
		}
	}
	if ( i == (HANYU_INITIALS) ) {
		/* No initials. might be £¸£¹£º */
		/* XXX: I NEED Implementation
		   if(finalsKeySeq[0] != ) {
		   }
		   */
		return 1;
	}

	if ( cursor ) {
		for ( i = 0; i < HANYU_FINALS; i++ ) {
			p = strstr( cursor, hanyuFinalsMap[ i ].pinying );
			if ( p == cursor ) {
				final = hanyuFinalsMap[ i ].zuin;
				break;
			}
		}
		if ( i == HANYU_FINALS )
			return 2;
	}
	
	if ( ! strcmp( final, "j0" ) ) {
		if (
			! strcmp( initial, "f" ) || 
			! strcmp( initial, "r" ) ||
			! strcmp( initial,"v" ) ) {
			final = "m0";
		}
	}

	sprintf( zuinKeySeq, "%s%s\0", initial, final );
	return 0;
}
