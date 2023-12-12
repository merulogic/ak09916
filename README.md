<!--
SPDX-FileCopyrightText: Joonas Javanainen <joonas.javanainen@gmail.com>

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# `ak09916`

Rust `embedded-hal` and `embedded-hal-async` drivers for the AKM AK09916 3-axis I²C magnetometer.

Example (blocking API):

```rust
use ak09916::{blocking::Ak09916, Mode, WhoIAm};
use defmt::info;
use embedded_hal::{delay::DelayNs, i2c::I2c};

fn example<I: I2c, D: DelayNs>(i2c: I, delay: D) -> Result<(), I::Error> {
    let mut ak09916 = Ak09916::new(i2c, delay);

    // optional: check who I am (WIA) information
    let wia = ak09916.who_i_am()?;
    if wia != WhoIAm::AK09916 {
      // try again or fail, depending on your use case
    }

    // optional: do a self-test
    let test_result = ak09916.self_test()?;
    if !test_result.is_valid {
      // try again or fail, depending on your use case
    }

    ak09916.switch_mode(Mode::Continuous10Hz)?;
    loop {
        // In continuous 10 Hz measurement mode we get a measurement every 100ms, so the poll
        // interval needs to be much less than that or we might miss measurements
        const POLL_INTERVAL_US: u32 = 10_000; // 10 ms

        let measurement = ak09916.poll_measurement(POLL_INTERVAL_US)?;
        if measurement.overflow() {
            info!("Magnetic sensor overflow: data is not valid");
        }
        if measurement.overrun() {
            info!("Data overrun: at least one measurement was missed");
        }
        info!("X-axis: {} nT", measurement.x_nanoteslas());
        info!("Y-axis: {} nT", measurement.y_nanoteslas());
        info!("Z-axis: {} nT", measurement.z_nanoteslas());
    }
}
```

The asynchronous API uses async functions for most driver operations but is otherwise identical.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSES/Apache-2.0.txt](LICENSES/Apache-2.0.txt) or <https://opensource.org/license/apache-2-0/>)
 * MIT license ([LICENSES/MIT.txt](LICENSES/MIT.txt) or <https://opensource.org/licenses/MIT/>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
