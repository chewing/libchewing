/* @(#)hanyupinying.c
 */

#include <stdio.h>
#include <string.h>
#include "hanyupinying.h"

/*
  according to http://oclccjk.lib.uci.edu/pycywg.htm
 */
#define HANYU_INITIALS 23
PinYingZuinMap hanyuInitialsMap[HANYU_INITIALS] = {
    {"b" , "1"}, {"p" , "q"}, {"m" , "a"}, {"f" ,"z"},
    {"d" , "2"}, {"t" , "w"}, {"n" , "s"}, {"l" ,"x"},
    {"g" , "e"}, {"k" , "d"}, {"h" , "c"},
    {"j" , "r"}, {"g" , "f"}, {"x" , "v"},
    {"zh", "5"}, {"ch", "t"}, {"sh", "g"}, {"r" ,"b"},
    {"z" , "y"}, {"c" , "h"}, {"s" , "n"},
    {"er", "-"}
};

#define HANYU_FINALS 40
PinYingZuinMap hanyuFinalsMap[HANYU_FINALS] = {
    {"iang","u;"},
    {"iong","m/"},
    {"uang","j;"},
    {"uang","j;"},
    {"yuan","m0"},

    {"ang",";"},
    {"eng","/"},
    {"ian","u0"},
    {"iao","ul"},
    {"ing","u/"},
    {"ong","j/"},
    {"uai","j9"},
    {"uan","j0"},
    {"uei","jo"},
    {"yue","m,"},
    {"yun","mp"},

    {"ai","9" },
    {"an","0" },
    {"ao","l" },
    {"ei","o" },
    {"en","p" },
    {"ia","u8"},
    {"ie","u,"},
    {"in","up"},
    {"iu","u."},
    {"ou","l" },
    {"ou","l" },
    {"un","jp"},
    {"wa","j8"},
    {"wo","ji"},
    {"ya","m8"},
    {"yu","m"},

    {"a","8"},
    {"e",","},
    {"e","k"},
    {"o","i"},
    {"w","j"},
    {"y","u"},
    {"r","-"}
};

/*
  0: Success
  Non-Zero: Fail to fully convert
  1: Failed fo lookup initals
  2: Failed fo lookup finals
  Map pinyin key-sequence to Zuin key-sequence.
  Caller should allocate char zuin[4].
 */

int
HanyuPinYingToZuin(char *pinyingKeySeq, char *zuinKeySeq) {
    // pinyinKeySeq[] should have at most 6 letters (Shuang)
    // zuinKeySeq[] has at most 3 letters.
    char *p,*cursor;
    char *initial = 0;
    char *final   = 0;
    int i;
    for(i=0;i< HANYU_INITIALS;i++) {
        p = strstr( pinyingKeySeq, hanyuInitialsMap[i].pinying);
        if(p == pinyingKeySeq) {
            initial = hanyuInitialsMap[i].zuin;
            cursor = pinyingKeySeq + strlen(hanyuInitialsMap[i].pinying);
            break;
        }
    }
    if(i == HANYU_INITIALS) {
        // No initials. might be £¸£¹£º
        /* XXX: I NEED Implementation
        if(finalsKeySeq[0] != ) {
        }
        */
        return 1;
    }

    if(cursor) {
        for(i=0;i< HANYU_FINALS;i++) {
            p = strstr(cursor,hanyuFinalsMap[i].pinying);
            if(p == cursor) {
                final = hanyuFinalsMap[i].zuin;
                break;
            }
        }
        if(i == HANYU_FINALS) return 2;
    }

    sprintf(zuinKeySeq,"%s%s\0",initial,final);
    return 0;
}

int main(void) {
    char z[5];
    char p[255];
    int err;
    while(scanf("%s",p) == 1) {
        err = HanyuPinYingToZuin(p,z);
        if(!err) {
            printf("==> (%s) -> (%s)\n",p,z);
        } else {
            printf("Error %n (%s)\n",err, p);
        }
    }
    return 0;
}
