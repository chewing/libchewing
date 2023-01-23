set(INFO_SRC ${PROJECT_SOURCE_DIR}/doc/libchewing.texi)
set(INFO_BIN ${PROJECT_BINARY_DIR}/doc/libchewing.info)

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