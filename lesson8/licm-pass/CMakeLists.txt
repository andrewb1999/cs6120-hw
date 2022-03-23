cmake_minimum_required(VERSION 3.1)
project(LICMPass)

set(CMAKE_C_COMPILER "/opt/llvm/bin/clang-13")
set(CMAKE_CXX_COMPILER "/opt/llvm/bin/clang-13")

# support C++14 features used by LLVM 10.0.0
set(CMAKE_CXX_STANDARD 14)

find_package(LLVM REQUIRED CONFIG)
message(STATUS ${LLVM_INCLUDE_DIRS})
add_definitions(${LLVM_DEFINITIONS})
include_directories(${LLVM_INCLUDE_DIRS})
link_directories(${LLVM_LIBRARY_DIRS})

add_subdirectory(licm)  # Use your pass name here.
add_subdirectory(mem2reg)  # Use your pass name here.
