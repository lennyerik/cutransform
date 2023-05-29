use std::process::Command;

fn main() -> Result<(), String> {
    println!("cargo:rerun-if-changed=src/kernel.rs");

    let build_dir =
        std::env::var("OUT_DIR").map_err(|_| "Failed to get OUT_DIR environment variable")?;

    run_and_check_command(
        "rustc",
        &[
            "-O",
            "-C",
            "opt-level=3",
            "--emit",
            "llvm-bc",
            "--target",
            "nvptx64-nvidia-cuda",
            "-C",
            "target-cpu=sm_86",
            "-C",
            "target-feature=+ptx75",
            "--crate-type",
            "lib",
            "-o",
            &format!("{}/kernel.bc", build_dir),
            "src/kernel.rs",
        ],
    )?;

    run_and_check_command(
        "../cutransform/target/release/cutransform",
        &[&format!("{}/kernel.bc", build_dir)],
    )?;

    run_and_check_command(
        "llc",
        &[
            "-O3",
            "-mcpu=sm_86",
            "-mattr=+ptx75",
            &format!("{}/kernel.bc", build_dir),
        ],
    )?;

    run_and_check_command(
        "ptxas",
        &[
            "--allow-expensive-optimizations",
            "true",
            "--gpu-name",
            "sm_89",
            "-o",
            &format!("{}/kernel.cubin", build_dir),
            &format!("{}/kernel.s", build_dir),
        ],
    )?;

    Ok(())
}

fn run_and_check_command(executable: &str, args: &[&str]) -> Result<(), String> {
    if Command::new(executable)
        .args(args)
        .status()
        .map_err(|_| format!("Failed to get command status of {}", executable))?
        .success()
    {
        Ok(())
    } else {
        Err(format!("Command {} failed!", executable))
    }
}
