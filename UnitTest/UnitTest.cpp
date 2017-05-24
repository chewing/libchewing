#include <gtest/gtest.h>
#include <chewing.h>
#include <iostream>
#include "func.h"
//#include "BoundaryValueTest.cpp"
//#include "EquivalenceClass.cpp"
#include "PathTesting.cpp"

using namespace std;

int main( int argc, char** argv){
	testing::InitGoogleTest( &argc, argv);
	return RUN_ALL_TESTS();
	
	return 0;
}
