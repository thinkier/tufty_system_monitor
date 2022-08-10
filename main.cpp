#include "pico/stdlib.h"
#include <stdio.h>
#include <cstring>
#include <string>
#include <algorithm>
#include "pico/time.h"
#include "pico/platform.h"

#include "common/pimoroni_common.hpp"
#include "drivers/st7789/st7789.hpp"
#include "libraries/pico_graphics/pico_graphics.hpp"
#include "tufty2040.hpp"
#include "button.hpp"

using namespace pimoroni;

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

uint32_t time() {
    absolute_time_t t = get_absolute_time();
    return to_ms_since_boot(t);
}

// HSV Conversion expects float inputs in the range of 0.00-1.00 for each channel
// Outputs are rgb in the range 0-255 for each channel
void from_hsv(float h, float s, float v, uint8_t &r, uint8_t &g, uint8_t &b) {
    float i = floor(h * 6.0f);
    float f = h * 6.0f - i;
    v *= 255.0f;
    uint8_t p = v * (1.0f - s);
    uint8_t q = v * (1.0f - f * s);
    uint8_t t = v * (1.0f - (1.0f - f) * s);

    switch (int(i) % 6) {
        case 0: r = v; g = t; b = p; break;
        case 1: r = q; g = v; b = p; break;
        case 2: r = p; g = v; b = t; break;
        case 3: r = p; g = q; b = v; break;
        case 4: r = t; g = p; b = v; break;
        case 5: r = v; g = p; b = q; break;
    }
}

int main() {
    st7789.set_backlight(255);

    Pen WHITE = graphics.create_pen(255, 255, 255);
    Pen BG = graphics.create_pen(120, 40, 60);

    struct pt {
        float      x;
        float      y;
        uint8_t    r;
        float     dx;
        float     dy;
        uint16_t pen;
    };

    std::vector<pt> shapes;
    for(int i = 0; i < 100; i++) {
        pt shape;
        shape.x = rand() % graphics.bounds.w;
        shape.y = rand() % graphics.bounds.h;
        shape.r = (rand() % 10) + 3;
        shape.dx = float(rand() % 255) / 64.0f;
        shape.dy = float(rand() % 255) / 64.0f;
        shape.pen = graphics.create_pen(rand() % 255, rand() % 255, rand() % 255);
        shapes.push_back(shape);
    }

    Point text_location(0, 0);
    uint8_t i = 0;

    while(true) {

        graphics.set_pen(BG);
        graphics.clear();

        for(auto &shape : shapes) {
            shape.x += shape.dx;
            shape.y += shape.dy;
            if((shape.x - shape.r) < 0) {
                shape.dx *= -1;
                shape.x = shape.r;
            }
            if((shape.x + shape.r) >= graphics.bounds.w) {
                shape.dx *= -1;
                shape.x = graphics.bounds.w - shape.r;
            }
            if((shape.y - shape.r) < 0) {
                shape.dy *= -1;
                shape.y = shape.r;
            }
            if((shape.y + shape.r) >= graphics.bounds.h) {
                shape.dy *= -1;
                shape.y = graphics.bounds.h - shape.r;
            }

            graphics.set_pen(shape.pen);
            graphics.circle(Point(shape.x, shape.y), shape.r);

        }


        graphics.set_pen(WHITE);
        graphics.text("Hello World", text_location, 320);

        // update screen
        st7789.update(&graphics);

        i+=10;
        tufty.led(i);
    }

    return 0;
}
