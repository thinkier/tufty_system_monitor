cmake_minimum_required(VERSION 3.13)
set(NAME tufty2040_system_monitor)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

# initialize pico-sdk from submodule
# note: this must happen before project()
include(../pico-sdk/pico_sdk_init.cmake)
include(../pimoroni-pico/pimoroni_pico_import.cmake)

project(${NAME} C CXX ASM)

# initialize the Raspberry Pi Pico SDK
pico_sdk_init()

# initialize all the pimoroni drivers and libraries and stuff
include(libraries/tufty2040/tufty2040)
include(libraries/pico_graphics/pico_graphics)
include(drivers/st7789/st7789)
include(drivers/button/button)
# required commons that aren't imported upstream
include(libraries/bitmap_fonts/bitmap_fonts)
include(libraries/hershey_fonts/hershey_fonts)
include(common/pimoroni_bus)

# rest of your project
add_executable(${NAME} main.cpp)

target_link_libraries(${NAME}
        pico_stdlib
        pico_multicore
        pico_sync
        tufty2040
        pico_graphics
        st7789
        button
        )

# enable usb output
pico_enable_stdio_usb(${NAME} 1)
pico_enable_stdio_uart(${NAME} 0)

pico_add_extra_outputs(${NAME})

# create map/bin/hex/uf2 file in addition to ELF.
pico_add_extra_outputs(${NAME})
