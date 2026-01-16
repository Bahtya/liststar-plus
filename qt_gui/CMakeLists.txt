cmake_minimum_required(VERSION 3.16)

# Set vcpkg toolchain if not already set
if(NOT DEFINED CMAKE_TOOLCHAIN_FILE)
    set(CMAKE_TOOLCHAIN_FILE "D:/Project/vcpkg/scripts/buildsystems/vcpkg.cmake"
        CACHE STRING "Vcpkg toolchain file")
endif()

project(listory_search VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Quick Concurrent)

# Find Protobuf (vcpkg will provide it automatically if CMAKE_TOOLCHAIN_FILE is set)
find_package(Protobuf REQUIRED)
find_package(absl REQUIRED)

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
    protobuf::libprotobuf
)

# Fix MSVC-specific linker flags when using MinGW with vcpkg MSVC libraries
if(MINGW)
    # Function to recursively clean MSVC flags from a target and its dependencies
    function(clean_msvc_flags target)
        if(TARGET ${target})
            get_target_property(link_libs ${target} INTERFACE_LINK_LIBRARIES)
            if(link_libs)
                set(cleaned_libs "")
                foreach(lib ${link_libs})
                    # Skip MSVC-specific flags
                    if(NOT lib MATCHES "^-ignore:" AND NOT lib MATCHES "^/IGNORE:")
                        list(APPEND cleaned_libs ${lib})
                        # Recursively clean dependencies
                        if(TARGET ${lib})
                            clean_msvc_flags(${lib})
                        endif()
                    endif()
                endforeach()
                set_target_properties(${target} PROPERTIES INTERFACE_LINK_LIBRARIES "${cleaned_libs}")
            endif()

            # Also clean INTERFACE_LINK_OPTIONS
            get_target_property(link_opts ${target} INTERFACE_LINK_OPTIONS)
            if(link_opts)
                set(cleaned_opts "")
                foreach(opt ${link_opts})
                    if(NOT opt MATCHES "^-ignore:" AND NOT opt MATCHES "^/IGNORE:")
                        list(APPEND cleaned_opts ${opt})
                    endif()
                endforeach()
                set_target_properties(${target} PROPERTIES INTERFACE_LINK_OPTIONS "${cleaned_opts}")
            endif()
        endif()
    endfunction()

    # Clean protobuf and its dependencies
    clean_msvc_flags(protobuf::libprotobuf)
endif()

# Windows specific settings
if(WIN32)
    target_compile_definitions(listory_search PRIVATE UNICODE _UNICODE)
    set_target_properties(listory_search PROPERTIES
        WIN32_EXECUTABLE TRUE
    )
endif()
