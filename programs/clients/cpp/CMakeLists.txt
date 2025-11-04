cmake_minimum_required(VERSION 3.10)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CXX_FLAGS "-g -O3 -Wall -Werror")
#set(CMAKE_CXX_FLAGS "-g -O3 -Wall -Werror -ggdb -fsanitize=address -fno-omit-frame-pointer -static-libstdc++ -static-libasan -lrt")

project(cpp_client VERSION 1.0)
# file(GLOB_RECURSE SRC_FILES ..)
file(GLOB SRC_FILES ./src/*.h, ./src/lib/*.h ./src/*.cc ./src/lib/*.c ./src/lib/*.cc ./src/lib/vincenthz/*)
#file(GLOB LIB_FILES ./lib/*.cc ./lib/*.h ./lib/*.c)
add_executable(cpp_client ${SRC_FILES})
#file(GLOB LIB_FILES ./lib/*.cc ./lib/*.h ./lib/*.c)
#add_executable(cpp_client ${SRC_FILES} ${LIB_FILES})
# add_executable(cpp_client main.cc modsecurity_json.cc modsecurity_json.h mjson.c mjson.h json_parser.cc json_parser.h)


#find_package(Boost REQUIRED)
#include_directories(${Boost_INCLUDE_DIRS})
#add_subdirectory(./boost-json/)
find_package(Poco REQUIRED JSON)
# find_package(Boost REQUIRED COMPONENTS json)

target_include_directories(cpp_client PRIVATE ./boost_1_82_0/)

target_link_libraries(cpp_client -lyajl -ljansson -lm -ljson-c)
target_link_libraries(cpp_client Poco::JSON)
#target_link_libraries(cpp_client Boost::json)
# -lm -g -O3 -Wall -Werror -ljansson
#target_include_directories(cpp_client PRIVATE ${CMAKE_CURRECT_SOURCE_DIR}/include)
#target_include_directories(cpp_client PUBLIC ${CMAKE_CURRECT_SOURCE_DIR}/modsecurity_mock)


# all:
# 	g++ -o client main.cpp modsecurity_mock/json.cc -g -O3 -Wall -Werror -lyajl

# gcc -o client main.c cJSON.c json.c json_parser.c -lm -g -O3 -Wall -Werror -ljansson
#-I./lib/jansson-2.14.1/build/include/

# # https://stackoverflow.com/questions/42357101/how-to-create-and-run-a-make-file-for-c
# # The name of the source files
# SOURCES = main.c cJSON.c
#
# # The name of the executable
# EXE = c_client
#
# # Flags for compilation (adding warnings are always good)
# CFLAGS = -Wall -Werror
#
# # Flags for linking (none for the moment)
# LDFLAGS =
#
# # Libraries to link with (none for the moment)
# LIBS =
#
# # Use the GCC frontend program when linking
# LD = gcc
#
# # This creates a list of object files from the source files
# OBJECTS = $(SOURCES:%.c=%.o)
#
# # The first target, this will be the default target if none is specified
# # This target tells "make" to make the "all" target
# default: all
#
# # Having an "all" target is customary, so one could write "make all"
# # It depends on the executable program
# all: $(EXE)
#
# # This will link the executable from the object files
# $(EXE): $(OBJECTS)
# 	$(LD) $(LDFLAGS) $(OBJECTS) -o  $(EXE) $(LIBS)
#
# # This is a target that will compiler all needed source files into object files
# # We don't need to specify a command or any rules, "make" will handle it automatically
# %.o: %.c
#
# # Target to clean up after us
# clean:
# 	-rm -f $(EXE)      # Remove the executable file
# 	-rm -f $(OBJECTS)  # Remove the object files
#
# # Finally we need to tell "make" what source and header file each object file depends on
# main.o: main.c cJSON.o
# cJSON.o: cJSON.c cJSON.h
# # cJSON.o: cJSON.c cJSON.h
