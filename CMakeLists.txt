cmake_minimum_required(VERSION 2.8.8)
project(libchewing)

# Update configure.ac if LIBCHEWING_VERSION is changed.
set(LIBCHEWING_VERSION 0.3.5)
set(CMAKE_MODULE_PATH ${PROJECT_SOURCE_DIR}/cmake)

if(UNIX)
	set(CMAKE_C_FLAGS "-g -O2 -Wall -fPIC ${CMAKE_C_FLAGS}")
	add_definitions(-DUNDER_POSIX -DPIC)
endif()

if(${CMAKE_C_COMPILER_ID} STREQUAL "GNU" OR
	${CMAKE_C_COMPILER_ID} STREQUAL "Clang")
	set(CMAKE_C_FLAGS "-std=gnu99 ${CMAKE_C_FLAGS}")
	add_definitions(-D_GNU_SOURCE)
	option(ENABLE_GCOV "Coverage support" false)
	if(ENABLE_GCOV)
		set(CMAKE_C_FLAGS "-coverage ${CMAKE_C_FLAGS}")
	endif()

	# Use NO_UNDEFINED=no when running with address sanitizer
	option(NO_UNDEFINED "No undefined symbol in object file" true)
	if(NO_UNDEFINED)
		set(CMAKE_SHARED_LINKER_FLAGS "-Wl,--no-undefined ${CMAKE_SHARED_LINKER_FLAGS}")
	endif()
elseif(CMAKE_C_COMPILER_ID MATCHES "MSVC")
	# XXX: Not sure why we need to use MATCHES instead of STREQUAL here.

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

# Use valgrind when testing
option(USE_VALGRIND "Use valgrind when testing" true)

# Feature probe
include(CheckTypeSize)
check_type_size(uint16_t UINT16_T)

set(CURSES_NEED_WIDE true)
find_package(Curses)

include(CheckFunctionExists)
check_function_exists(strtok_r HAVE_STRTOK_R)
check_function_exists(asprintf HAVE_ASPRINTF)

include(CheckIncludeFiles)
check_include_files(unistd.h HAVE_UNISTD_H)
check_include_files(stdint.h HAVE_STDINT_H)

include(TestBigEndian)
test_big_endian(WORDS_BIGENDIAN)

configure_file(
	${PROJECT_SOURCE_DIR}/cmake/config.h.in
	${PROJECT_BINARY_DIR}/include/config.h
)

configure_file(
	${PROJECT_SOURCE_DIR}/cmake/chewing.pc.in
	${PROJECT_BINARY_DIR}/chewing.pc
	@ONLY
)

configure_file(
	${PROJECT_SOURCE_DIR}/cmake/version.texi.in
	${PROJECT_BINARY_DIR}/doc/version.texi
)

include_directories(
	${PROJECT_BINARY_DIR}/include
	${PROJECT_SOURCE_DIR}/include
	${PROJECT_SOURCE_DIR}/include/internal
	${PROJECT_SOURCE_DIR}/src
	${PROJECT_SOURCE_DIR}/src/porting_layer/include
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

set(INFO_SRC ${PROJECT_SOURCE_DIR}/doc/libchewing.texi)
set(INFO_BIN ${PROJECT_BINARY_DIR}/doc/libchewing.info)

set(ALL_DATA
	${DATA_BIN_DIR}/dictionary.dat
	${DATA_BIN_DIR}/index_tree.dat
)

set(ALL_INC
	${INC_DIR}/chewing.h
	${INC_DIR}/chewingio.h
	${INC_DIR}/global.h
	${INC_DIR}/mod_aux.h
)

# info page
find_program(MAKEINFO makeinfo)
if (MAKEINFO)
	add_custom_command(
		OUTPUT
			${INFO_BIN}
		COMMAND ${MAKEINFO} ${INFO_SRC} -o ${INFO_BIN} -I ${PROJECT_BINARY_DIR}/doc
		DEPENDS
			${INFO_SRC}
	)
	add_custom_target(INFO ALL DEPENDS ${INFO_BIN})

	find_program(INSTALL_INFO NAMES ginstall-info install-info)
	if (INSTALL_INFO)
		install(FILES ${INFO_BIN} DESTINATION ${INSTALL_INFO_DIR})
		install(CODE "execute_process(COMMAND ${INSTALL_INFO} --info-dir=${INSTALL_INFO_DIR} ${INFO_BIN})")
	endif()
endif()


# We need to copy static data to binary tree when using out of tree build.
set(ALL_STATIC_DATA
	${DATA_BIN_DIR}/pinyin.tab
	${DATA_BIN_DIR}/swkb.dat
	${DATA_BIN_DIR}/symbols.dat
)

add_custom_target(all_static_data
	COMMAND	${CMAKE_COMMAND} -E copy_if_different ${DATA_SRC_DIR}/pinyin.tab ${DATA_BIN_DIR}/pinyin.tab
	COMMAND	${CMAKE_COMMAND} -E copy_if_different ${DATA_SRC_DIR}/swkb.dat ${DATA_BIN_DIR}/swkb.dat
	COMMAND	${CMAKE_COMMAND} -E copy_if_different ${DATA_SRC_DIR}/symbols.dat ${DATA_BIN_DIR}/symbols.dat
)

set(ALL_STATIC_TEST stresstest.py)
foreach(target ${ALL_STATIC_TEST})
	add_custom_target(${target} ALL
		COMMAND ${CMAKE_COMMAND} -E copy_if_different ${TEST_SRC_DIR}/${target} ${TEST_BIN_DIR}/${target}
	)
endforeach()

# tools
set(ALL_TOOLS init_database)
add_executable(init_database ${TOOLS_SRC_DIR}/init_database.c $<TARGET_OBJECTS:common>)
set_target_properties(${ALL_TOOLS} PROPERTIES
		RUNTIME_OUTPUT_DIRECTORY ${TOOLS_BIN_DIR}
		RUNTIME_OUTPUT_DIRECTORY_DEBUG ${TOOLS_BIN_DIR}
		RUNTIME_OUTPUT_DIRECTORY_MINSIZEREL ${TOOLS_BIN_DIR}
		RUNTIME_OUTPUT_DIRECTORY_RELEASE ${TOOLS_BIN_DIR}
		RUNTIME_OUTPUT_DIRECTORY_RELWITHDEBINFO ${TOOLS_BIN_DIR}
)

# tools command
add_custom_command(
	OUTPUT
		${ALL_DATA}
	COMMAND ${CMAKE_COMMAND} -E make_directory ${DATA_BIN_DIR}
	COMMAND ${CMAKE_COMMAND} -E chdir ${DATA_BIN_DIR} ${TOOLS_BIN_DIR}/init_database ${DATA_SRC_DIR}/phone.cin ${DATA_SRC_DIR}/tsi.src
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
	test-logger
	test-mmap
	test-path
	test-regression
	test-reset
	test-special-symbol
	test-symbol
	test-userphrase
	test-utf8
)
set(ALL_TESTTOOLS
	performance
	randkeystroke
	simulate
	testchewing
)

if(${CURSES_FOUND})
	set(ALL_TESTTOOLS ${ALL_TESTTOOLS} genkeystroke)
endif()

enable_testing()

set(ALL_TESTS ${ALL_TESTCASES} ${ALL_TESTTOOLS})

foreach(target ${ALL_TESTCASES})
	add_test(${target} ${TEST_BIN_DIR}/${target})
endforeach()

if(USE_VALGRIND)
	find_program(VALGRIND valgrind)
	if(VALGRIND)
		foreach(target ${ALL_TESTCASES})
			add_test("valgrind-${target}" ${VALGRIND} --error-exitcode=255 --leak-check=full ${TEST_BIN_DIR}/${target})
		endforeach()
	endif()
endif()

add_custom_target(check COMMAND ${CMAKE_CTEST_COMMAND})
foreach(target ${ALL_TESTS})
	add_executable(${target} ${TEST_SRC_DIR}/${target}.c)
	add_dependencies(${target} data all_static_data)
	add_dependencies(check ${target})
endforeach()

add_library(testhelper STATIC
	${TEST_SRC_DIR}/testhelper.c
	$<TARGET_OBJECTS:chewing>
	$<TARGET_OBJECTS:common>
)
set_target_properties(testhelper PROPERTIES
	COMPILE_DEFINITIONS
		"CHEWING_DATA_PREFIX=\"${DATA_BIN_DIR}\";TEST_HASH_DIR=\"${TEST_BIN_DIR}\";TESTDATA=\"${TEST_SRC_DIR}/default-test.txt\""
)

set_target_properties(${ALL_TESTS} PROPERTIES
	RUNTIME_OUTPUT_DIRECTORY ${TEST_BIN_DIR}
	RUNTIME_OUTPUT_DIRECTORY_DEBUG ${TEST_BIN_DIR}
	RUNTIME_OUTPUT_DIRECTORY_RELEASE ${TEST_BIN_DIR}
	COMPILE_DEFINITIONS
		"CHEWING_DATA_PREFIX=\"${DATA_BIN_DIR}\";TEST_HASH_DIR=\"${TEST_BIN_DIR}\";TESTDATA=\"${TEST_SRC_DIR}/default-test.txt\""
)
foreach(target ${ALL_TESTS})
	target_link_libraries(${target} testhelper)
endforeach()

if (${CURSES_FOUND})
	target_link_libraries(genkeystroke ${CURSES_LIBRARIES})
endif()

# data
add_custom_target(data ALL DEPENDS ${ALL_DATA})

# library
add_library(chewing OBJECT
	${ALL_INC}
	${INC_DIR}/internal/char-private.h
	${INC_DIR}/internal/chewing-private.h
	${INC_DIR}/internal/chewingutil.h
	${INC_DIR}/internal/choice-private.h
	${INC_DIR}/internal/dict-private.h
	${INC_DIR}/internal/global-private.h
	${INC_DIR}/internal/hash-private.h
	${INC_DIR}/internal/pinyin-private.h
	${INC_DIR}/internal/tree-private.h
	${INC_DIR}/internal/userphrase-private.h
	${INC_DIR}/internal/zuin-private.h

	${SRC_DIR}/char.c
	${SRC_DIR}/chewingio.c
	${SRC_DIR}/chewingutil.c
	${SRC_DIR}/choice.c
	${SRC_DIR}/dict.c
	${SRC_DIR}/hash.c
	${SRC_DIR}/mod_aux.c
	${SRC_DIR}/pinyin.c
	${SRC_DIR}/porting_layer/include/plat_mmap.h
	${SRC_DIR}/porting_layer/include/plat_path.h
	${SRC_DIR}/porting_layer/include/plat_types.h
	${SRC_DIR}/porting_layer/include/sys/plat_posix.h
	${SRC_DIR}/porting_layer/include/sys/plat_windows.h
	${SRC_DIR}/porting_layer/src/plat_mmap_posix.c
	${SRC_DIR}/porting_layer/src/plat_mmap_windows.c
	${SRC_DIR}/porting_layer/src/plat_path.c
	${SRC_DIR}/private.h
	${SRC_DIR}/tree.c
	${SRC_DIR}/userphrase.c
	${SRC_DIR}/zuin.c
)
set_target_properties(chewing PROPERTIES
	COMPILE_DEFINITIONS "LIBDIR=\"${INSTALL_DATA}\""
)

if (NOT MSVC)
	add_library(chewing_shared SHARED
		$<TARGET_OBJECTS:chewing>
		$<TARGET_OBJECTS:common>
	)
	list(APPEND LIBS chewing_shared)
endif()

add_library(chewing_static STATIC
	$<TARGET_OBJECTS:chewing>
	$<TARGET_OBJECTS:common>
)
list(APPEND LIBS chewing_static)

set_target_properties(${LIBS} PROPERTIES
	OUTPUT_NAME chewing
	# Update configure.ac if one of SOVERSION/VERSION is updated.
	# See configure.ac for more information.
	SOVERSION 3
	VERSION 3.1.0
)

add_library(common OBJECT
	${INC_DIR}/internal/chewing-utf8-util.h
	${INC_DIR}/internal/key2pho-private.h
	${INC_DIR}/internal/memory-private.h

	${SRC_DIR}/common/chewing-utf8-util.c
	${SRC_DIR}/common/key2pho.c
)

# install
install(FILES ${ALL_DATA} DESTINATION ${INSTALL_DATA_DIR})
install(FILES ${ALL_STATIC_DATA} DESTINATION ${INSTALL_DATA_DIR})
install(FILES ${ALL_INC} DESTINATION ${INSTALL_INC_DIR})
install(FILES ${PROJECT_BINARY_DIR}/chewing.pc
	DESTINATION ${INSTALL_LIB_DIR}/pkgconfig)
install(TARGETS ${LIBS} DESTINATION ${INSTALL_LIB_DIR})
