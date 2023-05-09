# cutransform
Are you tired of having to write your CUDA kernel code in C++?
This project aims to make it possible to compile CUDA kernels written in any language supported by LLVM without much hassle.
Specifically, this is basically a transpiler from LLVM-IR to NVVM-IR.

This, of course, includes languages like plain C or even Rust.
CUDA in Rust is not yet good and [Rust-CUDA](https://github.com/Rust-GPU/Rust-CUDA) has been stale since July 2022.
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

## Example usage
First, make sure you have the nvptx Rust target installed:

    rustup target add nvptx64-nvidia-cuda

You also need to compile the cutransform binary:

    cd cutransform
    cargo build --release

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

