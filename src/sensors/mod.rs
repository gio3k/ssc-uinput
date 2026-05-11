//! ssc-uinput by Gianni S. <me@gio.blue>
//! https://github.com/gio3k/ssc-uinput

mod raw;
use glib::Error;
use glib::translate::FromGlibPtrFull;
use std::time::Duration;

pub struct Gyroscope {
    _device: raw::SscObject,
}

impl Gyroscope {
    pub fn try_create<F>(callback: F, timeout: Duration) -> Result<Self, Error>
    where
        F: FnMut(f32, f32, f32) + 'static,
    {
        let cancellable = raw::Cancellable::cancel_after(timeout);

        let mut err: *mut raw::GError = std::ptr::null_mut();
        let ptr = unsafe {
            let ptr = raw::ssc_sensor_gyroscope_new_sync(cancellable.ptr(), &mut err);

            if !err.is_null() {
                return Err(Error::from_glib_full(err as *mut glib::ffi::GError));
            }

            ptr
        };

        let device = raw::SscObject::from(ptr as *mut raw::_GObject);
        match device.connect_measurement_signal(callback) {
            Ok(_) => println!("Connected measurement signal for {:p}", ptr),
            Err(e) => return Err(e),
        }

        unsafe {
            let result = raw::ssc_sensor_gyroscope_open_sync(
                ptr as *mut raw::_SSCSensorGyroscope,
                cancellable.ptr(),
                &mut err,
            );

            if !err.is_null() || result != 1 {
                return Err(Error::from_glib_full(err as *mut glib::ffi::GError));
            }
        }

        Ok(Self { _device: device })
    }
}

pub struct Accelerometer {
    _device: raw::SscObject,
}

impl Accelerometer {
    pub fn try_create<F>(callback: F, timeout: Duration) -> Result<Self, Error>
    where
        F: FnMut(f32, f32, f32) + 'static,
    {
        let cancellable = raw::Cancellable::cancel_after(timeout);

        let mut err: *mut raw::GError = std::ptr::null_mut();
        let ptr = unsafe {
            let ptr = raw::ssc_sensor_accelerometer_new_sync(cancellable.ptr(), &mut err);

            if !err.is_null() {
                return Err(Error::from_glib_full(err as *mut glib::ffi::GError));
            }

            ptr
        };

        let device = raw::SscObject::from(ptr as *mut raw::_GObject);
        match device.connect_measurement_signal(callback) {
            Ok(_) => println!("Connected measurement signal for {:p}", ptr),
            Err(e) => return Err(e),
        }

        unsafe {
            let result = raw::ssc_sensor_accelerometer_open_sync(
                ptr as *mut raw::_SSCSensorAccelerometer,
                cancellable.ptr(),
                &mut err,
            );

            if !err.is_null() || result != 1 {
                return Err(Error::from_glib_full(err as *mut glib::ffi::GError));
            }
        }

        Ok(Self { _device: device })
    }
}
