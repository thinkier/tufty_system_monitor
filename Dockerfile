FROM debian:stable
WORKDIR /env
RUN apt-get update; apt-get install --yes python3 git cmake gcc-arm-none-eabi libnewlib-arm-none-eabi build-essential libstdc++-arm-none-eabi-newlib; rm -rf /var/apt/cache

RUN git clone https://github.com/raspberrypi/pico-sdk
RUN cd pico-sdk; git submodule update --init --recursive
ENV PICO_SDK_PATH /env/pico-sdk

RUN git clone https://github.com/pimoroni/pimoroni-pico
RUN cd pimoroni-pico; git submodule update --init --recursive
ENV PIMORONI_PICO_PATH /env/pimoroni-pico

VOLUME ["/env/pico-project", "/env/build"]
WORKDIR /env/build
COPY build.sh .
RUN chmod +x build.sh

ENTRYPOINT ./build.sh
