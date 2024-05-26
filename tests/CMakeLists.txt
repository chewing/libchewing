set(ALL_TESTCASES
    test-bopomofo
    test-config
    test-easy-symbol
    test-error-handling
    test-fullshape
    test-keyboard
    test-keyboardless
    test-logger
    test-regression
    test-reset
    test-special-symbol
    test-struct-size
    test-symbol
    test-userphrase
)
set(ALL_TESTTOOLS
    performance
    randkeystroke
    simulate
    stress
    testchewing
)

if(CURSES_FOUND)
    set(ALL_TESTTOOLS ${ALL_TESTTOOLS} genkeystroke)
endif()


set(ALL_TESTS ${ALL_TESTCASES} ${ALL_TESTTOOLS})

foreach(target ${ALL_TESTCASES})
    add_test(
        NAME ${target}
        COMMAND ${TEST_BIN_DIR}/${target}
        WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    )
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
    target_link_libraries(${target} testhelper)
    target_link_libraries(${target} chewing_testhelper)
    target_link_libraries(${target} libchewing)
    target_compile_definitions(${target} PRIVATE
        CHEWING_DATA_PREFIX=\"${DATA_BIN_DIR}\"
        TEST_HASH_DIR=\"${TEST_BIN_DIR}\"
        TEST_DATA_DIR=\"${TEST_SRC_DIR}/data\"
        TESTDATA=\"${TEST_SRC_DIR}/default-test.txt\"
    )
    add_dependencies(${target} data all_static_data)
    add_dependencies(check ${target})
endforeach()

add_library(testhelper STATIC
    ${TEST_SRC_DIR}/testhelper.c
    ${TEST_SRC_DIR}/porting_layer/src/asprintf.c
    ${TEST_SRC_DIR}/porting_layer/src/asprintf.h
    ${TEST_SRC_DIR}/porting_layer/include/plat_types.h
    ${TEST_SRC_DIR}/porting_layer/include/sys/plat_posix.h
    ${TEST_SRC_DIR}/porting_layer/include/sys/plat_windows.h
)
target_include_directories(testhelper PUBLIC
        ${TEST_SRC_DIR}/porting_layer/include
)
target_compile_definitions(testhelper PRIVATE
    CHEWING_DATA_PREFIX=\"${DATA_BIN_DIR}\"
    TEST_HASH_DIR=\"${TEST_BIN_DIR}\"
    TEST_DATA_DIR=\"${TEST_SRC_DIR}/data\"
    TESTDATA=\"${TEST_SRC_DIR}/default-test.txt\"
)

set_target_properties(${ALL_TESTS} PROPERTIES
    RUNTIME_OUTPUT_DIRECTORY ${TEST_BIN_DIR}
    RUNTIME_OUTPUT_DIRECTORY_DEBUG ${TEST_BIN_DIR}
    RUNTIME_OUTPUT_DIRECTORY_RELEASE ${TEST_BIN_DIR}
    RUNTIME_OUTPUT_DIRECTORY_RELEASE ${TEST_BIN_DIR}
    RUNTIME_OUTPUT_DIRECTORY_RELWITHDEBINFO ${TEST_BIN_DIR}
)

if (CURSES_FOUND)
    target_include_directories(genkeystroke PRIVATE ${CURSES_INCLUDE_DIRS})
    target_link_libraries(genkeystroke ${CURSES_LIBRARIES})
endif()

set(ALL_STATIC_TEST stresstest.py)
foreach(target ${ALL_STATIC_TEST})
    add_custom_target(${target} ALL
        COMMAND ${CMAKE_COMMAND} -E copy_if_different ${TEST_SRC_DIR}/${target} ${TEST_BIN_DIR}/${target}
    )
    add_dependencies(check ${target})
endforeach()

add_test(NAME cargo-test-lib COMMAND cargo test --features $<$<BOOL:${WITH_SQLITE3}>:sqlite>)
add_test(NAME cargo-test-capi COMMAND cargo test -p chewing_capi --features $<$<BOOL:${WITH_SQLITE3}>:sqlite>)
