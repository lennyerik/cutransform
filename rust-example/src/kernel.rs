#![no_std]

extern "C" {
    fn threadIdxX() -> u32;
    // fn threadIdxY() -> u32;
    // fn threadIdxZ() -> u32;
}

#[no_mangle]
pub extern "C" fn kernel(arr: *mut u32) {
    unsafe {
        let idx = threadIdxX() as usize;
        *arr.add(idx) = 123;
    }
}
