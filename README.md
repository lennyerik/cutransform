# cutransform
Are you tired of having to write your CUDA kernel code in C++?
This project aims to make it possible to compile CUDA kernels written in any language supported by LLVM without much hassle.
Specifically, this is basically a transpiler from LLVM-IR to NVVM-IR.

Importantly, languages like plain C, Rust and Zig are all supported.
Expecially CUDA in Rust is not yet very good and [Rust-CUDA](https://github.com/Rust-GPU/Rust-CUDA) has been stale since July 2022.
Maybe we can fix that by using a different approach to the problem of CUDA codegen.

**This is not a CUDA runtime API wrapper! You cannot run the kernels with this project alone!**
If you're just looking for a simple way to write CUDA in Rust though, you're in luck.
[cust](https://crates.io/crates/cust) is a really good wrapper around the CUDA API.


## How it works
In order to compile a kernel in any language with an LLVM frontend, we

* Invoke the standard compiler for the language and tell it to output LLVM bitcode for the nvptx64-nvidia-cuda target
* Pass the generated bitcode to the code transformer (cutransform)
  * The transformer will parse the bitcode and add required attributes and functions and structs
  * It will output this modified version of the bitcode
* Finally the bitcode can simply be passed through the llvm-bitcode compiler, llc to generate the PTX assembly
* (Optional) Additionally can now choose to assemble the PTX to a SASS (cubin) program for your specific graphics card using Nvidia's proprietary ptxas assembler


## Setup
You should already have

* clang
* llvm
* cuda

Then compile the cutransform binary:

    cd cutransform
    cargo build --release

If the build fails with an error message from the `llvm-sys` crate, you likely have a build of LLVM without the static libraries.
This is the default for newer LLVM binary distributions.
To build with a dynamically linked LLVM, run:

    cargo build --release --features dynamic-llvm

instead.

## Rust example usage
First, make sure you have the nvptx Rust target installed:

    rustup target add nvptx64-nvidia-cuda

Here is an example Rust kernel:
```rust
#![no_std]

extern "C" {
    fn threadIdxX() -> u32;
}

#[no_mangle]
pub extern "C" fn kernel(arr: *mut u32) {
    unsafe {
        let idx = threadIdxX() as usize;
        *arr.add(idx) = 123;
    }
}
```

**Please note that all kernel functions should have a name starting with the word "kernel". Otherwise they won't be exported.**

To compile the Rust kernel to LLVM bitcode, run:

    rustc -O -C opt-level=3 -o kernel.bc --emit llvm-bc --target nvptx64-nvidia-cuda -C target-cpu=sm_86 -C target-feature=+ptx75 --crate-type lib kernel.rs

You can change `sm_86` flag to the minimum supported compute capability of your kernel (8.6 is the newest supported in clang and it's mostly for 30-series cards and onwards).
Refer to [this Wikipedia page](https://en.wikipedia.org/wiki/CUDA#GPUs_supported) for a list of cards and their supported compute capabilities.

Now, run cutransform on the llvm bitcode

    cutransform/target/release/cutransform kernel.bc

Finally, compile the new bitcode to PTX:

    llc -O3 -mcpu=sm_86 -mattr=+ptx75 kernel.bc

Now you can also choose to assemble the PTX for your card:

    ptxas --allow-expensive-optimizations true -o kernel.cubin --gpu-name sm_89 kernel.s

Where you can again change `sm_89` to the compute capability of your card.
Compute capability 8.9 is for 40-series cards.

For a complete and integrated example, see the `rust-example` crate included in this repo.


## C example usage
Here is an example C kernel:
```c
extern int threadIdxX(void);

void kernel(int *arr) {
    arr[threadIdxX()] = 123;
}
```

**Please note that all kernel functions should have a name starting with the word "kernel". Otherwise they won't be exported.**

To compile the C kernel to LLVM bitcode, run:

    clang -cc1 -O3 -triple=nvptx64-nvidia-cuda -target-cpu sm_86 -target-feature +ptx75 -emit-llvm-bc -o kernel.bc kernel.c

Now, run cutransform on the llvm bitcode

    cutransform/target/release/cutransform kernel.bc

Finally, compile the new bitcode to PTX:

    llc -O3 -mcpu=sm_86 -mattr=+ptx75 kernel.bc

Now you can also choose to assemble the PTX for your card:

    ptxas --allow-expensive-optimizations true -o kernel.cubin --gpu-name sm_89 kernel.s

Where you can again change `sm_89` to the compute capability of your card.
Compute capability 8.9 is for 40-series cards.

For a complete and integrated example, see the `c-example` folder included in this repo.

## Zig example usage
Here is an example Zig kernel:
```zig
extern fn threadIdxX() i32;

export fn kernel(arr: [*]u32) callconv(.C) void {
    arr[@intCast(usize, threadIdxX())] = 123;
}

// Override the default entrypoint
pub fn _start() callconv(.Naked) void {}
```

**Please note that all kernel functions should have a name starting with the word "kernel". Otherwise they won't be exported.**

To compile the Zig kernel to LLVM bitcode, run:

    zig build-obj -O ReleaseSmall -target nvptx64-cuda -mcpu sm_86+ptx75 -fno-emit-asm -femit-llvm-bc=kernel.bc kernel.zig

Now, run cutransform on the llvm bitcode

    cutransform/target/release/cutransform kernel.bc

Finally, compile the new bitcode to PTX:

    llc -O3 -mcpu=sm_86 -mattr=+ptx75 kernel.bc

Now you can also choose to assemble the PTX for your card:

    ptxas --allow-expensive-optimizations true -o kernel.cubin --gpu-name sm_89 kernel.s

Where you can again change `sm_89` to the compute capability of your card.
Compute capability 8.9 is for 40-series cards.

For a complete and integrated example, see the `zig-example` folder included in this repo.

