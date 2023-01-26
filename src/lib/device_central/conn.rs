use nrf_softdevice::{
    Softdevice, raw,
};
use nrf_softdevice::ble::{central, Address, gatt_client, AddressType, Connection};
use core::{mem, slice, str};
use defmt::*;
use embassy_executor::Spawner;


// GATT server task. When there is a new connection, this passes the connection to conn_task.
#[embassy_executor::task]
pub async fn ble_central_scan(spawner: Spawner, sd: &'static Softdevice)  {
    let config = central::ScanConfig::default();

    loop{
        let res = central::scan(sd, &config, |params| unsafe {
            info!("AdvReport!");
            info!(
                "type: connectable={} scannable={} directed={} scan_response={} extended_pdu={} status={}",
                params.type_.connectable(),
                params.type_.scannable(),
                params.type_.directed(),
                params.type_.scan_response(),
                params.type_.extended_pdu(),
                params.type_.status()
            );
            info!(
                "addr: resolved={} type={} addr={:x}",
                params.peer_addr.addr_id_peer(),
                params.peer_addr.addr_type(),
                params.peer_addr.addr
            );
            let mut data = slice::from_raw_parts(params.data.p_data, params.data.len as usize);
            while data.len() != 0 {
                let len = data[0] as usize;
                if data.len() < len + 1 {
                    warn!("Advertisement data truncated?");
                    break;
                }
                if len < 1 {
                    warn!("Advertisement data malformed?");
                    break;
                }
                let key = data[1];
                let value = &data[2..len + 1];
                info!("value {}: {:x}", key, value);
                data = &data[len + 1..];

                if key == 9 {
                let name = str::from_utf8(value).unwrap();  //unsafe?

                    if name == "GorazdPeriph" {
                        info!("name {}. About to stop scanning...", name);
                        //return Some(0); // Have to return Some(), otherwise future will be pending, in progress
                        return Some(Address::from_raw(params.peer_addr));
                    }

                }
            }
            None
        })
        .await;
        info!("scan stop!");
        let _ret = unsafe { raw::sd_ble_gap_scan_stop() };

        // connect to the device TODO try connecting to 2 devices


        let address = unwrap!(res);
        //let addrs = &[&address];

        //let mut config = central::ConnectConfig::default();
        //config.scan_config.whitelist = Some(addrs);

        unwrap!(spawner.spawn(connection_task(sd, address)));
        info!("***END OF A TAKS");

        use embassy_time::{Duration, Timer};

        Timer::after(Duration::from_millis(60000)).await;

        info!("******END OF SLEEP");
}

}


//use crate::message::{ MESSAGE_BUS};

/// BLE connection task. Max 3 concurrent executions.
#[embassy_executor::task(pool_size = 2)]
async fn connection_task(
     sd: &'static Softdevice, address: Address)
    /*publisher: AppPublisher,
    subscriber: AppSubscriber, */
 {

    let addrs = &[&address];
    let mut config = central::ConnectConfig::default();
    config.scan_config.whitelist = Some(addrs);

    let conn = unwrap!(central::connect(sd, &config).await);
    info!("connected");

    let client : HeartRateserviceClient= unwrap!(gatt_client::discover(&conn).await);

    // Read
    let val = unwrap!(client.location_read().await);
    info!("read location: {}", val);

    // notifications:


    //let val = unwrap!(client.heart_rate_notify().await);
    //info!("read location: {}", val);


    // if i don't have a infinite loop here: the conection will be droped after the value is read
    // But what if I turn on notifications here? And await them? I can read the notifications for 10 seconds an then
    // drop the connection of I can rad the notifications indefinitely -> on connection drop everything will drop surely!!


    /*
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
        }
        ServerEvent::Button(ButtonServiceEvent::StateCccdWrite { notifications }) => {
            info!("Button notifications: {}", notifications);
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
    */
}


#[nrf_softdevice::gatt_client(uuid = "180D")]
struct HeartRateserviceClient {
    #[characteristic(uuid = "2a38", read)]
    location: u8,
    #[characteristic(uuid = "2a37", notify)]
    heart_rate : u16
}



// pub unsafe fn sd_ble_gatts_hvx(conn_handle: u16, p_hvx_params: *const ble_gatts_hvx_params_t) -> u32 {

/*
pub struct ble_gatts_hvx_params_t {
    #[doc = "< Characteristic Value Handle."]
    pub handle: u16,
    #[doc = "< Indication or Notification, see @ref BLE_GATT_HVX_TYPES."]
    pub type_: u8,
    #[doc = "< Offset within the attribute value."]
    pub offset: u16,
    #[doc = "< Length in bytes to be written, length in bytes written after return."]
    pub p_len: *mut u16,
    #[doc = "< Actual data content, use NULL to use the current attribute value."]
    pub p_data: *const u8,
}
*/