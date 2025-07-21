cmake_minimum_required(VERSION 3.16)
project(OpenAstroViz CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_library(poc_sgp STATIC poc_sgp.cpp)
target_include_directories(poc_sgp PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})
target_compile_definitions(poc_sgp PUBLIC POC_SGP_NO_MAIN)

find_package(GTest REQUIRED)

add_executable(poc_sgp_test tests/poc_sgp_test.cpp)
target_link_libraries(poc_sgp_test poc_sgp GTest::gtest GTest::gtest_main)

enable_testing()
add_test(NAME poc_sgp_test COMMAND poc_sgp_test)
