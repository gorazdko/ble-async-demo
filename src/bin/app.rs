#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]


use nrf_softdevice::raw::ble_gatts_hvx_params_t;
use nrfxlib_sys::ocrypto_hmac_sha256_init;
use nrfxlib_sys::ocrypto_hmac_sha256_ctx;

use nrfxlib_sys as sys;

use ble_async_demo::{
    self as _,
    ble::{sd, server},
    button_handler,
    device::Board,
    led_handler,
    message::{self, AppEvent, PinState},
    uart_rx_handler,
    device_central::conn
};


use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::interrupt::Priority;

//int hello();
//int mydelay();
//void delay0(void (*rust_delay)());


// export a C-compatible function: https://anssi-fr.github.io/rust-guide/07_ffi.html
#[no_mangle]
unsafe extern "C" fn rust_delay() {
  info!("IT WORKS!!!");
}


extern {
    fn hello() -> i32;
    // read: https://rust-lang.github.io/unsafe-code-guidelines/layout/function-pointers.html
    fn delay0(f: unsafe extern fn());
}

pub fn call_clib() {
    unsafe {
        
        let val = hello();
        println!("calue from c library: {:?}", val);

        delay0(rust_delay);

    }
}





#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Async + SoftDevice Demo");

    // Configure peripherals
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    let p = embassy_nrf::init(config);


    call_clib();
    


/*

#[doc = "  Address information."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct nrf_addrinfo {
    #[doc = " Input flags."]
    pub ai_flags: ctypes::c_int,
    #[doc = " Address family of the socket."]
    pub ai_family: ctypes::c_int,
    #[doc = " Socket type."]
    pub ai_socktype: ctypes::c_int,
    #[doc = " Protocol of the socket."]
    pub ai_protocol: ctypes::c_int,
    #[doc = " Length of the socket address."]
    pub ai_addrlen: nrf_socklen_t,
    #[doc = " Address of the socket."]
    pub ai_addr: *mut nrf_sockaddr,
    #[doc = " Canonical name of service location."]
    pub ai_canonname: *mut ctypes::c_char,
    #[doc = " Pointer to next in list."]
    pub ai_next: *mut nrf_addrinfo,
}


extern "C" {
    #[doc = "  Get address information.\n\n @details\n See <a href=\"http://pubs.opengroup.org/onlinepubs/9699919799/functions/getaddrinfo.html\">\n POSIX.1-2017 article</a> for normative description.\n\n In addition, the function shall return -1 and set the following errno:\n [NRF_ESHUTDOWN] Modem was shut down."]
    pub fn nrf_getaddrinfo(
        nodename: *const ctypes::c_char,
        servname: *const ctypes::c_char,
        hints: *const nrf_addrinfo,
        res: *mut *mut nrf_addrinfo,
    ) -> ctypes::c_int;
}


*/



// TESTING OUT NRFXLIB-SYS:
let hints = sys::nrf_addrinfo {
    ai_flags: 0,
    ai_family: sys::NRF_AF_INET as i32,
    ai_socktype: sys::NRF_SOCK_DGRAM as i32,
    ai_protocol: 0,
    ai_addrlen: 0,
    ai_addr: core::ptr::null_mut(),
    ai_canonname: core::ptr::null_mut(),
    ai_next: core::ptr::null_mut(),
};

/* 
let mut result;
let mut hostname_smallstring: heapless::String<64> = heapless::String::new();

let mut output_ptr: *mut sys::nrf_addrinfo = core::ptr::null_mut();
		result = unsafe {
			sys::nrf_getaddrinfo(
				// hostname
				hostname_smallstring.as_ptr(),
				// service
				core::ptr::null(),
				// hints
				&hints,
				// output pointer
				&mut output_ptr,
			)
		};
*/


/*
extern "C" {
    #[doc = " SHA-1 hash.\n\n The SHA-1 hash of a given input message  * `in` -  is computed and put into  * `r` - .\n\n * `r` - Generated hash.\n * `in` - Input data.\n * `in_len` - Length of  * `in` - ."]
    pub fn ocrypto_sha1(r: *mut u8, in_: *const u8, in_len: usize);
}

*/

    let buf2: &mut [u8] = &mut[0; 20];
    let ptr2 = buf2.as_mut_ptr();

    let buf: &[u8] = &[97,97,97,97];  // 'a', 'a', 'a', 'a' or in hex 61616161
    let length = buf.len();
    let ptr = buf.as_ptr();


    // function prototype found in /home/gorazd/Projects/ble-async-demo/target/thumbv7em-none-eabihf/debug/build/nrfxlib-sys-849014581ba04543/out/bindings.rs
    let res : &[u8] = unsafe {sys::ocrypto_sha1(ptr2 as *mut _, ptr as *const _, length as usize);
        core::slice::from_raw_parts_mut(ptr2, buf2.len() as usize)
    };
    info!("result: {:#02x}", res);

    // result is correct result: [0x70, 0xc8, 0x81, 0xd4, 0xa2, 0x69, 0x84, 0xdd, 0xce, 0x79, 0x5f, 0x6f, 0x71, 0x81, 0x7c, 0x9c, 0xf4, 0x48, 0xe, 0x79]
    // validated with 61616161 in https://emn178.github.io/online-tools/sha1.html // 70c881d4a26984ddce795f6f71817c9cf4480e79


    // Initialize board with peripherals
    let board = Board::init(p);

    // Run LED indicator task
    unwrap!(spawner.spawn(led_handler::run(board.led3)));

    // BLE controllable LED
    let mut ble_led = board.led4;

    // Messaging: Create Publishers and Subscribers
    let mut subscriber = unwrap!(message::MESSAGE_BUS.subscriber());
    let publisher_1 = unwrap!(message::MESSAGE_BUS.publisher());
    let publisher_2 = unwrap!(message::MESSAGE_BUS.publisher());

    // Enable SoftDevice
    let sd = nrf_softdevice::Softdevice::enable(&sd::softdevice_config());

    // Create BLE GATT server
    let server = unwrap!(server::Server::new(sd));

    // Run SoftDevice task
    unwrap!(spawner.spawn(sd::softdevice_task(sd)));

    // Run BLE server task
    unwrap!(spawner.spawn(server::ble_server_task(spawner, server, sd)));

    // Run Button task
    unwrap!(spawner.spawn(button_handler::run(board.button1, publisher_1)));

    // UART device: split to tx and rx
    let (mut tx, rx) = board.uart.split();

    // Run UART RX task
    unwrap!(spawner.spawn(uart_rx_handler::run(rx, publisher_2)));


    // Create BLE central device to connect and read some heart rate characteristic every 10 seconds:
    unwrap!(spawner.spawn(conn::ble_central_scan(spawner, sd)));

    // Create task for writing and reading (wip) to flash
    unwrap!(spawner.spawn(server::flash_task(sd)));

    //info!("Going into loop..");

    // Wait for a message...
    loop {
        match subscriber.next_message_pure().await {
            AppEvent::Led(pin_state) => match pin_state {
                PinState::High => ble_led.set_high(),
                PinState::Low => ble_led.set_low(),
            },
            AppEvent::BleBytesWritten(data) => {
                // Send the received data through UART TX
                if let Err(e) = tx.write(&data).await {
                    error!("{:?}", e);
                }
            }
            _ => (),
        }
    }
}





