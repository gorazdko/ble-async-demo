#![doc = include_str!("../../README.md")]
#![feature(type_alias_impl_trait)]
#![no_main]
#![no_std]


pub mod device_central;
pub mod ble;
pub mod button_handler;
pub mod device;
pub mod led_handler;
pub mod message;
pub mod uart_rx_handler;

use defmt_rtt as _; // global logger
use embassy_nrf as _; // memory layout
use panic_probe as _;


// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}


extern {
    fn hello() -> i32;
}


// defmt-test 0.3.0 has the limitation that this `#[tests]` attribute can only be used
// once within a crate. the module can be in any file but there can only be at most
// one `#[tests]` module in this library crate
// cargo test --lib
#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use defmt::assert;
    use super::*;

    #[test]
    fn it_works() {
        assert!(true)
    }

    #[test]
    fn test_hello() {
        assert!(unsafe {hello()} == 42);
    }
}
