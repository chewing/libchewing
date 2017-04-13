#include "func.h"

TEST(chewingtest,boundarytest){
	EXPECT_STREQ("「",chewing_commit_String(convert2ctx("[")));
}
TEST(chewingtest,boundarytest2){
    EXPECT_STREQ("」",chewing_commit_String(convert2ctx("]")));
}
TEST(chewingtest,boundarytest3){
	EXPECT_STRNE("」",chewing_commit_String(convert2ctx("[")));
}
TEST(chewingtest,boundarytest4){
       EXPECT_STREQ("＝",chewing_commit_String(convert2ctx("=")));
}