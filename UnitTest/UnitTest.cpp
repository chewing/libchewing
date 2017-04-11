#include <chewing.h>
#include <testhelper.h>
#include <iostream>
using namespace std;

int main( int argc, char** argv){
	//testing::InitGoogleTest( &argc, argv);
	//return RUN_ALL_TESTS();
	ChewingContext* ctx = chewing_new();
	type_keystroke_by_string( ctx, "su3cl3");
  
	if( chewing_commit_Check(ctx)){
		cout << chewing_commit_String(ctx) << endl;
	}
	cout << "finish" << endl;
	return 0;
}