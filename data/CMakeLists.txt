set(ALL_DATA
    ${DATA_BIN_DIR}/dictionary.dat
    ${DATA_BIN_DIR}/index_tree.dat
)

add_custom_target(data ALL DEPENDS ${ALL_DATA})

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

# tools
set(ALL_TOOLS init_database)
add_executable(init_database ${TOOLS_SRC_DIR}/init_database.c $<TARGET_OBJECTS:common>)
if (WITH_RUST)
target_link_libraries(init_database chewing-internal)
endif()

if (NOT WITH_RUST)
list(APPEND ALL_TOOLS dump_database)
add_executable(dump_database
    ${TOOLS_SRC_DIR}/dump_database.c
    $<TARGET_OBJECTS:common>
)
endif()
set_target_properties(${ALL_TOOLS} PROPERTIES
        RUNTIME_OUTPUT_DIRECTORY ${TOOLS_BIN_DIR}
        RUNTIME_OUTPUT_DIRECTORY_DEBUG ${TOOLS_BIN_DIR}
        RUNTIME_OUTPUT_DIRECTORY_MINSIZEREL ${TOOLS_BIN_DIR}
        RUNTIME_OUTPUT_DIRECTORY_RELEASE ${TOOLS_BIN_DIR}
        RUNTIME_OUTPUT_DIRECTORY_RELWITHDEBINFO ${TOOLS_BIN_DIR}
)

# tools command
file(MAKE_DIRECTORY ${DATA_BIN_DIR})
add_custom_command(
    OUTPUT
        ${ALL_DATA}
    COMMAND init_database ${DATA_SRC_DIR}/phone.cin ${DATA_SRC_DIR}/tsi.src
    DEPENDS
        ${ALL_TOOLS}
        ${DATA_SRC_DIR}/phone.cin
        ${DATA_SRC_DIR}/tsi.src
    WORKING_DIRECTORY ${DATA_BIN_DIR}
)

install(FILES ${ALL_DATA} DESTINATION ${CMAKE_INSTALL_DATADIR}/libchewing)
install(FILES ${ALL_STATIC_DATA} DESTINATION ${CMAKE_INSTALL_DATADIR}/libchewing)
