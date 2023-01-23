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