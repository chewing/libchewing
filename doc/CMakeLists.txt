set(INFO_SRC ${PROJECT_SOURCE_DIR}/doc/libchewing.texi)
set(INFO_BIN ${PROJECT_BINARY_DIR}/doc/libchewing.info)

option(BUILD_INFO "Build info doc from texi source")

find_program(MAKEINFO makeinfo)
if (BUILD_INFO AND MAKEINFO)
    add_custom_command(
        OUTPUT
            ${INFO_BIN}
        COMMAND ${MAKEINFO} ${INFO_SRC} -o ${INFO_BIN} -I ${PROJECT_BINARY_DIR}/doc
        DEPENDS
            ${INFO_SRC}
    )
    add_custom_target(INFO ALL DEPENDS ${INFO_BIN})
    add_dependencies(check INFO)

    install(FILES ${INFO_BIN} TYPE INFO)
endif()