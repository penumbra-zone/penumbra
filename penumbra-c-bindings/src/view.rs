// is exporting these directly in penumbra-c-bindings/lib.rs possible?
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn flip(a: bool) -> bool {
    !a
}
