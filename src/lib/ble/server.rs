//! BLE Server Config and Tasks
use super::services::*;
use crate::message::{AppEvent, AppPublisher, AppSubscriber, PinState, MESSAGE_BUS};
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::block_on;
use futures::{
    future::{select, Either},
    pin_mut,
};
use nrf_softdevice::{
    ble::{gatt_server, peripheral, Connection},
    raw::{
        BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE, BLE_GAP_AD_TYPE_128BIT_SERVICE_UUID_COMPLETE,
        BLE_GAP_AD_TYPE_COMPLETE_LOCAL_NAME, BLE_GAP_AD_TYPE_FLAGS,
    },
    Softdevice, Flash
};
use static_cell::StaticCell;

/// BLE advertising data
#[rustfmt::skip]
const ADV_DATA: &[u8; 14] =
    &[
        0x02, BLE_GAP_AD_TYPE_FLAGS as u8, BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x0a, BLE_GAP_AD_TYPE_COMPLETE_LOCAL_NAME as u8, b'H', b'e', b'l', b'l', b'o', b'R', b'u', b's', b't'
    ];

/// BLE scan response data
#[rustfmt::skip]
const SCAN_RESPONSE_DATA: &[u8; 18] = &[
    // AD length
    0x11, 
    // AD type
    BLE_GAP_AD_TYPE_128BIT_SERVICE_UUID_COMPLETE as u8,
    // AD data
    // UART service UUID: 9e7312e1-2354-11eb-9f10-fbc30a62cf38. This has to be sent in little endian order.
    0x38, 0xcf, 0x62, 0x0a, 0xc3, 0xfb, 0x10, 0x9f, 0xeb, 0x11, 0x54, 0x23, 0xe1, 0x12, 0x73, 0x9E,
];

/// BLE GATT server
#[nrf_softdevice::gatt_server]
pub struct Server {
    /// UART service
    pub uart: UartService,
    /// LED service
    pub led: LedService,
    /// Button service
    pub button: ButtonService,
}

/// GATT server task. When there is a new connection, this passes the connection to conn_task.
#[embassy_executor::task]
pub async fn ble_server_task(spawner: Spawner, server: Server, sd: &'static Softdevice) {
    static SERVER: StaticCell<Server> = StaticCell::new();
    let server: &'static mut Server = SERVER.init(server);

    info!("Bluetooth ON!");

    let config = peripheral::Config::default();
    let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
        adv_data: ADV_DATA,
        scan_data: SCAN_RESPONSE_DATA,
    };

    loop {
        match peripheral::advertise_connectable(sd, adv, &config).await {
            Ok(conn) => {
                let publisher = unwrap!(MESSAGE_BUS.publisher());
                let subscriber = unwrap!(MESSAGE_BUS.subscriber());
                unwrap!(spawner.spawn(conn_task(server, conn, publisher, subscriber)));
            }
            Err(e) => error!("{:?}", e),
        }
    }
}

/// BLE connection task. Max 3 concurrent executions.
#[embassy_executor::task(pool_size = 3)]
async fn conn_task(
    server: &'static Server,
    conn: Connection,
    publisher: AppPublisher,
    subscriber: AppSubscriber,
) {
    let subscribe_future = subscriber_task(server, &conn, subscriber);
    let gatt_future = gatt_server::run(&conn, server, |e| match e {
        ServerEvent::Uart(UartServiceEvent::BytesWrite(vec)) => {
            block_on(publisher.publish(AppEvent::BleBytesWritten(vec)));
        }
        ServerEvent::Uart(UartServiceEvent::BytesCccdWrite { notifications }) => {
            info!("Uart notifications: {}", notifications);
        }
        ServerEvent::Led(LedServiceEvent::StateWrite(requested_state)) => {
            block_on(publisher.publish(AppEvent::Led(PinState::from(requested_state))));
            // write led state also to flash:
            block_on(publisher.publish(AppEvent::LedFlashWrite(PinState::from(requested_state))));
        }
        ServerEvent::Button(ButtonServiceEvent::StateCccdWrite { notifications }) => {
            info!("Button notifications: {}", notifications);

            block_on(publisher.publish(AppEvent::LedFlashReq));
            //TODO: should wait for response being published from the flash task (aslo todo)
            // and put the reposnse in to button characteristic
        }
    });

    pin_mut!(subscribe_future);
    pin_mut!(gatt_future);

    match select(subscribe_future, gatt_future).await {
        Either::Left((_, _)) => {
            info!("Notification service encountered an error and stopped!")
        }
        Either::Right((res, _)) => {
            if let Err(e) = res {
                info!("gatt_server run exited with error: {:?}", e);
            }
        }
    };
}

/// Responds to incoming messages.
async fn subscriber_task<'a>(
    server: &'a Server,
    conn: &'a Connection,
    mut subscriber: AppSubscriber,
) {
    loop {
        match subscriber.next_message_pure().await {
            AppEvent::UartRxWritten(bytes) => {
                if let Err(e) = server.uart.bytes_notify(conn, bytes) {
                    error!("{:?}", e);
                }
            }
            AppEvent::Button(state) => {
                if let Err(e) = server.button.state_notify(conn, state.into()) {
                    error!("{:?}", e);
                }
            }
            _ => (),
        }
    }
}



use embedded_storage_async::nor_flash::*;



// mogoc flash_task_run(), da ga klices v main brez argumetnov?
// connection has to be sent probably over the pubsub channel not as a function variable
#[embassy_executor::task(pool_size = 1)]
pub async fn flash_task(/*server: &'a Server,*/  /*conn: &'a Connection, */ sd: &'static Softdevice) {


    let  publisher = unwrap!(MESSAGE_BUS.publisher());
    let mut subscriber = unwrap!(MESSAGE_BUS.subscriber());


    let f = Flash::take(sd);
    pin_mut!(f);


    loop {

        match subscriber.next_message_pure().await { 
            AppEvent::LedFlashWrite(state) => {
                // event is genreated by led service
                info!("+++starting flash erase");
                unwrap!(f.as_mut().erase(0x80000, 0x81000).await);

                let val : bool = state.into();  // dumb casting
                let val = if val == true{ 1 as u8} else { 0 as u8};

                info!("++++starting flash write");  // write 4 bytes in order for the chunk to be aligned
                unwrap!(f.as_mut().write(0x80000, &[val, 0 , 0 ,0]).await);


            }

            // TODO
            AppEvent::LedFlashReq => { 
                info!("read flash..");

                let mut val : [u8; 4] = [0, 0, 0 ,0];
                
                unwrap!(f.read(0x80000, &mut val).await);

                info!("read flash {:?}", val);

                /* 
                let val : bool = if val[0] == 0 {false} else {true};

                if let Err(e) = server.button.state_notify(conn, val) {
                    error!("{:?}", e);
                }
                */
            }
     
            _ => (),
        }

    }

}
