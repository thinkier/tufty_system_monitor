#include "pico/stdlib.h"
#include <cstdio>
#include <cstring>
#include <string>
#include <algorithm>
#include <iostream>
#include "pico/time.h"
#include "pico/platform.h"
#include "pico/multicore.h"
#include "pico/mutex.h"
#include "json.hpp"

#include "common/pimoroni_common.hpp"
#include "drivers/st7789/st7789.hpp"
#include "libraries/pico_graphics/pico_graphics.hpp"
#include "tufty2040.hpp"
#include "button.hpp"

using json = nlohmann::json;
using namespace pimoroni;
using namespace std;

Tufty2040 tufty;

ST7789 st7789(
        Tufty2040::WIDTH,
        Tufty2040::HEIGHT,
        ROTATE_0,
        ParallelPins{
                Tufty2040::LCD_CS,
                Tufty2040::LCD_DC,
                Tufty2040::LCD_WR,
                Tufty2040::LCD_RD,
                Tufty2040::LCD_D0,
                Tufty2040::BACKLIGHT
        }
);

PicoGraphics_PenRGB332 graphics(st7789.width, st7789.height, nullptr);

Button button_a(Tufty2040::A);
Button button_b(Tufty2040::B);
Button button_c(Tufty2040::C);
Button button_up(Tufty2040::UP);
Button button_down(Tufty2040::DOWN);

auto_init_mutex(serial_data_mtx);
shared_ptr<json> serial_data;

void comms() {
    string data;
    while (getline(cin, data)) {
        json j = json::parse(data);

        mutex_enter_blocking(&serial_data_mtx);
        serial_data = make_shared<json>(j);
        mutex_exit(&serial_data_mtx);
    }
}

int main() {
    stdio_init_all();
    multicore_launch_core1(comms);

    st7789.set_backlight(128);

    Pen BG = graphics.create_pen(0, 0, 0);

    while (true) {
        mutex_enter_blocking(&serial_data_mtx);
        cout << serial_data << endl;
        mutex_exit(&serial_data_mtx);

        graphics.set_pen(BG);
        graphics.clear();

        st7789.update(&graphics);
        sleep_ms(1000);
    }
}
