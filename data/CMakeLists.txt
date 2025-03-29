set(ALL_DATA
    ${DATA_BIN_DIR}/tsi.dat
    ${DATA_BIN_DIR}/word.dat
)

add_custom_target(data ALL DEPENDS ${ALL_DATA})

# We need to copy static data to binary tree when using out of tree build.
set(ALL_STATIC_DATA
    ${DATA_SRC_DIR}/swkb.dat
    ${DATA_SRC_DIR}/symbols.dat
)

add_custom_target(all_static_data
    COMMAND ${CMAKE_COMMAND} -E copy_if_different ${DATA_SRC_DIR}/swkb.dat ${DATA_BIN_DIR}/swkb.dat
    COMMAND ${CMAKE_COMMAND} -E copy_if_different ${DATA_SRC_DIR}/symbols.dat ${DATA_BIN_DIR}/symbols.dat
)

# tools
set(ALL_TOOLS init_database)

# tools command
file(MAKE_DIRECTORY ${DATA_BIN_DIR})
set(DATA_COPYRIGHT "Copyright \\(c\\) 2022 libchewing Core Team")
set(DATA_LICENSE "LGPL-2.1-or-later")
set(DATA_VERSION ${LIBCHEWING_VERSION})
add_custom_command(
    OUTPUT
        ${ALL_DATA}
    COMMAND chewing-cli init-database
        -c ${DATA_COPYRIGHT}
        -l ${DATA_LICENSE}
        -r ${DATA_VERSION}
        -t trie
        -n 內建詞庫
        ${DATA_SRC_DIR}/tsi.src tsi.dat
    COMMAND chewing-cli init-database
        -c ${DATA_COPYRIGHT}
        -l ${DATA_LICENSE}
        -r ${DATA_VERSION}
        -t trie
        -n 內建字庫
        ${DATA_SRC_DIR}/word.src word.dat
    COMMAND chewing-cli init-database
        -c ${DATA_COPYRIGHT}
        -l ${DATA_LICENSE}
        -r ${DATA_VERSION}
        -t trie
        -n 內嵌字庫
        ${DATA_SRC_DIR}/mini.src mini.dat
    DEPENDS
        chewing-cli
        ${DATA_SRC_DIR}/mini.src
        ${DATA_SRC_DIR}/word.src
        ${DATA_SRC_DIR}/tsi.src
    WORKING_DIRECTORY ${DATA_BIN_DIR}
)

install(FILES ${ALL_DATA} DESTINATION ${CMAKE_INSTALL_DATADIR}/libchewing)
install(FILES ${ALL_STATIC_DATA} DESTINATION ${CMAKE_INSTALL_DATADIR}/libchewing)

