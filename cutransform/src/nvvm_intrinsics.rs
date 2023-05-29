use inkwell::{module::Module, types::FunctionType};

pub fn replace_stub_functions(module: &Module) -> Result<(), String> {
    let ctx = module.get_context();
    let i32fntype = ctx.i32_type().fn_type(&[], false);
    let voidfntype = ctx.void_type().fn_type(&[], false);

    // Replace ThreadIdx functions
    replace_stub_fn_with_intrinsic(
        module,
        "threadIdxX",
        "llvm.nvvm.read.ptx.sreg.tid.x",
        i32fntype,
    )?;
    replace_stub_fn_with_intrinsic(
        module,
        "threadIdxY",
        "llvm.nvvm.read.ptx.sreg.tid.y",
        i32fntype,
    )?;
    replace_stub_fn_with_intrinsic(
        module,
        "threadIdxZ",
        "llvm.nvvm.read.ptx.sreg.tid.z",
        i32fntype,
    )?;

    // Replace GroupIdx functions
    replace_stub_fn_with_intrinsic(
        module,
        "blockIdxX",
        "llvm.nvvm.read.ptx.sreg.ctaid.x",
        i32fntype,
    )?;
    replace_stub_fn_with_intrinsic(
        module,
        "blockIdxY",
        "llvm.nvvm.read.ptx.sreg.ctaid.y",
        i32fntype,
    )?;
    replace_stub_fn_with_intrinsic(
        module,
        "blockIdxZ",
        "llvm.nvvm.read.ptx.sreg.ctaid.z",
        i32fntype,
    )?;

    // Replace blockDim functions
    replace_stub_fn_with_intrinsic(
        module,
        "blockDimX",
        "llvm.nvvm.read.ptx.sreg.ntid.x",
        i32fntype,
    )?;
    replace_stub_fn_with_intrinsic(
        module,
        "blockDimY",
        "llvm.nvvm.read.ptx.sreg.ntid.y",
        i32fntype,
    )?;
    replace_stub_fn_with_intrinsic(
        module,
        "blockDimZ",
        "llvm.nvvm.read.ptx.sreg.ntid.z",
        i32fntype,
    )?;

    // Replace gridDim functions
    replace_stub_fn_with_intrinsic(
        module,
        "gridDimX",
        "llvm.nvvm.read.ptx.sreg.nctaid.x",
        i32fntype,
    )?;
    replace_stub_fn_with_intrinsic(
        module,
        "gridDimY",
        "llvm.nvvm.read.ptx.sreg.nctaid.y",
        i32fntype,
    )?;
    replace_stub_fn_with_intrinsic(
        module,
        "gridDimZ",
        "llvm.nvvm.read.ptx.sreg.nctaid.z",
        i32fntype,
    )?;

    // Replace warpSize function
    replace_stub_fn_with_intrinsic(
        module,
        "warpSize",
        "llvm.nvvm.read.ptx.sreg.warpsize",
        i32fntype,
    )?;

    // Replace __syncthreads function
    replace_stub_fn_with_intrinsic(module, "__syncthreads", "llvm.nvvm.barrier0", voidfntype)?;

    Ok(())
}

fn replace_stub_fn_with_intrinsic<'a>(
    module: &Module<'a>,
    stub_fn_name: &str,
    instrinsic_name: &str,
    fn_type: FunctionType<'a>,
) -> Result<(), String> {
    if let Some(func) = module.get_function(stub_fn_name) {
        let intrinsic_func = module
            .get_function(instrinsic_name)
            .unwrap_or_else(|| module.add_function(instrinsic_name, fn_type, None));

        // Check the function type
        let actual_type = func.get_type();
        if actual_type != fn_type {
            return Err(format!(
                "Intrinsic stub function `{}` has wrong type! Expected {}, got {}.",
                stub_fn_name,
                fn_type
                    .print_to_string()
                    .to_str()
                    .unwrap_or("<Invalid UTF8>"),
                actual_type
                    .print_to_string()
                    .to_str()
                    .unwrap_or("<Invalid UTF8>")
            ));
        }

        func.replace_all_uses_with(intrinsic_func);
    }

    Ok(())
}
