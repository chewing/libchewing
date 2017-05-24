//#include "func.h"

TEST(chewingtest,boundarytest){
	//EXPECT_STREQ("「",chewing_commit_String(convert2ctx("[")));
	EXPECT_EQ(0,chewing_cand_hasNext(convert2ctx("[ ")));
}

TEST(chewingtest,boundarytest2){
	EXPECT_EQ(0,chewing_cand_hasNext(convert2ctx("] ")));
}

TEST(chewingtest,boundarytest3){
	EXPECT_EQ(1,chewing_cand_hasNext(convert2ctx("- ")));
}

TEST(chewingtest,boundarytest4){
	EXPECT_EQ(1,chewing_cand_hasNext(convert2ctx("1 ")));
}

TEST(chewingtest,boundarytest5){
	EXPECT_EQ(0,chewing_cand_hasNext(convert2ctx("= ")));
}