cmake_minimum_required(VERSION 3.24.0)
project(libchewing LANGUAGES C)

set(CMAKE_PROJECT_VERSION 0.8.5)
set(LIBCHEWING_VERSION ${CMAKE_PROJECT_VERSION})
set(PACKAGE_VERSION ${CMAKE_PROJECT_VERSION})
set(LIBCHEWING_BINARY_VERSION 1.0.0)
set(CMAKE_MODULE_PATH ${PROJECT_SOURCE_DIR}/cmake)

include(CTest)
set(CTEST_PARALLEL_LEVEL 1)

if(UNIX)
    set(CMAKE_C_FLAGS "-g -O2 -Wall -fPIC ${CMAKE_C_FLAGS}")
    add_compile_definitions(UNDER_POSIX PIC)
endif()

if(MSVC)
    set(CMAKE_C_FLAGS "/utf-8 ${CMAKE_C_FLAGS}")
endif()

include(CheckCCompilerFlag)

option(BUILD_SHARED_LIBS "Build using shared libraries" ON)

if(CMAKE_C_COMPILER_ID MATCHES GNU|Clang)
    set(CMAKE_C_FLAGS "-std=gnu99 ${CMAKE_C_FLAGS}")
    add_compile_definitions(_GNU_SOURCE)
    option(ENABLE_GCOV "Coverage support" false)
    if(ENABLE_GCOV)
        if(WITH_RUST AND CMAKE_C_COMPILER_ID MATCHES Clang)
            set(CMAKE_C_FLAGS "-fprofile-instr-generate -fcoverage-mapping ${CMAKE_C_FLAGS}")
        else()
            set(CMAKE_C_FLAGS "--coverage ${CMAKE_C_FLAGS}")
        endif()
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
elseif(MSVC)
    # /wd4819
    # Without BOM, Visual Studio does not treat source file as UTF-8
    # encoding, thus it will complain about invalid character. Use
    # /wd4819 can suppress this warning.
    set(CMAKE_C_FLAGS "/wd4819 ${CMAKE_C_FLAGS}")
    add_compile_definitions(_CRT_SECURE_NO_WARNINGS _CRT_NONSTDC_NO_DEPRECATE)
    add_compile_definitions(__func__=__FUNCTION__)

    # MSVC 2015 supports `snprintf`, so no need to redefine it
    if(MSVC_VERSION LESS 1900)
        add_compile_definitions(snprintf=_snprintf)
    endif()
endif()


check_c_compiler_flag(-fvisibility=hidden FVISIBILITY_HIDDEN)
if(${FVISIBILITY_HIDDEN})
    set(CMAKE_C_FLAGS "-fvisibility=hidden ${CMAKE_C_FLAGS}")
endif()

# automake compatibility
add_compile_definitions(HAVE_CONFIG_H=1)
add_custom_target(check COMMAND ${CMAKE_CTEST_COMMAND})

option(WITH_SQLITE3 "Use sqlite3 to store userphrase" true)
option(WITH_INTERNAL_SQLITE3 "Use internal sqlite3" false)
if(MSVC)
    set(WITH_INTERNAL_SQLITE3 true)
endif()

if(CMAKE_SYSTEM_NAME MATCHES "Android")
    set(WITH_INTERNAL_SQLITE3 true)
endif()

# Use valgrind when testing
option(USE_VALGRIND "Use valgrind when testing" true)

option(WITH_RUST "Use rust implemented internals" true)
if(WITH_RUST)
    find_package(Corrosion QUIET)
    if(NOT Corrosion_FOUND)
        include(FetchContent)
        FetchContent_Declare(
            Corrosion
            GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
            GIT_TAG 64289b1d79d6d19cd2e241db515381a086bb8407 # v0.5
            FIND_PACKAGE_ARGS
        )
        FetchContent_MakeAvailable(Corrosion)
    endif()

    corrosion_import_crate(MANIFEST_PATH Cargo.toml CRATES chewing_capi)
    corrosion_import_crate(MANIFEST_PATH Cargo.toml CRATES chewing_testhelper)
    corrosion_import_crate(MANIFEST_PATH Cargo.toml CRATES chewing-cli)
    add_compile_definitions(WITH_RUST)
    if(WITH_SQLITE3)
        corrosion_set_features(chewing_capi FEATURES sqlite)
        corrosion_set_features(chewing_testhelper FEATURES sqlite)
    endif()
    if(ENABLE_GCOV)
        corrosion_set_env_vars(chewing_capi CARGO_INCREMENTAL=0)
        corrosion_add_target_local_rustflags(chewing_capi -Cinstrument-coverage -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort)
    endif()
endif()

# Feature probe
include(CheckTypeSize)
check_type_size(uint16_t UINT16_T)

set(CURSES_NEED_WIDE true)
find_package(Curses)

if(WITH_SQLITE3)
    if(WITH_INTERNAL_SQLITE3)
        set(SQLITE3_SRC_DIR ${PROJECT_SOURCE_DIR}/thirdparty/sqlite-amalgamation)
        include_directories(
            ${SQLITE3_SRC_DIR}
        )
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
            RUNTIME_OUTPUT_DIRECTORY_RELWITHDEBINFO ${SQLITE3_SRC_DIR}
            RUNTIME_OUTPUT_DIRECTORY_MINSIZEREL ${SQLITE3_SRC_DIR}
        )
        set(SQLite3_LIBRARIES sqlite3_library)
    else()
        find_package(SQLite3 REQUIRED)
        include_directories(SQLite3_INCLUDE_DIRS)
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
set(TEST_SRC_DIR ${PROJECT_SOURCE_DIR}/tests)
set(TEST_BIN_DIR ${PROJECT_BINARY_DIR}/tests)

include(GNUInstallDirs)

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

set(ALL_INC
    ${INC_DIR}/chewing.h
    ${INC_DIR}/chewing-compat.h
    ${INC_DIR}/chewingio.h
    ${INC_DIR}/global.h
    ${INC_DIR}/mod_aux.h
)

if(WITH_RUST)
    list(APPEND ALL_INC ${INC_DIR}/chewing_rs.h)
endif()

add_subdirectory(doc)
add_subdirectory(data)
if(BUILD_TESTING)
    add_subdirectory(tests)
endif()

# library
add_library(common OBJECT
    src/porting_layer/src/asprintf.h

    src/porting_layer/src/asprintf.c
)
target_compile_definitions(common PRIVATE
    CHEWING_DATADIR=\"${CMAKE_INSTALL_FULL_DATADIR}/libchewing\"
)

add_library(libchewing ${ALL_INC} src/chewing.c)
set_target_properties(libchewing PROPERTIES LINKER_LANGUAGE C)
target_compile_definitions(libchewing PRIVATE
    CHEWING_DATADIR=\"${CMAKE_INSTALL_FULL_DATADIR}/libchewing\"
)
if(NOT WITH_RUST)
    target_sources(common PRIVATE
        ${SRC_DIR}/porting_layer/include/plat_mmap.h
        ${SRC_DIR}/porting_layer/include/plat_path.h
        ${SRC_DIR}/porting_layer/include/plat_types.h
        ${SRC_DIR}/porting_layer/include/sys/plat_posix.h
        ${SRC_DIR}/porting_layer/include/sys/plat_windows.h

        ${SRC_DIR}/porting_layer/src/plat_mmap_posix.c
        ${SRC_DIR}/porting_layer/src/plat_mmap_windows.c
        ${SRC_DIR}/porting_layer/src/plat_path.c
        ${SRC_DIR}/porting_layer/src/rpl_malloc.c
        ${SRC_DIR}/common/chewing-utf8-util.c
        ${SRC_DIR}/common/key2pho.c
    )
    target_sources(libchewing PRIVATE
        ${INC_DIR}/internal/bopomofo-private.h
        ${INC_DIR}/internal/chewing-private.h
        ${INC_DIR}/internal/chewingutil.h
        ${INC_DIR}/internal/choice-private.h
        ${INC_DIR}/internal/dict-private.h
        ${INC_DIR}/internal/global-private.h
        ${INC_DIR}/internal/pinyin-private.h
        ${INC_DIR}/internal/tree-private.h
        ${INC_DIR}/internal/userphrase-private.h

        ${SRC_DIR}/bopomofo.c
        ${SRC_DIR}/chewingio.c
        ${SRC_DIR}/chewingutil.c
        ${SRC_DIR}/choice.c
        ${SRC_DIR}/compat.c
        ${SRC_DIR}/dict.c
        ${SRC_DIR}/mod_aux.c
        ${SRC_DIR}/pinyin.c
        ${SRC_DIR}/private.h
        ${SRC_DIR}/tree.c
        ${SRC_DIR}/userphrase.c
    )
    if(WITH_SQLITE3)
        add_library(userphrase OBJECT
            ${INC_DIR}/internal/chewing-sql.h
            ${SRC_DIR}/chewing-sql.c
            ${SRC_DIR}/userphrase-sql.c
        )
    else()
        add_library(userphrase OBJECT
            ${INC_DIR}/internal/hash-private.h
            ${SRC_DIR}/hash.c
            ${SRC_DIR}/userphrase-hash.c
        )
    endif()
    target_link_libraries(libchewing
        PRIVATE common
        PRIVATE userphrase)
    if(BUILD_SHARED_LIBS AND MSVC)
        target_compile_definitions(libchewing PRIVATE CHEWINGDLL_EXPORTS)
    endif()
endif()

if(WITH_RUST)
    corrosion_set_env_vars(chewing_capi
        CHEWING_DATADIR=${CMAKE_INSTALL_FULL_DATADIR}/libchewing
    )
    target_link_libraries(libchewing PRIVATE chewing_capi)
    target_link_libraries(chewing_capi INTERFACE ${SQLite3_LIBRARIES})
    if(BUILD_SHARED_LIBS)
        if(CMAKE_C_COMPILER_ID MATCHES GNU|^Clang)
            target_link_options(libchewing
                PRIVATE LINKER:-version-script,${PROJECT_SOURCE_DIR}/src/symbols-elf.map
                PRIVATE LINKER:--gc-sections
                PRIVATE LINKER:-u,chewing_new
                PRIVATE LINKER:-u,ueStrLen
                PRIVATE LINKER:-u,UintFromPhone
                PRIVATE LINKER:-u,find_path_by_files
            )
        elseif(CMAKE_C_COMPILER_ID MATCHES AppleClang)
            target_link_options(libchewing
                PRIVATE LINKER:-exported_symbols_list,${PROJECT_SOURCE_DIR}/src/symbols-mach_o.map
                PRIVATE LINKER:-dead_strip
            )
        elseif(MSVC)
            target_link_options(libchewing
                PRIVATE /DEF:${PROJECT_SOURCE_DIR}/src/symbols-msvc.def)
            set_target_properties(libchewing PROPERTIES MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>DLL")
        endif()
    endif()
elseif(WITH_SQLITE3)
   target_link_libraries(libchewing PRIVATE ${SQLite3_LIBRARIES})
endif()

if(MSVC)
    set_target_properties(libchewing PROPERTIES
        OUTPUT_NAME chewing-msvc
    )
else()
    set_target_properties(libchewing PROPERTIES
        OUTPUT_NAME chewing
        SOVERSION 3
        VERSION 3.3.1
    )
endif()

install(FILES ${ALL_INC} DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}/chewing)
install(FILES ${PROJECT_BINARY_DIR}/chewing.pc
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/pkgconfig)
install(TARGETS libchewing DESTINATION ${CMAKE_INSTALL_LIBDIR})
if(WITH_RUST)
    install(IMPORTED_RUNTIME_ARTIFACTS chewing-cli DESTINATION ${CMAKE_INSTALL_BINDIR})
endif()


set(CPACK_PACKAGE_CHECKSUM SHA256)
set(CPACK_PACKAGE_VERSION ${CMAKE_PROJECT_VERSION})
set(CPACK_SOURCE_IGNORE_FILES "/build" "/out" "/target" "/\\\\..*")
set(CPACK_SOURCE_GENERATOR TZST)
set(CPACK_SOURCE_PACKAGE_FILE_NAME libchewing-${CMAKE_PROJECT_VERSION})
include(CPack)
