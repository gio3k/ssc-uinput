//! ssc-uinput by Gianni S. <me@gio.blue>
//! https://github.com/gio3k/ssc-uinput

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use glib::Error;
use std::{os::raw::c_void, time::Duration};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

unsafe extern "C" fn callback_measurement_trampoline(
    _ptr: *mut GObject,
    x: gfloat,
    y: gfloat,
    z: gfloat,
    _user_data: gpointer,
) {
    let callback = unsafe { &mut *(_user_data as *mut Box<dyn FnMut(f32, f32, f32)>) };
    callback(x, y, z);
}

unsafe extern "C" fn callback_destroy(data: gpointer, _: *mut GClosure) {
    drop(unsafe { Box::from_raw(data as *mut Box<dyn FnMut(f32, f32, f32)>) });
}

pub(crate) struct SscObject {
    pub ptr: *mut _GObject,
}

impl SscObject {
    pub fn from(ptr: *mut _GObject) -> Self {
        Self { ptr }
    }

    pub fn connect_measurement_signal<F>(&self, callback: F) -> Result<(), Error>
    where
        F: FnMut(f32, f32, f32) + 'static,
    {
        unsafe {
            let user_data: Box<Box<dyn FnMut(f32, f32, f32)>> = Box::new(Box::new(callback));
            let user_data_ptr = Box::into_raw(user_data) as gpointer;

            let result = g_signal_connect_data(
                self.ptr as *mut _,
                b"measurement\0".as_ptr() as *const _,
                Some(std::mem::transmute(
                    callback_measurement_trampoline as *const (),
                )),
                user_data_ptr,
                Some(callback_destroy),
                0,
            );

            // todo: what's the success result? 0 or 1?
            // note: doesn't seem to be either!
            println!("connect_measurement_signal: result = {}", result);

            Ok(())
        }
    }
}

impl Drop for SscObject {
    fn drop(&mut self) {
        unsafe {
            println!("dropping GObject ref: {:p}", self.ptr);
            g_object_unref(self.ptr as *mut _);
        }
    }
}

pub struct Cancellable {
    _ptr: *mut _GCancellable,
}

impl Cancellable {
    fn new() -> Self {
        Self {
            _ptr: unsafe { g_cancellable_new() },
        }
    }

    pub fn ptr(&self) -> *mut _GCancellable {
        self._ptr
    }

    pub fn cancel_after(timeout: Duration) -> Self {
        let this = Self::new();
        let this_ptr_u64 = this.ptr() as ulong;
        let this_ptr = this.ptr() as *mut c_void;

        unsafe { g_object_ref(this_ptr) };

        std::thread::spawn(move || {
            std::thread::sleep(timeout);
            unsafe {
                let this_raw = this_ptr_u64 as *mut _GCancellable;
                g_cancellable_cancel(this_raw);
                g_object_unref(this_raw as *mut c_void);
            }
        });

        this
    }
}

impl Drop for Cancellable {
    fn drop(&mut self) {
        unsafe { g_object_unref(self.ptr() as gpointer) };
    }
}
