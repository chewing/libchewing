cmake_minimum_required(VERSION 3.24.0)
project(chewing-sample-cpp LANGUAGES C CXX)
set(CMAKE_CXX_STANDARD 23)
set(CHEWING_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../..")

file(GLOB APP_SOURCES_CPP *.c *.cpp)
file(GLOB APP_SOURCES_C *.c)
file(GLOB APP_HEADERS *.h)

add_executable(${PROJECT_NAME} ${APP_SOURCES_C} ${APP_SOURCES_CPP} ${APP_HEADERS})

target_include_directories(${PROJECT_NAME} PRIVATE
    ${CHEWING_PATH}/include
    ${CHEWING_PATH}/tests
)
target_compile_definitions(${PROJECT_NAME} PRIVATE
    CHEWING_INSTALL_DATA_PREFIX=\"${PROJECT_BINARY_DIR}/lib/libchewing/data\"
)
add_subdirectory(${CHEWING_PATH} ${CMAKE_BINARY_DIR}/lib/libchewing)

target_link_libraries(${PROJECT_NAME} PRIVATE
    libchewing
)
set_target_properties(${PROJECT_NAME} PROPERTIES LINKER_LANGUAGE CXX
)

