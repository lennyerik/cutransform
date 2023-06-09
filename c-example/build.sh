#!/bin/bash

set -e

mkdir build &> /dev/null || true
clang -cc1 -Wall -Werror -pedantic -O3 -triple=nvptx64-nvidia-cuda -target-cpu sm_86 -target-feature +ptx75 -emit-llvm-bc -o build/kernel.bc kernel.c
../cutransform/target/release/cutransform build/kernel.bc
llc -O3 -mcpu=sm_86 -mattr=+ptx75 build/kernel.bc
ptxas --allow-expensive-optimizations true --gpu-name sm_89 -o build/kernel.cubin build/kernel.s

if [[ "$(pkg-config --libs --cflags cuda &> /dev/null; echo $?)" == "0" ]]; then
  clang -Wall -Werror -pedantic -O3 --std=c17 `pkg-config --libs --cflags cuda` -o c-example main.c
else
  clang -Wall -Werror -pedantic -O3 --std=c17 -lcuda -o c-example main.c
fi
