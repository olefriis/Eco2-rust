extern crate btleplug;

use btleplug::api::{Central, Characteristic, Peripheral};
use std::thread;
use std::time::{Duration, SystemTime};
use std::collections::{HashMap, HashSet};
use std::io;
use btleplug::{Result, Error};

#[cfg(target_os = "linux")]
use btleplug::bluez::{adapter::ConnectedAdapter, manager::Manager};
#[cfg(target_os = "macos")]
use btleplug::corebluetooth::{adapter::Adapter, manager::Manager};
#[cfg(target_os = "windows")]
use btleplug::winrtble::{adapter::Adapter, manager::Manager};

const PIN_CODE_CHARACTERISTIC: &str = "10:02:00:01:27:49:00:01:00:00:00:80:5F:9B:04:2F";
pub const BATTERY_LEVEL: &str = "00:00:2A:19:00:00:10:00:80:00:00:80:5F:9B:34:FB";
pub const SECRET_KEY: &str = "10:02:00:0B:27:49:00:01:00:00:00:80:5F:9B:04:2F";
pub const DEVICE_NAME: &str = "10:02:00:06:27:49:00:01:00:00:00:80:5F:9B:04:2F";
pub const TEMPERATURE: &str = "10:02:00:05:27:49:00:01:00:00:00:80:5F:9B:04:2F";
pub const SETTINGS: &str = "10:02:00:03:27:49:00:01:00:00:00:80:5F:9B:04:2F";
// Home temperature, Out temperature, Schedule Monday + Tuesday + Wednesday
pub const SCHEDULE_1: &str = "10:02:00:0D:27:49:00:01:00:00:00:80:5F:9B:04:2F";
// Schedule Thursday + Friday
pub const SCHEDULE_2: &str = "10:02:00:0E:27:49:00:01:00:00:00:80:5F:9B:04:2F";
// Schedule Saturday + Sunday
pub const SCHEDULE_3: &str = "10:02:00:0F:27:49:00:01:00:00:00:80:5F:9B:04:2F";

//
// btleplug's interface varies a bit between OSes. According to the crate docs,
// this will be fixed in the future.
//

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().unwrap();
    adapters.into_iter().nth(0).unwrap()
}

#[cfg(target_os = "linux")]
fn get_central(manager: &Manager) -> ConnectedAdapter {
    let adapters = manager.adapters().unwrap();
    let adapter = adapters.into_iter().nth(0).unwrap();
    adapter.connect().unwrap()
}

pub struct ScannedBluetoothPeripheral {
    pub name: String,
    pub address: String,
}

trait PeripheralWrapper: Send + Sync + std::fmt::Debug {
    fn discover_characteristics(&self) -> Result<Vec<Characteristic>>;
    fn command(&self, characteristic: &Characteristic, data: &[u8]) -> Result<()>;
    fn read(&self, characteristic: &Characteristic) -> Result<Vec<u8>>;
    fn request(&self, characteristic: &Characteristic, data: &[u8]) -> Result<()>;
}

#[derive(Debug)]
struct BtleplugPeripheralWrapper<P>
where
    P: Peripheral,
{
    p: P,
}

impl<P> BtleplugPeripheralWrapper<P>
where
    P: Peripheral,
{
    pub fn new(peripheral: P) -> Self {
        Self {
            p: peripheral,
        }
    }
}

impl<P> PeripheralWrapper for BtleplugPeripheralWrapper<P>
where
    P: Peripheral,
{
    fn command(&self, characteristic: &Characteristic, data: &[u8]) -> Result<()> {
        self.p.command(characteristic, data)
    }

    fn discover_characteristics(&self) -> Result<Vec<Characteristic>> {
        self.p.discover_characteristics()
    }

    fn read(&self, characteristic: &Characteristic) -> Result<Vec<u8>> {
        self.p.read(characteristic)
    }

    fn request(&self, characteristic: &Characteristic, data: &[u8]) -> Result<()> {
        self.p.request(characteristic, data)
    }
}

#[derive(Debug)]
pub struct ConnectedBluetoothPeripheral {
    p: Box<dyn PeripheralWrapper>,
}

impl ConnectedBluetoothPeripheral {
    pub fn new<P>(peripheral: P) -> Self
    where
        P: Peripheral + 'static,
    {
        let p = Box::new(BtleplugPeripheralWrapper::new(peripheral));
        Self { p }
    }

    pub fn read_characteristics(&self, relevant_uuids: HashSet<String>) -> Result<HashMap<String, Vec<u8>>> {
        let characteristics = self.p.discover_characteristics()?;
        let mut result = HashMap::new();
        for characteristic in characteristics.iter() {
            let uuid = characteristic.uuid.to_string();
            if relevant_uuids.contains(&uuid) {
                let data = self.read_data(&uuid).unwrap();
                result.insert(uuid, data);
            }
        }
        Ok(result)
    }
    
    pub fn send_pin_code(&self) {
        let pin_code_data: Vec<u8> = vec![0 as u8, 0 as u8, 0 as u8, 0 as u8];
        self.write_data(PIN_CODE_CHARACTERISTIC, &pin_code_data).unwrap();
        eprintln!("Wrote pin code");
    }
    
    pub fn write_data(&self, characteristic: &str, data: &Vec<u8>) -> Result<()> {
        let characteristics = self.p.discover_characteristics()?;
        let bluetooth_characteristic = self.characteristic_with_uuid(&characteristics, characteristic);
        self.p.request(bluetooth_characteristic, data)?;
        Ok(())
    }
    
    pub fn read_data(&self, characteristic: &str) -> Result<Vec<u8>> {
        let characteristics = self.p.discover_characteristics()?;
        let bluetooth_characteristic = self.characteristic_with_uuid(&characteristics, characteristic);
        self.p.read(bluetooth_characteristic)
    }
    
    fn characteristic_with_uuid<'a>(&self, characteristics: &'a Vec<Characteristic>, uuid: &str) -> &'a Characteristic {
        let uuid_string = uuid.to_string();
        characteristics
            .iter()
            .find(|&characteristic| characteristic.uuid.to_string() == uuid.to_string())
            .unwrap()
    }
}

pub fn scan(duration: Duration) -> Vec<ScannedBluetoothPeripheral> {
    let manager = Manager::new().unwrap();
    let central = get_central(&manager);
    central.start_scan().unwrap();

    // Panics after a while, at least on MacOS
    /*match central.event_receiver() {
      Some(receiver) => {
        for i in 0..10 {
          let event = receiver.recv().unwrap();
          match event {
            CentralEvent::DeviceDiscovered(address) => {
              match central.peripheral(address) {
                Some(peripheral) => {
                  let local_name = peripheral.properties().local_name;
                  match local_name {
                    Some(name) => println!("Got a peripheral: {}", name),
                    None => println!("Unknown peripheral")
                  }
                },
                None => {
                  println!("Warning: Discovered device unknown to central: {}", address)
                }
              }
            },
            _ => {} // Ignore
          }
        }
      },
      None => println!("Could not get an event receiver")
    }*/

    // Since the above commented-out code panics, let's just wait print out the
    // peripherals discovered at the end.
    thread::sleep(duration);

    central.stop_scan().unwrap();

    let mut result: Vec<ScannedBluetoothPeripheral> = vec![];
    for peripheral in central.peripherals().iter() {
        match peripheral.properties().local_name {
            Some(name) => result.push(ScannedBluetoothPeripheral {
                name: name,
                address: peripheral.address().to_string(),
            }),
            None => {} // Ignore
        }
    }

    result
}

pub fn connect<F>(matches_name: F, ensure_timer_button_pressed: bool) -> Result<ConnectedBluetoothPeripheral>
    where F: Fn(&String) -> bool {
    let manager = Manager::new().unwrap();
    let central = get_central(&manager);

    // This is hacky: Just start scanning, and query for discovered peripherals every couple of seconds.
    // If our peripheral does not appear within 2 minutes, we will give up.
    central.start_scan().unwrap();

    let end = SystemTime::now() + Duration::from_secs(120);
    while SystemTime::now() < end {
        eprint!(".");

        for peripheral in central.peripherals().into_iter() {
            match peripheral.properties().local_name {
                Some(peripheral_name) => {
                    if matches_name(&peripheral_name) {
                        eprintln!("Found thermostat");
                        if ensure_timer_button_pressed {
                            eprintln!("This is the first time you connect to this thermostat, so we need to fetch the secret key.");
                            eprintln!("Please click the timer button on the thermostat, then press enter on your keyboard to continue connecting.");
                            let mut input = String::new();
                            io::stdin().read_line(&mut input).unwrap();
                        }

                        peripheral.connect()?;

                        let result = ConnectedBluetoothPeripheral::new(peripheral);
                        eprintln!("Connected to peripheral");
                        result.send_pin_code();

                        return Ok(result)
                    }
                }
                None => {} // Ignore
            }
        }

        thread::sleep(Duration::from_secs(2));
    }

    central.stop_scan().unwrap();

    Err(Error::NotConnected)
}
