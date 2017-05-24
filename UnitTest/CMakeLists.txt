cmake_minimum_required(VERSION 2.6)

project(UnitTest)

find_package(PkgConfig)
find_package(GTest REQUIRED)
pkg_check_modules(CHEWING REQUIRED chewing)

include_directories(${CHEWING_INCLUDE_DIRS} ${GTEST_INCLUDE_DIRS} ../src/porting_layer/include ../include/ ../include/internal )
link_directories(${CHEWING_LIBRARY_DIRS})

add_executable(UnitTest UnitTest.cpp)
target_link_libraries(UnitTest ${CHEWING_LIBRARIES} ${GTEST_LIBRARIES} pthread)
