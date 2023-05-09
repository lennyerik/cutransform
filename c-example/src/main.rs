use cust::error::CudaError;
use cust::prelude::*;

include!(concat!(env!("OUT_DIR"), "/kernel.rs"));

fn main() -> Result<(), CudaError> {
    println!("{}", env!("OUT_DIR"));

    // We have to assign the context to an unused variable, because the
    // compiler optimises the function call out otherwise
    let _ctx = cust::quick_init()?;

    let module = Module::from_cubin(KERNEL_COMPILED, &[])?;
    let stream = Stream::new(StreamFlags::NON_BLOCKING, None)?;

    let arr = DeviceBuffer::from_slice(&[0u32; 512])?;
    let arr_ptr = arr.as_device_ptr();

    unsafe {
        launch!(module.kernel<<<1, 512, 0, stream>>>(arr_ptr))?;
    }

    stream.synchronize()?;

    let mut host_arr = [0u32; 512];
    arr.copy_to(&mut host_arr)?;

    for (i, value) in host_arr.iter().enumerate() {
        println!("arr[{}] = {}", i, value);
    }

    Ok(())
}
