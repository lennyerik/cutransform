const std = @import("std");
const EmitOption = std.build.LibExeObjStep.EmitOption;

pub fn build(b: *std.build.Builder) !void {
    std.fs.cwd().makeDir("build") catch |err| {
        if (err != std.os.MakeDirError.PathAlreadyExists) {
            return err;
        }
    };
    
    // CUDA kernel
    const kernel_build = setupKernelBuild(b);

    // Main executable
    const executable_build = b.addExecutable("zig-example", "src/main.zig");
    executable_build.setTarget(b.standardTargetOptions(.{}));
    executable_build.setBuildMode(.ReleaseFast);
    executable_build.emit_bin = EmitOption{
        .emit_to = "zig-example"
    };
    executable_build.linkLibC();
    executable_build.linkSystemLibrary("cuda");
    executable_build.step.dependOn(kernel_build);

    b.default_step = &executable_build.step;
}

fn setupKernelBuild(b: *std.build.Builder) *std.build.Step {
    // Build the LLVM bitcode from the Zig source
    const zig_build_step = b.addObject("kernel", "src/kernel.zig");

    var cuda_features = std.Target.Cpu.Feature.Set.empty;
    cuda_features.addFeature(@enumToInt(std.Target.nvptx.Feature.ptx75));
    cuda_features.addFeature(@enumToInt(std.Target.nvptx.Feature.sm_86));
    zig_build_step.setTarget(.{
        .ofmt = .nvptx,
        .os_tag = .cuda,
        .abi = .eabi,
        .cpu_arch = .nvptx64,
        .cpu_features_add = cuda_features,
    });

    zig_build_step.setBuildMode(.ReleaseSmall);
    zig_build_step.emit_bin = .no_emit;
    zig_build_step.emit_llvm_bc = EmitOption{
        .emit_to = "build/kernel.bc"
    };

    // Transform the bitcode
    const cutransform_cmd_str = &[_][]const u8 {
        "../cutransform/target/release/cutransform", "build/kernel.bc"
    };
    const cutransform_step = b.addSystemCommand(cutransform_cmd_str);
    cutransform_step.step.dependOn(&zig_build_step.step);

    // Compile the bitcode
    const llc_cmd_str = &[_][]const u8 {
        "llc", "-O3", "-mcpu=sm_86", "-mattr=+ptx75", "-o", "build/kernel.s", "build/kernel.bc"
    };
    const bitcode_compile_step = b.addSystemCommand(llc_cmd_str);
    bitcode_compile_step.step.dependOn(&cutransform_step.step);

    // Compile the ptx
    const ptxas_cmd_str = &[_][]const u8 {
        "ptxas", "--allow-expensive-optimizations", "true",
        "-o", "src/kernel.cubin", "--gpu-name", "sm_89", "build/kernel.s"
    };
    const ptx_compile_step = b.addSystemCommand(ptxas_cmd_str);
    ptx_compile_step.step.dependOn(&bitcode_compile_step.step);
    
    return &ptx_compile_step.step;
}
