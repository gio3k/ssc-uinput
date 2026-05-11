//! ssc-uinput by Gianni S. <me@gio.blue>
//! https://github.com/gio3k/ssc-uinput

use evdev::uinput::VirtualDevice;
use evdev::{AbsInfo, AttributeSet};
use evdev::{AbsoluteAxisCode, PropType};
use std::sync::{Arc, Mutex};

pub mod sensors;

// ENV keys
const ENV_GYROSCOPE_MOUNT_MATRIX: &str = "SSCU_GYROSCOPE_MOUNT_MATRIX";
const ENV_ACCELEROMETER_MOUNT_MATRIX: &str = "SSCU_ACCELEROMETER_MOUNT_MATRIX";
const ENV_IMU_OUTPUT_SCALE: &str = "SSCU_IMU_OUTPUT_SCALE";

const GYROSCOPE_EVENT_RANGE: i32 = 0x8000;
const GYROSCOPE_UNITS_PER_ONE: f32 = 1024.0;
const GYROSCOPE_EVENT_RESOLUTION: f32 = (GYROSCOPE_EVENT_RANGE as f32) / GYROSCOPE_UNITS_PER_ONE;

const ACCELEROMETER_EVENT_RANGE: i32 = 0x8000;
const ACCELEROMETER_UNITS_PER_ONE: f32 = 1024.0;
const ACCELEROMETER_EVENT_RESOLUTION: f32 =
    (ACCELEROMETER_EVENT_RANGE as f32) / ACCELEROMETER_UNITS_PER_ONE;

fn get_mount_matrix(key: &str) -> [[f32; 3]; 3] {
    let identity = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    println!("Trying to parse mount matrix from key '{}'...", key);

    let value = match std::env::var(key) {
        Ok(v) => v,
        Err(e) => {
            println!(
                "Failed to get mount matrix from key '{}'! (error = {})",
                key, e
            );
            return identity;
        }
    };

    let vals: Vec<f32> = match value
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().parse().ok())
        .collect::<Option<Vec<_>>>()
    {
        Some(v) => v,
        None => {
            println!("Failed to parse the mount matrix for key '{}'", key);
            return identity;
        }
    };

    if vals.len() != 9 {
        identity
    } else {
        [
            [vals[0], vals[1], vals[2]],
            [vals[3], vals[4], vals[5]],
            [vals[6], vals[7], vals[8]],
        ]
    }
}

fn apply_matrix(m: &[[f32; 3]; 3], x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    (
        m[0][0] * x + m[0][1] * y + m[0][2] * z,
        m[1][0] * x + m[1][1] * y + m[1][2] * z,
        m[2][0] * x + m[2][1] * y + m[2][2] * z,
    )
}

// note
// InputPlumber scales all IMU inputs by 0.01f
// https://github.com/ShadowBlip/InputPlumber/blob/32b5edb0f2163897c1b849338c8bc57b1aa0d229/src/input/event/evdev/translator.rs#L504
// if using InputPlumber, the IMU OUTPUT env var should be 100.0f
fn get_imu_output_scale() -> f32 {
    const DEFAULT: f32 = 1.0;

    let value_str = match std::env::var(ENV_IMU_OUTPUT_SCALE) {
        Ok(v) => v,
        Err(_) => return DEFAULT,
    };

    let value_f32 = match value_str.parse::<f32>() {
        Ok(v) => v,
        Err(_) => {
            println!(
                "Failed to parse key '{}', it should be a float",
                ENV_IMU_OUTPUT_SCALE
            );
            return DEFAULT;
        }
    };

    if value_f32 == 0.0 {
        println!(
            "IMU output scale was set to 0.0! (using key '{}') As this doesn't make sense, it'll be set to 1.0",
            ENV_IMU_OUTPUT_SCALE
        );
        1.0
    } else {
        value_f32
    }
}

fn main() {
    let imu_output_scale = get_imu_output_scale();
    println!("Using IMU output scale: {}", imu_output_scale);

    let gyro_info = AbsInfo::new(
        0,
        -(GYROSCOPE_EVENT_RANGE as f32 * imu_output_scale) as i32,
        (GYROSCOPE_EVENT_RANGE as f32 * imu_output_scale) as i32,
        0,
        0,
        (GYROSCOPE_EVENT_RESOLUTION * imu_output_scale) as i32,
    );
    let accel_info = AbsInfo::new(
        0,
        -(ACCELEROMETER_EVENT_RANGE as f32 * imu_output_scale) as i32,
        (ACCELEROMETER_EVENT_RANGE as f32 * imu_output_scale) as i32,
        0,
        0,
        (ACCELEROMETER_EVENT_RESOLUTION * imu_output_scale) as i32,
    );

    let device = {
        let mut props = AttributeSet::<PropType>::new();
        props.insert(PropType::ACCELEROMETER);

        Arc::new(Mutex::new(
            VirtualDevice::builder()
                .unwrap()
                // note
                // InputPlumber only allows virtual sources when they pass the whitelist
                // to get around this, we can just use a name that's already in the whitelist (like this Sunshine one)
                .name("Sunshine gamepad (virtual) motion sensors")
                .with_properties(&props)
                .unwrap()
                .with_absolute_axis(&evdev::UinputAbsSetup::new(
                    AbsoluteAxisCode::ABS_RX,
                    gyro_info,
                ))
                .unwrap()
                .with_absolute_axis(&evdev::UinputAbsSetup::new(
                    AbsoluteAxisCode::ABS_RY,
                    gyro_info,
                ))
                .unwrap()
                .with_absolute_axis(&evdev::UinputAbsSetup::new(
                    AbsoluteAxisCode::ABS_RZ,
                    gyro_info,
                ))
                .unwrap()
                .with_absolute_axis(&evdev::UinputAbsSetup::new(
                    AbsoluteAxisCode::ABS_X,
                    accel_info,
                ))
                .unwrap()
                .with_absolute_axis(&evdev::UinputAbsSetup::new(
                    AbsoluteAxisCode::ABS_Y,
                    accel_info,
                ))
                .unwrap()
                .with_absolute_axis(&evdev::UinputAbsSetup::new(
                    AbsoluteAxisCode::ABS_Z,
                    accel_info,
                ))
                .unwrap()
                .build()
                .unwrap(),
        ))
    };

    let _gyroscope = {
        println!("Preparing the gyroscope...");
        let matrix = get_mount_matrix(ENV_GYROSCOPE_MOUNT_MATRIX);
        let device = Arc::clone(&device);

        println!("Opening / creating the gyroscope...");
        sensors::Gyroscope::try_create(move |x, y, z| {
            let mut device = device.lock().unwrap();

            let (x, y, z) = apply_matrix(&matrix, x, y, z);

            device
                .emit(&[
                    evdev::InputEvent::new(
                        evdev::EventType::ABSOLUTE.0,
                        AbsoluteAxisCode::ABS_RX.0,
                        (x.to_degrees() * GYROSCOPE_EVENT_RESOLUTION * imu_output_scale) as i32,
                    ),
                    evdev::InputEvent::new(
                        evdev::EventType::ABSOLUTE.0,
                        AbsoluteAxisCode::ABS_RY.0,
                        (y.to_degrees() * GYROSCOPE_EVENT_RESOLUTION * imu_output_scale) as i32,
                    ),
                    evdev::InputEvent::new(
                        evdev::EventType::ABSOLUTE.0,
                        AbsoluteAxisCode::ABS_RZ.0,
                        (z.to_degrees() * GYROSCOPE_EVENT_RESOLUTION * imu_output_scale) as i32,
                    ),
                ])
                .expect("Failed to emit an event (virtual gyroscope)")
        })
        .expect("Failed to open the gyroscope!")
    };

    let _accelerometer = {
        println!("Preparing the accelerometer...");
        let matrix = get_mount_matrix(ENV_ACCELEROMETER_MOUNT_MATRIX);
        let device = Arc::clone(&device);

        sensors::Accelerometer::try_create(move |x, y, z| {
            let mut device = device.lock().unwrap();

            let (x, y, z) = apply_matrix(&matrix, x, y, z);

            device
                .emit(&[
                    evdev::InputEvent::new(
                        evdev::EventType::ABSOLUTE.0,
                        AbsoluteAxisCode::ABS_X.0,
                        (x * ACCELEROMETER_EVENT_RESOLUTION) as i32,
                    ),
                    evdev::InputEvent::new(
                        evdev::EventType::ABSOLUTE.0,
                        AbsoluteAxisCode::ABS_Y.0,
                        (y * ACCELEROMETER_EVENT_RESOLUTION) as i32,
                    ),
                    evdev::InputEvent::new(
                        evdev::EventType::ABSOLUTE.0,
                        AbsoluteAxisCode::ABS_Z.0,
                        (z * ACCELEROMETER_EVENT_RESOLUTION) as i32,
                    ),
                ])
                .expect("Failed to emit an event (virtual accelerometer)")
        })
        .expect("Failed to open the accelerometer!")
    };

    println!("Sources ready, starting main loop...");
    let main_loop = glib::MainLoop::new(None, false);
    main_loop.run();
}
