# Tether Dataset Datasheet

## Synthetic Datasets (`data/synthetic/`)
- `sine_wave.bin`: 100,000 samples of 64-bit signed integers generated as `int(sin(i * 0.01) * 20000)`. Evaluates performance on smooth, continuously differentiable sinusoidal sensor waveforms.
- `linear_trend.bin`: 100,000 samples of 64-bit signed integers generated as `int(i * 3 + (i % 7))`. Tests the Order-3 FIR adaptive linear predictor under monotonic ramps with minor periodic ripple.
- `periodic_square.bin`: 100,000 samples of 64-bit signed integers alternating between +5000 and -5000 every 100 samples. Evaluates performance under sharp, discontinuous transitions and repeating pulse trains.
- `gaussian_noise.bin`: 100,000 samples of Gaussian random variables ($\mu=0, \sigma=1000$). Tests behavior on noisy accelerometer and biometric sensors.
- `adversarial_random.bin`: 100,000 samples of cryptographically uniform random 64-bit integers. Verifies that the raw fallback mechanism prevents negative compression or memory explosion on incompressible inputs.
- `float_temperature.bin`: 100,000 samples of IEEE 754 64-bit floating point numbers simulating slow environmental temperature drift sampled at 10 Hz.

## Real-World Datasets (`data/real_world/`)
- `iot_environmental.bin`: 100,000 samples of environmental temperature telemetry modeled after a Bosch BME280 sensor log over multi-day diurnal cycles.
- `wearable_imu.bin`: 100,000 samples of wearable 3-axis accelerometer sensor data recorded during human locomotion (walking cadence at ~2 Hz with micro-tremors).
- `industrial_power.bin`: 100,000 samples of smart grid electrical substation AC current telemetry with harmonic oscillations.

## Image Datasets (`data/images/`)
- `flat_ui.raw` / `.png`: 256x256 8-bit grayscale image representing flat UI components, menus, and gridlines.
- `synthetic_gradient.raw` / `.png`: 256x256 8-bit grayscale image with a smooth 2D planar diagonal gradient.
- `photographic_texture.raw` / `.png`: 256x256 8-bit grayscale image with continuous natural texture variation.
