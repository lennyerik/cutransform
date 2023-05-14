#include <stdio.h>
#include <cuda.h>

#define CUDA_CHECK_ERR(err, err_msg) if (err != CUDA_SUCCESS) { fprintf(stderr, "%s\nError code: %d\n", err_msg, err); return 1; }

#define ARR_SIZE 512

int main(void) {
    CUresult result = cuInit(0);
    CUDA_CHECK_ERR(result, "Failed to intialise CUDA!");

    // Get the first CUDA device
    CUdevice device;
    result = cuDeviceGet(&device, 0);
    CUDA_CHECK_ERR(result, "Failed to get default CUDA device!");

    CUcontext ctx;
    result = cuCtxCreate(&ctx, 0, device);
    CUDA_CHECK_ERR(result, "Failed to create CUDA device context!");

    CUmodule module;
    result = cuModuleLoad(&module, "build/kernel.cubin");
    CUDA_CHECK_ERR(result, "Failed to load build/kernel.cubin! Please make sure the file exists.");

    CUfunction kernel_func;
    result = cuModuleGetFunction(&kernel_func, module, "kernel");
    CUDA_CHECK_ERR(result, "Failed to get function `kernel` from cubin!");

    // Allocate the array
    CUdeviceptr device_arr;
    result = cuMemAlloc(&device_arr, sizeof(int) * ARR_SIZE);
    CUDA_CHECK_ERR(result, "Failed to allocate array on device");

    // Launch the kernel
    void *args[] = {
        (void *) &device_arr
    };
    result = cuLaunchKernel(
        kernel_func,
        1,        1, 1,  // Grid size
        ARR_SIZE, 1, 1,  // Block size
        0,               // Shared mem
        0,               // Stream
        args,            // Arguments
        0                // Extra flags
    );
    CUDA_CHECK_ERR(result, "Failed to launch kernel!");

    // Synchronize
    cuCtxSynchronize();

    // Copy the data into host memory
    int host_arr[ARR_SIZE] = {0};
    result = cuMemcpyDtoH((void *) host_arr, device_arr, sizeof(int) * ARR_SIZE);
    cuMemFree(device_arr);
    CUDA_CHECK_ERR(result, "Failed to copy array from device to host!");

    for (size_t i = 0; i < ARR_SIZE; i++) {
        printf("arr[%ld] = %d\n", i, host_arr[i]);
    }

    return 0;
}
