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
foreach(target ${ALL_TESTS})
    target_link_libraries(${target} testhelper)
    target_compile_definitions(${target} PRIVATE
        CHEWING_DATA_PREFIX=\"${DATA_BIN_DIR}\"
        TEST_HASH_DIR=\"${TEST_BIN_DIR}\"
        TEST_DATA_DIR=\"${TEST_SRC_DIR}/data\"
        TESTDATA=\"${TEST_SRC_DIR}/default-test.txt\"
    )
endforeach()

if (${CURSES_FOUND})
    target_link_libraries(genkeystroke ${CURSES_LIBRARIES})
endif()

set(ALL_STATIC_TEST stresstest.py)
foreach(target ${ALL_STATIC_TEST})
    add_custom_target(${target} ALL
        COMMAND ${CMAKE_COMMAND} -E copy_if_different ${TEST_SRC_DIR}/${target} ${TEST_BIN_DIR}/${target}
    )
    add_dependencies(check ${target})
endforeach()