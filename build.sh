#!/bin/bash
mkdir build
cd build || exit 1

cmake -G "MSYS Makefiles" ..
make
