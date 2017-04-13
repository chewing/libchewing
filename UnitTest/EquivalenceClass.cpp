//#include "func.h"


TEST( TriEquivalence, WeekNormal){
	char NVa[4] = "2rg";	//ㄉ ㄐ ㄕ
	char NVb[5] = "j8l;";	//ㄨ ㄜ ㄠ ㄤ
	EXPECT_STRNE("",chewing_commit_String(convert2ctx_bychar(NVa[0], NVb[0])));
	EXPECT_STRNE("",chewing_commit_String(convert2ctx_bychar(NVa[2], NVb[1])));
	EXPECT_STREQ("",chewing_commit_String(convert2ctx_bychar(NVa[1], NVb[2])));
	EXPECT_STRNE("",chewing_commit_String(convert2ctx_bychar(NVa[0], NVb[3])));
}

TEST( TriEquivalence, StrongNormal){
	char NVa[4] = "2rg";	//ㄉ ㄐ ㄕ
	char NVb[5] = "j8l;";	//ㄨ ㄜ ㄠ ㄤ
	EXPECT_STRNE("",chewing_commit_String(convert2ctx_bychar(NVa[0], NVb[0])));
	EXPECT_STRNE("",chewing_commit_String(convert2ctx_bychar(NVa[0], NVb[1])));
	EXPECT_STRNE("",chewing_commit_String(convert2ctx_bychar(NVa[0], NVb[2])));
	EXPECT_STRNE("",chewing_commit_String(convert2ctx_bychar(NVa[0], NVb[3])));
	EXPECT_STREQ("",chewing_commit_String(convert2ctx_bychar(NVa[1], NVb[0])));
	EXPECT_STREQ("",chewing_commit_String(convert2ctx_bychar(NVa[1], NVb[1])));
	EXPECT_STREQ("",chewing_commit_String(convert2ctx_bychar(NVa[1], NVb[2])));
	EXPECT_STREQ("",chewing_commit_String(convert2ctx_bychar(NVa[1], NVb[3])));
	EXPECT_STRNE("",chewing_commit_String(convert2ctx_bychar(NVa[2], NVb[0])));
	EXPECT_STRNE("",chewing_commit_String(convert2ctx_bychar(NVa[2], NVb[1])));
	EXPECT_STRNE("",chewing_commit_String(convert2ctx_bychar(NVa[2], NVb[2])));
	EXPECT_STRNE("",chewing_commit_String(convert2ctx_bychar(NVa[2], NVb[3])));
}