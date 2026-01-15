cmake_minimum_required(VERSION 3.16)
project(listory_search VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Quick Concurrent)
find_package(Protobuf REQUIRED)

# Generate protobuf files
set(PROTO_FILES ${CMAKE_CURRENT_SOURCE_DIR}/proto/search.proto)
protobuf_generate_cpp(PROTO_SRCS PROTO_HDRS ${PROTO_FILES})

# Include directories
include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_CURRENT_BINARY_DIR}
    ${Protobuf_INCLUDE_DIRS}
)

# Source files
set(SOURCES
    src/main.cpp
    src/search_backend.cpp
    src/ipc/pipe_client.cpp
    ${PROTO_SRCS}
)

set(HEADERS
    src/search_backend.h
    src/ipc/pipe_client.h
    src/ipc/ipc_codec.h
    src/model/search_result.h
    ${PROTO_HDRS}
)

# QML resources
qt_add_executable(listory_search
    ${SOURCES}
    ${HEADERS}
)

qt_add_qml_module(listory_search
    URI ListorySearch
    VERSION 1.0
    QML_FILES
        qml/main.qml
)

target_link_libraries(listory_search PRIVATE
    Qt6::Core
    Qt6::Gui
    Qt6::Quick
    Qt6::Concurrent
    ${Protobuf_LIBRARIES}
)

# Windows specific settings
if(WIN32)
    target_compile_definitions(listory_search PRIVATE UNICODE _UNICODE)
    set_target_properties(listory_search PROPERTIES
        WIN32_EXECUTABLE TRUE
    )
endif()
