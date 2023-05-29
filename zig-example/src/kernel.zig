extern fn threadIdxX() i32;

export fn kernel(arr: [*]u32) callconv(.C) void {
    arr[@intCast(usize, threadIdxX())] = 123;
}

// Override the default entrypoint
pub fn _start() callconv(.Naked) void {}

