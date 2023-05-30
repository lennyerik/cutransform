#!/bin/bash

set -xe

mkdir build |
clang -cc1 -Wall -Werror -pedantic -O3 -triple=nvptx64-nvidia-cuda -target-cpu sm_86 -target-feature +ptx75 -emit-llvm-bc -o build/kernel.bc kernel.c
../cutransform/target/release/cutransform build/kernel.bc
llc -O3 -mcpu=sm_86 -mattr=+ptx75 build/kernel.bc
ptxas --allow-expensive-optimizations true --gpu-name sm_89 -o build/kernel.cubin build/kernel.s
clang -Wall -Werror -pedantic -O3 --std=c17 `pkg-config --libs --cflags cuda` -lcuda -o c-example main.c
