cmake_minimum_required(VERSION 2.6)

project(chewing_test)

find_package(PkgConfig)
pkg_check_modules(CHEWING REQUIRED chewing)

include_directories(${CHEWING_INCLUDE_DIRS})
link_directories(${CHEWING_LIBRARY_DIRS})

add_executable(chewing_test chewing_test.cpp)
target_link_libraries(chewing_test ${CHEWING_LIBRARIES})
