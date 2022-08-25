#include "pico/stdlib.h"
#include <cstdio>
#include <cstring>
#include <string>
#include <algorithm>
#include <iostream>
#include <pico/unique_id.h>
#include "pico/time.h"
#include "pico/platform.h"
#include "pico/multicore.h"
#include "pico/mutex.h"
#include "json.hpp"

#include "font6_data.hpp"
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
Pen BLACK = graphics.create_pen(0, 0, 0);
Pen WHITE = graphics.create_pen(255, 255, 255);

Pen CPU_TEMP = graphics.create_pen(239, 165, 95);
Pen GPU_TEMP = graphics.create_pen(239, 95, 98);


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
        st7789.set_backlight(255);

        json j = json::parse(data);

        mutex_enter_blocking(&serial_data_mtx);
        serial_data = make_shared<json>(j);
        mutex_exit(&serial_data_mtx);
    }
}

int GRAPH_TEXT_SCALE = 2;
int FONT_HEIGHT = 6;
int GRAPH_TEXT_HEIGHT = GRAPH_TEXT_SCALE * FONT_HEIGHT;

// Right-Justify text rendering function
int text_rjust(PicoGraphics *g, const string &text, Point pt, int32_t wrap, float s = 2.0f) {
    uint16_t width = g->measure_text(text, s);

    g->text(text, Point(pt.x - width, pt.y), wrap, s);

    return width;
}

void graph_temperatures(PicoGraphics *g, Rect bounds, const Pen &p, json *data) {
    uint16_t min = -1;
    uint16_t max = 0;
    for (const auto &num: *data) {
        if (num > max) {
            max = num;
        }
        if (num < min) {
            min = num;
        }
    }

    // Strip last digit (floor)
    min /= 10;
    min *= 10;

    float delta = 10 * ceil((float) (max - min) / 10);
    if (delta < 20) {
        max = min + 20;
        delta = 20;
    }

    g->set_pen(WHITE);

    string max_s = to_string(max / 10);
    string min_s = to_string(min / 10);
    Point max_p = Point(bounds.x + bounds.w, bounds.y);
    Point min_p = Point(bounds.x + bounds.w, bounds.y + bounds.h - GRAPH_TEXT_HEIGHT);

    {
        int wd1 = text_rjust(g, max_s, max_p, 60, GRAPH_TEXT_SCALE);
        int wd2 = text_rjust(g, min_s, min_p, 60, GRAPH_TEXT_SCALE);

        bounds.y += GRAPH_TEXT_HEIGHT / 2;
        bounds.h -= GRAPH_TEXT_HEIGHT;
        bounds.w -= 4;
        g->line(Point(bounds.x, bounds.y), Point(bounds.x + bounds.w - wd1, bounds.y));
        g->line(Point(bounds.x, bounds.y + bounds.h), Point(bounds.x + bounds.w - wd2, bounds.y + bounds.h));
        bounds.w -= wd1 > wd2 ? wd1 : wd2;
        bounds.h -= 1;
    }


    g->set_pen(p);
    uint8_t size = data->size();
    float s = size;
    float w = bounds.w;
    float h = bounds.h;
    for (uint8_t i = 1; i < size; i++) {
        float prev = (*data)[i - 1];
        float cur = (*data)[i];
        float t0 = (prev - (float) min) / delta;
        float t1 = (cur - (float) min) / delta;

        if (delta == 0) {
            t0 = t1 = 0.5;
        }

        int x0 = (int) ((((float) i - 1.0f) / s) * w) + bounds.x;
        int x1 = (int) ((((float) i) / s) * w) + bounds.x;
        int y0 = bounds.y + bounds.h - (int) (t0 * h);
        int y1 = bounds.y + bounds.h - (int) (t1 * h);

        Point p0 = Point(x0, y0);
        Point p1 = Point(x1, y1);
        p0.clamp(bounds);
        p1.clamp(bounds);
        g->line(p0, p1);
    }
}

int main() {
    stdio_usb_init();
    stdio_usb_connected();
    multicore_launch_core1(comms);

    st7789.set_backlight(80);
    graphics.set_font(&font6);

    while (true) {
        mutex_enter_blocking(&serial_data_mtx);
        bool ready = (*serial_data).is_object();
        mutex_exit(&serial_data_mtx);

        if (ready) {
            break;
        }

        sleep_ms(50);
    }

    while (true) {
        st7789.update(&graphics);
        sleep_ms(10);
        graphics.set_pen(BLACK);
        graphics.clear();
        graphics.set_pen(WHITE);

        mutex_enter_blocking(&serial_data_mtx);
        string cpu_name = (*serial_data)["cpu_name"];
        string gpu_name = (*serial_data)["gpu_name"];
        string time = (*serial_data)["time"];
        json cpu_temps = (*serial_data)["cpu_temps"];
        json gpu_temps = (*serial_data)["gpu_temps"];
        mutex_exit(&serial_data_mtx);

        int32_t time_w = graphics.measure_text(time, 2.5);

        graphics.text(time, Point(320 - time_w, 2), 320, 2.5);
        graphics.text(cpu_name, Point(0, 0), 240, 2);
        graphics.text(gpu_name, Point(0, 120), 240, 2);

        graph_temperatures(&graphics, Rect(0, 20, 320, 100), CPU_TEMP, &cpu_temps);
        graph_temperatures(&graphics, Rect(0, 140, 320, 100), GPU_TEMP, &gpu_temps);
    }
}
