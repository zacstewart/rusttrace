#![feature(cstr_memory)]

extern crate libc;

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use libc::c_char;
use std::str;

#[derive(Debug)]
pub struct Usage {
    calls: HashMap<String, u32>,
    total: u32
}

impl Usage {
    fn new() -> Usage {
        Usage {
            calls: HashMap::new(),
            total: 0
        }
    }

    fn record(&mut self, method_name: String) {
        *self.calls.entry(method_name).or_insert(0) += 1;
        self.total += 1;
    }
}

#[derive(Debug)]
pub struct CallCount {
    count: u32,
    method_name: String
}

impl CallCount {
    fn new(method: &str, count: &u32) -> CallCount {
        CallCount {
            method_name: method.to_string(),
            count: *count
        }
    }

    fn to_c_struct(&self) -> CCallCount {
        let method = self.method_name.clone();
        let method = CString::new(method).unwrap();
        CCallCount {
            method_name: method.into_ptr(),
            count: self.count
        }
    }
}

#[repr(C)]
pub struct CCallCount {
    count: u32,
    method_name: *const c_char
}

#[repr(C)]
pub struct Report {
    length: u32,
    call_counts: *const CCallCount
}

#[no_mangle]
pub extern "C" fn new_usage() -> Box<Usage> {
    Box::new(Usage::new())
}

#[no_mangle]
pub extern "C" fn record(usage: &mut Usage,
                         event: *const c_char,
                         file: *const c_char,
                         line: u32,
                         id: *const c_char,
                         classname: *const c_char) {
    let event = unsafe { CStr::from_ptr(event) };
    let event = str::from_utf8(event.to_bytes()).unwrap();

    if event == "call" || event == "c-call" {
        let method_name = unsafe { CStr::from_ptr(id) };
        let method_name = str::from_utf8(method_name.to_bytes()).unwrap();
        let class_name = unsafe { CStr::from_ptr(classname) };
        let class_name = str::from_utf8(class_name.to_bytes()).unwrap();
        usage.record(class_name.to_string() + "#" + method_name)
    }
}

#[no_mangle]
pub extern "C" fn report(usage: &mut Usage) -> Report {
    let mut counts: Vec<CallCount> = usage.calls
        .iter()
        .map(|(method, count)| CallCount::new(method, count) )
        .collect();
    counts.sort_by(|a, b| a.count.cmp(&b.count).reverse());

    let c_counts: Vec<CCallCount> = counts
        .iter()
        .map(|cc| cc.to_c_struct())
        .collect();

    Report {
        length: c_counts.len() as u32,
        call_counts: c_counts.as_ptr()
    }
}
