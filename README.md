# ssc-uinput

Uses evdev to create a virtual IMU from [libssc](https://codeberg.org/DylanVanAssche/libssc) data

Has some quirks in order to work with InputPlumber (like pretending to be Sunshine's virtual input device)

## Config

Everything is controlled by environment variable currently

```
SSCU_GYROSCOPE_MOUNT_MATRIX
SSCU_ACCELEROMETER_MOUNT_MATRIX
SSCU_IMU_OUTPUT_SCALE
```

### SSCU_\*_MOUNT_MATRIX 

String of 9 float values, comma delimited

Example: `export SSCU_GYROSCOPE_MOUNT_MATRIX="1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0"`

### SSCU_IMU_OUTPUT_SCALE (default 1.0)

InputPlumber scales all incoming IMU data by 0.01f (see [here](https://github.com/ShadowBlip/InputPlumber/blob/32b5edb0f2163897c1b849338c8bc57b1aa0d229/src/input/event/evdev/translator.rs#L504))

To get around this, use the output scale env var

Example: `export SSCU_IMU_OUTPUT_SCALE=100.0`

## Usage

This tool is dependent on [hexagonrpcd](https://github.com/linux-msm/hexagonrpc) and [libssc](https://codeberg.org/DylanVanAssche/libssc)

Try to get iio-sensor-proxy working first - if that runs on your device, you should be able to run this

```
# run it in the background
./ssc-uinput &

# check evtest, should see the device
evtest
```

## Compiling

This is a Rust project, you should be able to `cargo build` normally

Keep in mind that this tool uses bindgen to generate Rust bindings for libssc

This is pretty host specific, but this is what I use for Rocknix:

```
export RUSTFLAGS="-C link-arg=-lssc"

export BINDGEN_EXTRA_CLANG_ARGS="--sysroot=${SYSROOT_PREFIX} --target=${TARGET_NAME} -I${SYSROOT_PREFIX}/usr/include/libssc -I${SYSROOT_PREFIX}/usr/include/glib-2.0 -I${SYSROOT_PREFIX}/usr/lib/glib-2.0/include -I${SYSROOT_PREFIX}/usr/include/glib-2.0/glib/**"
```



