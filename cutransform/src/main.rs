use inkwell::{context::Context, module::Module};
use std::path::Path;

mod nvvm_cleanup;
mod nvvm_intrinsics;
mod nvvm_metadata;

fn main() -> Result<(), String> {
    let file_path_str = std::env::args().nth(1).unwrap_or("kernel.bc".into());
    let file_path = Path::new(&file_path_str);

    let ctx = Context::create();
    let module = Module::parse_bitcode_from_path(file_path, &ctx).map_err(|e| e.to_string())?;

    nvvm_metadata::add_metadata_to_kernel_functions(&module)?;
    nvvm_intrinsics::replace_stub_functions(&module)?;
    nvvm_cleanup::replace_external_void_functions(&module)?;

    module.write_bitcode_to_path(file_path);

    Ok(())
}
