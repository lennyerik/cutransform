use std::fs;
use std::path::Path;
use std::process::Command;

fn main() -> Result<(), String> {
    println!("cargo:rerun-if-changed=src/kernel.c");

    let build_dir =
        std::env::var("OUT_DIR").map_err(|_| "Failed to get OUT_DIR environment variable")?;

    run_and_check_command(
        "clang",
        &[
            "-cc1",
            "-O3",
            "-triple=nvptx64-nvidia-cuda",
            "-target-cpu",
            "sm_86",
            "-target-feature",
            "+ptx75",
            "-emit-llvm-bc",
            "-o",
            &format!("{}/kernel.bc", build_dir),
            "src/kernel.c",
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

    // In this example, we compile the ptx to a cubin and embed the cubin in the binary
    // by writing a temporary rust file containing a u8 array. You could also embed the ptx
    // directly and compile at runtime using Module::from_ptx for greater compatibility.
    // This example just showcases what is possible.
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

    let bytes = fs::read(Path::new(&format!("{}/kernel.cubin", build_dir)))
        .map_err(|_| "Failed to read compiled kernel cubin file")?;
    let bytes_formatted = bytes
        .iter()
        .map(|b| b.to_string())
        .fold(String::new(), |a, b| a + &b + ", ");

    fs::write(
        Path::new(&format!("{}/kernel.rs", build_dir)),
        format!(
            "pub const KERNEL_COMPILED: [u8; {}] = [{}];",
            bytes.len(),
            bytes_formatted
        ),
    )
    .map_err(|_| "Failed to write output kernel.rs file")?;

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
