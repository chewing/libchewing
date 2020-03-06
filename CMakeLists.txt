cmake_minimum_required(VERSION 2.8.8)
project(libchewing)

# Update configure.ac if LIBCHEWING_VERSION is changed.
set(LIBCHEWING_VERSION 0.5.1)
set(PACKAGE_VERSION ${LIBCHEWING_VERSION})
# Update configure.ac if LIBCHEWING_BINARY_VERSION is changed.
set(LIBCHEWING_BINARY_VERSION 1.0.0)
set(CMAKE_MODULE_PATH ${PROJECT_SOURCE_DIR}/cmake)

if(UNIX)
    set(CMAKE_C_FLAGS "-g -O2 -Wall -fPIC ${CMAKE_C_FLAGS}")
    add_definitions(-DUNDER_POSIX -DPIC)
endif()

include(CheckCCompilerFlag)

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
        set(saved_CMAKE_REQUIRED_FLAGS ${CMAKE_REQUIRED_FLAGS})
        set(CMAKE_REQUIRED_FLAGS "-Wl,--no-undefined")
        check_c_compiler_flag("" HAVE_NO_UNDEFINED)
        set(CMAKE_REQUIRED_FLAGS ${saved_CMAKE_REQUIRED_FLAGS})

        if(HAVE_NO_UNDEFINED)
            set(CMAKE_SHARED_LINKER_FLAGS "-Wl,--no-undefined ${CMAKE_SHARED_LINKER_FLAGS}")
        endif()
    endif()
elseif(CMAKE_C_COMPILER_ID MATCHES "MSVC")
    # XXX: Not sure why we need to use MATCHES instead of STREQUAL here.

    # /wd4819
    # Without BOM, Visual Studio does not treat source file as UTF-8
    # encoding, thus it will complain about invalid character. Use
    # /wd4819 can suppress this warning.
    set(CMAKE_C_FLAGS "/wd4819 ${CMAKE_C_FLAGS}")
    add_definitions(/D_CRT_SECURE_NO_WARNINGS /D_CRT_NONSTDC_NO_DEPRECATE)
    add_definitions(/D__func__=__FUNCTION__)

    # MSVC 2015 supports `snprintf`, so no need to redefine it
    if(MSVC_VERSION LESS 1900)
        add_definitions(/Dsnprintf=_snprintf)
    endif()

    option(BUILD_DLL "Build dynamic link library (*.dll) instead of static lib." false)
endif()

check_c_compiler_flag(-fvisibility=hidden FVISIBILITY_HIDDEN)
if(${FVISIBILITY_HIDDEN})
    set(CMAKE_C_FLAGS "-fvisibility=hidden ${CMAKE_C_FLAGS}")
endif()

# automake compatibility
add_definitions(-DHAVE_CONFIG_H=1)
add_custom_target(check COMMAND ${CMAKE_CTEST_COMMAND})

option(WITH_SQLITE3 "Use sqlite3 to store userphrase" true)
option(WITH_INTERNAL_SQLITE3 "Use internal sqlite3" false)
if(MSVC)
    set(WITH_INTERNAL_SQLITE3 true)
endif()

# Use valgrind when testing
option(USE_VALGRIND "Use valgrind when testing" true)

# Feature probe
include(CheckTypeSize)
check_type_size(uint16_t UINT16_T)

set(CURSES_NEED_WIDE true)
find_package(Curses)

if (WITH_SQLITE3)
    if (WITH_INTERNAL_SQLITE3)
        set(SQLITE3_SRC_DIR ${PROJECT_SOURCE_DIR}/thirdparty/sqlite-amalgamation)
        include_directories(
            ${SQLITE3_SRC_DIR}
        )
    else()
        find_library(SQLITE3_LIBRARY sqlite3)
        find_file(SQLITE3_HEADER sqlite3.h)
        get_filename_component(SQLITE3_INCLUDE ${SQLITE3_HEADER} PATH)

        if (NOT (SQLITE3_LIBRARY AND SQLITE3_INCLUDE))
            message(SEND_ERROR "Cannot find sqlite3 library")
        endif()

        include_directories(SQLITE3_INCLUDE)
    endif()
endif()

include(CheckFunctionExists)
check_function_exists(strtok_r HAVE_STRTOK_R)
check_function_exists(asprintf HAVE_ASPRINTF)

include(CheckIncludeFiles)
check_include_files(unistd.h HAVE_UNISTD_H)
check_include_files(stdint.h HAVE_STDINT_H)

include(TestBigEndian)
test_big_endian(WORDS_BIGENDIAN)

set(SRC_DIR ${PROJECT_SOURCE_DIR}/src)
set(INC_DIR ${PROJECT_SOURCE_DIR}/include)
set(TOOLS_SRC_DIR ${PROJECT_SOURCE_DIR}/src/tools)
set(TOOLS_BIN_DIR ${PROJECT_BINARY_DIR}/src/tools)
set(DATA_SRC_DIR ${PROJECT_SOURCE_DIR}/data)
set(DATA_BIN_DIR ${PROJECT_BINARY_DIR}/data)
set(TEST_SRC_DIR ${PROJECT_SOURCE_DIR}/test)
set(TEST_BIN_DIR ${PROJECT_BINARY_DIR}/test)

include(GNUInstallDirs)

set(INFO_SRC ${PROJECT_SOURCE_DIR}/doc/libchewing.texi)
set(INFO_BIN ${PROJECT_BINARY_DIR}/doc/libchewing.info)

configure_file(
    ${PROJECT_SOURCE_DIR}/cmake/config.h.in
    ${PROJECT_BINARY_DIR}/include/config.h
)

configure_file(
    ${PROJECT_SOURCE_DIR}/cmake/version.texi.in
    ${PROJECT_BINARY_DIR}/doc/version.texi
)

set(prefix "${CMAKE_INSTALL_PREFIX}")
set(exec_prefix "\${prefix}")
set(libdir "\${exec_prefix}/lib")
set(includedir "\${prefix}/include")
set(datarootdir "\${prefix}/share")
set(datadir "\${datarootdir}")
set(sysconfdir "\${prefix}/etc")
configure_file(
    ${PROJECT_SOURCE_DIR}/chewing.pc.in
    ${PROJECT_BINARY_DIR}/chewing.pc
    @ONLY
)

include_directories(
    ${PROJECT_BINARY_DIR}/include
    ${PROJECT_SOURCE_DIR}/include
    ${PROJECT_SOURCE_DIR}/include/internal
    ${PROJECT_SOURCE_DIR}/src
    ${PROJECT_SOURCE_DIR}/src/porting_layer/include
)

set(ALL_DATA
    ${DATA_BIN_DIR}/dictionary.dat
    ${DATA_BIN_DIR}/index_tree.dat
)

set(ALL_INC
    ${INC_DIR}/chewing.h
    ${INC_DIR}/chewing-compat.h
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
    add_dependencies(check INFO)

    find_program(INSTALL_INFO NAMES ginstall-info install-info)
    if (INSTALL_INFO)
        install(FILES ${INFO_BIN} DESTINATION ${CMAKE_INSTALL_INFODIR})
        install(CODE "execute_process(COMMAND ${INSTALL_INFO} --info-dir=${CMAKE_INSTALL_INFODIR} ${INFO_BIN})")
    endif()
endif()


# We need to copy static data to binary tree when using out of tree build.
set(ALL_STATIC_DATA
    ${DATA_BIN_DIR}/pinyin.tab
    ${DATA_BIN_DIR}/swkb.dat
    ${DATA_BIN_DIR}/symbols.dat
)

add_custom_target(all_static_data
    COMMAND ${CMAKE_COMMAND} -E copy_if_different ${DATA_SRC_DIR}/pinyin.tab ${DATA_BIN_DIR}/pinyin.tab
    COMMAND ${CMAKE_COMMAND} -E copy_if_different ${DATA_SRC_DIR}/swkb.dat ${DATA_BIN_DIR}/swkb.dat
    COMMAND ${CMAKE_COMMAND} -E copy_if_different ${DATA_SRC_DIR}/symbols.dat ${DATA_BIN_DIR}/symbols.dat
)

set(ALL_STATIC_TEST stresstest.py)
foreach(target ${ALL_STATIC_TEST})
    add_custom_target(${target} ALL
        COMMAND ${CMAKE_COMMAND} -E copy_if_different ${TEST_SRC_DIR}/${target} ${TEST_BIN_DIR}/${target}
    )
    add_dependencies(check ${target})
endforeach()

# tools
set(ALL_TOOLS init_database dump_database)
add_executable(init_database ${TOOLS_SRC_DIR}/init_database.c $<TARGET_OBJECTS:common>)
add_executable(dump_database
    ${TOOLS_SRC_DIR}/dump_database.c
    ${SRC_DIR}/porting_layer/src/plat_mmap_posix.c
    ${SRC_DIR}/porting_layer/src/plat_mmap_windows.c
    ${SRC_DIR}/porting_layer/src/rpl_malloc.c
    $<TARGET_OBJECTS:common>
)
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
    test-error-handling
    test-fullshape
    test-key2pho
    test-keyboard
    test-keyboardless
    test-logger
    test-mmap
    test-path
    test-regression
    test-reset
    test-special-symbol
    test-struct-size
    test-symbol
    test-userphrase
    test-utf8
)
set(ALL_TESTTOOLS
    performance
    randkeystroke
    simulate
    stress
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
target_link_libraries(testhelper userphrase)
set_target_properties(testhelper PROPERTIES
    COMPILE_DEFINITIONS
        "CHEWING_DATA_PREFIX=\"${DATA_BIN_DIR}\";TEST_HASH_DIR=\"${TEST_BIN_DIR}\";TEST_DATA_DIR=\"${TEST_SRC_DIR}/data\";TESTDATA=\"${TEST_SRC_DIR}/default-test.txt\""
)

set_target_properties(${ALL_TESTS} PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY ${TEST_BIN_DIR}
    RUNTIME_OUTPUT_DIRECTORY_DEBUG ${TEST_BIN_DIR}
    RUNTIME_OUTPUT_DIRECTORY_RELEASE ${TEST_BIN_DIR}
    RUNTIME_OUTPUT_DIRECTORY_RELEASE ${TEST_BIN_DIR}
    RUNTIME_OUTPUT_DIRECTORY_RELWITHDEBINFO ${TEST_BIN_DIR}
    COMPILE_DEFINITIONS
        "CHEWING_DATA_PREFIX=\"${DATA_BIN_DIR}\";TEST_HASH_DIR=\"${TEST_BIN_DIR}\";TEST_DATA_DIR=\"${TEST_SRC_DIR}/data\""
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
    ${INC_DIR}/internal/chewing-private.h
    ${INC_DIR}/internal/chewingutil.h
    ${INC_DIR}/internal/choice-private.h
    ${INC_DIR}/internal/dict-private.h
    ${INC_DIR}/internal/global-private.h
    ${INC_DIR}/internal/pinyin-private.h
    ${INC_DIR}/internal/tree-private.h
    ${INC_DIR}/internal/userphrase-private.h
    ${INC_DIR}/internal/bopomofo-private.h

    ${SRC_DIR}/compat.c
    ${SRC_DIR}/chewingio.c
    ${SRC_DIR}/chewingutil.c
    ${SRC_DIR}/choice.c
    ${SRC_DIR}/dict.c
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
    ${SRC_DIR}/porting_layer/src/rpl_malloc.c
    ${SRC_DIR}/private.h
    ${SRC_DIR}/tree.c
    ${SRC_DIR}/userphrase.c
    ${SRC_DIR}/bopomofo.c
)
set_target_properties(chewing PROPERTIES
    COMPILE_DEFINITIONS "CHEWING_DATADIR=\"${CMAKE_INSTALL_FULL_DATADIR}/libchewing\""
)

if (WITH_SQLITE3)
    add_library(userphrase STATIC
        ${INC_DIR}/internal/chewing-sql.h

        ${SRC_DIR}/chewing-sql.c
        ${SRC_DIR}/userphrase-sql.c
    )

    if (WITH_INTERNAL_SQLITE3)
        find_package (Threads)
        add_library(sqlite3_library STATIC
            ${SQLITE3_SRC_DIR}/sqlite3.c
            ${SQLITE3_SRC_DIR}/sqlite3.h
        )
        target_link_libraries(sqlite3_library ${CMAKE_DL_LIBS} ${CMAKE_THREAD_LIBS_INIT})

        add_executable(sqlite3
            ${SQLITE3_SRC_DIR}/shell.c
        )
        target_link_libraries(sqlite3 sqlite3_library)
        set_target_properties(sqlite3 PROPERTIES
            RUNTIME_OUTPUT_DIRECTORY ${SQLITE3_SRC_DIR}
            RUNTIME_OUTPUT_DIRECTORY_DEBUG ${SQLITE3_SRC_DIR}
            RUNTIME_OUTPUT_DIRECTORY_RELEASE ${SQLITE3_SRC_DIR}
            RUNTIME_OUTPUT_DIRECTORY_RELEASE ${SQLITE3_SRC_DIR}
            RUNTIME_OUTPUT_DIRECTORY_RELWITHDEBINFO ${SQLITE3_SRC_DIR}
        )
        target_link_libraries(userphrase sqlite3_library)
    else()
        target_link_libraries(testhelper ${SQLITE3_LIBRARY})
    endif()
else()
    add_library(userphrase STATIC
        ${INC_DIR}/internal/hash-private.h

        ${SRC_DIR}/hash.c
        ${SRC_DIR}/userphrase-hash.c
    )
endif()

if (BUILD_DLL OR NOT MSVC)
    if (MSVC)
        add_definitions(-DCHEWINGDLL_EXPORTS)
    endif()
    add_library(chewing_shared SHARED
        $<TARGET_OBJECTS:chewing>
        $<TARGET_OBJECTS:common>
    )
    list(APPEND LIBS chewing_shared)
    add_dependencies(check chewing_shared)
endif()

if (NOT BUILD_DLL)
    add_library(chewing_static STATIC
        $<TARGET_OBJECTS:chewing>
        $<TARGET_OBJECTS:common>
    )
    list(APPEND LIBS chewing_static)
    add_dependencies(check chewing_static)
endif()

foreach(lib ${LIBS})
    target_link_libraries(${lib} userphrase)
    if (WITH_SQLITE3 AND NOT WITH_INTERNAL_SQLITE3)
        target_link_libraries(${lib} ${SQLITE3_LIBRARY})
    endif()
endforeach()

set_target_properties(${LIBS} PROPERTIES
    OUTPUT_NAME chewing
    # Update configure.ac if one of SOVERSION/VERSION is updated.
    # See configure.ac for more information.
    SOVERSION 3
    VERSION 3.3.1
)

add_library(common OBJECT
    ${INC_DIR}/internal/chewing-utf8-util.h
    ${INC_DIR}/internal/key2pho-private.h
    ${INC_DIR}/internal/memory-private.h

    ${SRC_DIR}/common/chewing-utf8-util.c
    ${SRC_DIR}/common/key2pho.c
)


# install
install(FILES ${ALL_DATA} DESTINATION ${CMAKE_INSTALL_DATADIR}/libchewing)
install(FILES ${ALL_STATIC_DATA} DESTINATION ${CMAKE_INSTALL_DATADIR}/libchewing)
install(FILES ${ALL_INC} DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}/chewing)
install(FILES ${PROJECT_BINARY_DIR}/chewing.pc
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/pkgconfig)
install(TARGETS ${LIBS} DESTINATION ${CMAKE_INSTALL_LIBDIR})
