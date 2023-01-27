#![no_std]
#![no_main]

use ble_async_demo as _; // memory layout + panic handler

// See https://crates.io/crates/defmt-test/0.3.0 for more documentation (e.g. about the 'state'
// feature)
// cargo test --test integration
#[defmt_test::tests]
mod tests {
    use defmt::assert;

    #[test]
    fn it_works() {
        assert!(true)
    }
}
