cmake_minimum_required(VERSION 2.8.8)
project(libchewing)

set(LIBCHEWING_VERSION 0.3.4)

string(COMPARE NOTEQUAL ${PROJECT_SOURCE_DIR} ${PROJECT_BINARY_DIR} OUT_OF_SOURCE)

option(ENABLE_DEBUG "Debug support" false)
if(ENABLE_DEBUG)
	add_definitions(-DENABLE_DEBUG=1)
endif()

if(UNIX)
	set(CMAKE_C_FLAGS "-g -O2 -Wall -fPIC ${CMAKE_C_FLAGS}")
	add_definitions(-DUNDER_POSIX -DPIC)
endif()

if(CMAKE_C_COMPILER_ID STREQUAL "GNU")
	set(CMAKE_C_FLAGS "-std=gnu99 ${CMAKE_C_FLAGS}")
	add_definitions(-D_GNU_SOURCE)
	option(ENABLE_GCOV "Coverage support" false)
	if(ENABLE_GCOV)
		set(CMAKE_C_FLAGS "-coverage ${CMAKE_C_FLAGS}")
	endif()
elseif(CMAKE_C_COMPILER_ID STREQUAL "Clang")
	add_definitions(-D_GNU_SOURCE)
	option(ENABLE_GCOV "Coverage support" false)
	if(ENABLE_GCOV)
		set(CMAKE_C_FLAGS "-coverage ${CMAKE_C_FLAGS}")
	endif()
elseif(CMAKE_C_COMPILER_ID STREQUAL "MSVC")
	# /wd4819
	# Without BOM, Visual Studio does not treat source file as UTF-8
	# encoding, thus it will complain about invalid character. Use
	# /wd4819 can suppress this warning.
	set(CMAKE_C_FLAGS "/wd4819 ${CMAKE_C_FLAGS}")
	add_definitions(/D_CRT_SECURE_NO_WARNINGS /D_CRT_NONSTDC_NO_DEPRECATE)
	add_definitions(/Dsnprintf=_snprintf /D__func__=__FUNCTION__)
endif()

include(CheckCCompilerFlag)
check_c_compiler_flag(-fvisibility=hidden FVISIBILITY_HIDDEN)
if(${FVISIBILITY_HIDDEN})
	set(CMAKE_C_FLAGS "-fvisibility=hidden ${CMAKE_C_FLAGS}")
endif()

# automake compatibility
add_definitions(-DHAVE_CONFIG_H=1)

# binary data representation
option(USE_BINARY_DATA "Use binary representation for internal data manipulation" true)
if (USE_BINARY_DATA)
	add_definitions(-DUSE_BINARY_DATA=1)
endif()

# Feature probe
include(CheckTypeSize)
check_type_size(uint16_t UINT16_T)

find_package(Curses)

include(CheckFunctionExists)
check_function_exists(strtok_r HAVE_STRTOK_R)
check_function_exists(asprintf HAVE_ASPRINTF)

include(CheckIncludeFiles)
check_include_files(unistd.h HAVE_UNISTD_H)

configure_file(
	${PROJECT_SOURCE_DIR}/include/cmake_config.h.in
	${PROJECT_BINARY_DIR}/config.h
)

configure_file(
	${PROJECT_SOURCE_DIR}/cmake_chewing.pc.in
	${PROJECT_BINARY_DIR}/chewing.pc
	@ONLY
)

include_directories(
	${PROJECT_BINARY_DIR}
	${PROJECT_SOURCE_DIR}/include
	${PROJECT_SOURCE_DIR}/include/internal
	${PROJECT_SOURCE_DIR}/src/porting_layer/include
	${PROJECT_SOURCE_DIR}/data/chewing
)

set(SRC_DIR ${PROJECT_SOURCE_DIR}/src)
set(INC_DIR ${PROJECT_SOURCE_DIR}/include)
set(TOOLS_SRC_DIR ${PROJECT_SOURCE_DIR}/src/tools)
set(TOOLS_BIN_DIR ${PROJECT_BINARY_DIR}/src/tools)
set(DATA_SRC_DIR ${PROJECT_SOURCE_DIR}/data)
set(DATA_BIN_DIR ${PROJECT_BINARY_DIR}/data)
set(TEST_SRC_DIR ${PROJECT_SOURCE_DIR}/test)
set(TEST_BIN_DIR ${PROJECT_BINARY_DIR}/test)

set(INSTALL_INC_DIR ${CMAKE_INSTALL_PREFIX}/include/chewing)
set(INSTALL_LIB_DIR ${CMAKE_INSTALL_PREFIX}/lib)
set(INSTALL_DATA_DIR ${CMAKE_INSTALL_PREFIX}/lib/libchewing)
set(INSTALL_INFO_DIR ${CMAKE_INSTALL_PREFIX}/share/info)

set(ALL_STATIC_DATA
	${DATA_BIN_DIR}/pinyin.tab
	${DATA_BIN_DIR}/swkb.dat
	${DATA_BIN_DIR}/symbols.dat
)

set(ALL_DATA
	${DATA_BIN_DIR}/dict.dat
	${DATA_BIN_DIR}/fonetree.dat
	${DATA_BIN_DIR}/ph_index.dat
	${DATA_BIN_DIR}/us_freq.dat
)

if (USE_BINARY_DATA)
	list(APPEND ALL_DATA
		${DATA_BIN_DIR}/ch_index_begin.dat
		${DATA_BIN_DIR}/ch_index_phone.dat
	)
else()
	list(APPEND ALL_DATA
		${DATA_BIN_DIR}/ch_index.dat
	)
endif()


set(ALL_INC
	${INC_DIR}/chewing.h
	${INC_DIR}/chewingio.h
	${INC_DIR}/global.h
	${INC_DIR}/mod_aux.h
)

# out of source
if(${OUT_OF_SOURCE})
	set(TABLE pinyin.tab swkb.dat symbols.dat)
	foreach(target ${TABLE})
		add_custom_command(
			OUTPUT
				${DATA_BIN_DIR}/${target}
			COMMAND ${CMAKE_COMMAND} -E make_directory ${DATA_BIN_DIR}
			COMMAND ${CMAKE_COMMAND} -E copy ${DATA_SRC_DIR}/${target} ${DATA_BIN_DIR}/${target}
			DEPENDS
				${DATA_SRC_DIR}/${target}
		)
	endforeach()
endif()

# tools
set(ALL_TOOLS sort maketree)
add_executable(sort ${TOOLS_SRC_DIR}/sort.c)
add_executable(maketree ${TOOLS_SRC_DIR}/maketree.c)
set_target_properties(${ALL_TOOLS} PROPERTIES
		RUNTIME_OUTPUT_DIRECTORY ${TOOLS_BIN_DIR}
		RUNTIME_OUTPUT_DIRECTORY_DEBUG ${TOOLS_BIN_DIR}
		RUNTIME_OUTPUT_DIRECTORY_RELEASE ${TOOLS_BIN_DIR}
)
foreach(target ${ALL_TOOLS})
	target_link_libraries(${target} common)
endforeach()

# tools command
add_custom_command(
	OUTPUT
		${ALL_DATA}
		${PROJECT_BINARY_DIR}/chewing-definition.h
	COMMAND ${CMAKE_COMMAND} -E make_directory ${DATA_BIN_DIR}
	COMMAND ${CMAKE_COMMAND} -E chdir ${DATA_BIN_DIR} ${TOOLS_BIN_DIR}/sort ${DATA_SRC_DIR}/phone.cin ${DATA_SRC_DIR}/tsi.src
	COMMAND ${CMAKE_COMMAND} -E chdir ${DATA_BIN_DIR} ${TOOLS_BIN_DIR}/maketree ${DATA_BIN_DIR}/phoneid.dic
	COMMAND ${CMAKE_COMMAND} -E copy ${DATA_BIN_DIR}/chewing-definition.h ${PROJECT_BINARY_DIR}/chewing-definition.h
	COMMAND ${CMAKE_COMMAND} -E remove -f ${DATA_BIN_DIR}/chewing-definition.h ${DATA_BIN_DIR}/phoneid.dic
	DEPENDS
		${ALL_TOOLS}
		${DATA_SRC_DIR}/phone.cin
		${DATA_SRC_DIR}/tsi.src
)

# test
set(ALL_TESTCASES
	test-bopomofo
	test-config
	test-easy-symbol
	test-fullshape
	test-key2pho
	test-keyboard
	test-mmap
	test-path
	test-regression
	test-reset
	test-special-symbol
	test-symbol
	test-utf8
)
set(ALL_TESTTOOLS
	randkeystroke
	simulate
	testchewing
)
# FIXME
#	if(${CURSES_FOUND})
#		set(ALL_TESTTOOLS ${ALL_TESTTOOLS} gen_keystroke)
#	endif()

set(ALL_TESTS ${ALL_TESTCASES} ${ALL_TESTTOOLS})

enable_testing()
foreach(target ${ALL_TESTCASES})
	add_test(${target} ${TEST_BIN_DIR}/${target})
endforeach()

find_program(VALGRIND valgrind)
if(EXISTS ${VALGRIND})
	foreach(target ${ALL_TESTCASES})
		add_test("valgrind-${target}" ${VALGRIND} --error-exitcode=255 --leak-check=full ${TEST_BIN_DIR}/${target})
	endforeach()
endif()

foreach(target ${ALL_TESTS})
	add_executable(${target} ${TEST_SRC_DIR}/${target}.c)
endforeach()

add_library(testhelper STATIC ${TEST_SRC_DIR}/testhelper.c $<TARGET_OBJECTS:chewing_obj>)

set_target_properties(${ALL_TESTS} PROPERTIES
	RUNTIME_OUTPUT_DIRECTORY ${TEST_BIN_DIR}
	RUNTIME_OUTPUT_DIRECTORY_DEBUG ${TEST_BIN_DIR}
	RUNTIME_OUTPUT_DIRECTORY_RELEASE ${TEST_BIN_DIR}
	COMPILE_DEFINITIONS
		"CHEWING_DATA_PREFIX=\"${DATA_BIN_DIR}\";TEST_HASH_DIR=\"${TEST_BIN_DIR}\";TESTDATA=\"${TEST_SRC_DIR}/default-test.txt\""
)
foreach(target ${ALL_TESTS})
	target_link_libraries(${target} testhelper common)
endforeach()

# data
add_custom_target(data ALL DEPENDS ${ALL_DATA} ${ALL_STATIC_DATA})

# library
add_library(chewing_obj OBJECT
	${SRC_DIR}/chewingio.c
	${SRC_DIR}/pinyin.c
	${SRC_DIR}/tree.c
	${SRC_DIR}/userphrase.c
	${SRC_DIR}/zuin.c
	${SRC_DIR}/chewingutil.c
	${SRC_DIR}/choice.c
	${SRC_DIR}/char.c
	${SRC_DIR}/dict.c
	${SRC_DIR}/porting_layer/src/plat_mmap_posix.c
	${SRC_DIR}/porting_layer/src/plat_mmap_windows.c
	${SRC_DIR}/porting_layer/src/plat_path.c
	${SRC_DIR}/hash.c
	${SRC_DIR}/mod_aux.c
)
add_custom_target(chewing-definition DEPENDS ${PROJECT_BINARY_DIR}/chewing-definition.h)
add_dependencies(chewing_obj chewing-definition)
set_target_properties(chewing_obj PROPERTIES
	COMPILE_DEFINITIONS "LIBDIR=\"${INSTALL_DATA}\""
)

if (NOT "${MSVC}")
	add_library(chewing_shared SHARED $<TARGET_OBJECTS:chewing_obj>)
	list(APPEND LIBS chewing_shared)
endif()

add_library(chewing_static STATIC $<TARGET_OBJECTS:chewing_obj>)
list(APPEND LIBS chewing_static)

set_target_properties(${LIBS} PROPERTIES
	OUTPUT_NAME chewing
	SOVERSION 3
	VERSION 3.0.1
)
foreach(target ${LIBS})
	target_link_libraries(${target} common)
endforeach()

add_library(common STATIC
	${SRC_DIR}/common/key2pho.c
	${SRC_DIR}/common/chewing-utf8-util.c
)

# install
install(FILES ${ALL_DATA} DESTINATION ${INSTALL_DATA_DIR})
install(FILES ${ALL_INC} DESTINATION ${INSTALL_INC_DIR})
install(FILES ${PROJECT_BINARY_DIR}/chewing.pc
	DESTINATION ${INSTALL_LIB_DIR}/pkgconfig)
install(TARGETS ${LIBS} DESTINATION ${INSTALL_LIB_DIR})
