#[path = "../../src/lib.rs"]
mod core;

#[no_mangle]
pub extern "C" fn allocate(len: u32) -> *mut u8 {
    let buffer = vec![0u8; len as usize].into_boxed_slice();
    Box::into_raw(buffer) as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn release(ptr: *mut u8, len: u32) {
    if !ptr.is_null() {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len as usize)));
    }
}

fn output(bytes: Vec<u8>) -> u64 {
    let len = bytes.len() as u64;
    let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;
    (len << 32) | ptr as u64
}

#[no_mangle]
pub extern "C" fn defaults() -> u64 {
    output(serde_json::to_vec(&core::Input::default()).unwrap())
}

#[no_mangle]
pub unsafe extern "C" fn calculate(ptr: *const u8, len: u32) -> u64 {
    if len > 16384 {
        return output(b"{\"error\":\"Input exceeds 16 KiB\"}".to_vec());
    }
    let bytes = std::slice::from_raw_parts(ptr, len as usize);
    let result = serde_json::from_slice::<core::Input>(bytes)
        .map_err(|e| e.to_string())
        .and_then(|input| core::calculate(&input));
    output(match result {
        Ok(value) => serde_json::to_vec(&value).unwrap(),
        Err(error) => serde_json::to_vec(&serde_json::json!({"error": error})).unwrap(),
    })
}
