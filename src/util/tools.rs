use std::{
    ffi::{c_char, CStr, CString},
    ptr,
};

pub fn vk_to_string(raw_string_array: &[c_char]) -> String {
    let raw_string = unsafe {
        let ptr = raw_string_array.as_ptr();
        CStr::from_ptr(ptr)
    };

    raw_string
        .to_str()
        .expect("Failed to convert to string")
        .to_owned()
}

pub fn vec_string_to_pp(v: &Vec<String>) -> *const *const i8 {
    let cstr_ext_names: Vec<_> = v
        .iter()
        .map(|x| CString::new(x.as_str()).unwrap())
        .collect();
    let mut pp_ext_names: Vec<_> = cstr_ext_names.iter().map(|x| x.as_ptr()).collect();
    pp_ext_names.push(ptr::null());

    let box_ptr = pp_ext_names.into_boxed_slice();
    let ptr = box_ptr.as_ptr();
    Box::leak(box_ptr);

    ptr
}
