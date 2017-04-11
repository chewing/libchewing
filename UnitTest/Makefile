GTEST_DIR = ~/1052Testing/googletest-master/googletest
SRC_DIR = ..
PROG = UnitTest.cpp
#PROG = chewing_test.cpp
LIB = -I$(SRC_DIR)/include \
		 -I$(SRC_DIR)/include/internal \
		 -I$(SRC_DIR)/src/porting_layer/include/ \
		 -I$(SRC_DIR)/test \
#LIB = -I/usr/local/include/chewing \
#		 -I/usr/local/include \
#		 -L/usr/local/lib \
#		 -lchewing

#all: gtest-all.o libgtest.a
#	g++ -isystem ${GTEST_DIR}/include -pthread $(LIB) $(PROG) libgtest.a -o UnitTest
	
all:
	g++ $(LIB) $(PROG) -o UnitTest -m32

init:
	gcc -isystem ${GTEST_DIR}/include -I${GTEST_DIR} \
	    -pthread -c ${GTEST_DIR}/src/gtest-all.cc
	ar -rv libgtest.a gtest-all.o