const std = @import("std");
const cuda = @cImport(
    @cInclude("cuda.h"),
);

const kernel_cubin = @embedFile("kernel.cubin");

const ARR_SIZE = 512;

pub fn main() !void {
    if (cuda.cuInit(0) != cuda.CUDA_SUCCESS) {
        return error.CudaInitialisationFailed;
    }

    var dev: cuda.CUdevice = undefined;
    if (cuda.cuDeviceGet(&dev, 0) != cuda.CUDA_SUCCESS) {
        return error.NoCudaDeviceFound;
    }

    var ctx: cuda.CUcontext = undefined;
    if (cuda.cuCtxCreate(&ctx, 0, dev) != cuda.CUDA_SUCCESS) {
        return error.FailedToCreateDeviceContext;
    }

    var module: cuda.CUmodule = undefined;
    var err = cuda.cuModuleLoadData(&module, kernel_cubin); 
    if (err != cuda.CUDA_SUCCESS) {
        return error.FailedToLoadCubin;
    }

    var kernel_func: cuda.CUfunction = undefined;
    if (cuda.cuModuleGetFunction(&kernel_func, module, "kernel") != cuda.CUDA_SUCCESS) {
        return error.FailedToGetKernelFunction;
    }

    var device_arr: cuda.CUdeviceptr = undefined;
    if (cuda.cuMemAlloc(&device_arr, @sizeOf(u32) * ARR_SIZE) != cuda.CUDA_SUCCESS) {
        return error.FailedToAllocateDeviceMemory;
    }

    const args = &[_]?*anyopaque {
        &device_arr
    };
    const launch_result = cuda.cuLaunchKernel(
        kernel_func,
        1,        1, 1,  // Grid size
        ARR_SIZE, 1, 1,  // Block size
        0,               // Shared mem
        null,            // Stream
        args,            // Arguments
        0                // Extra flags
    );
    if (launch_result != cuda.CUDA_SUCCESS) {
        return error.FailedToLaunchKernel;
    }

    if (cuda.cuCtxSynchronize() != cuda.CUDA_SUCCESS) {
        return error.FailedToSynchronize;
    }

    var host_arr: [ARR_SIZE]u32 = undefined;
    if (cuda.cuMemcpyDtoH(@ptrCast(*anyopaque, &host_arr), device_arr, @sizeOf(u32) * ARR_SIZE) != cuda.CUDA_SUCCESS) {
        return error.FailedMemcpyDeviceToHost;
    }
    
    _ = cuda.cuMemFree(device_arr);

    const stdout = std.io.getStdOut().writer();
    var i: usize = 0;
    while (i < host_arr.len) {
        try stdout.print("arr[{}] = {}\n", .{i, host_arr[i]});
        i += 1;
    }
}
