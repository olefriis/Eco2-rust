use std::collections::HashSet;

use crate::bluetooth;
use crate::bluetooth::ConnectedBluetoothPeripheral;
use crate::models::thermostats::{Thermostats, Thermostat};
use crate::models::thermostat_names::*;

pub fn execute(arguments: Vec<String>) {
    if arguments.len() != 1 {
        panic!("Expected just one parameter to read. Got {}.", arguments.len());
    }
    let serial = &arguments[0];

    let mut thermostats = Thermostats::load().unwrap();
    let secret = thermostats.get(serial).and_then(|t| { Some(&t.secret) });

    let new_thermostat = read_from_thermostat(serial, secret);
    thermostats.push(new_thermostat);
    thermostats.save().unwrap();
}

fn read_from_thermostat(serial: &String, secret: Option<&Vec<u8>>) -> Thermostat {
    let first_connection = secret.is_none();
    if first_connection {
        eprintln!("Reading from {} for the first time...", serial);
    } else {
        eprintln!("Reading from {}...", serial);
    }

    let connected_peripheral = bluetooth::connect(|name| { is_thermostat_name(name) && &stripped_name(name) == serial }, first_connection).unwrap();

    let result = read_from_connected_peripheral(&connected_peripheral, serial, secret);
    connected_peripheral.disconnect();
    result
}

pub fn read_from_connected_peripheral(peripheral: &ConnectedBluetoothPeripheral, serial: &String, secret: Option<&Vec<u8>>) -> Thermostat {
    let mut characteristics_to_read = HashSet::new();

    if secret.is_none() {
        characteristics_to_read.insert(bluetooth::SECRET_KEY.to_string());
    }

    characteristics_to_read.insert(bluetooth::DEVICE_NAME.to_string());
    characteristics_to_read.insert(bluetooth::BATTERY_LEVEL.to_string());
    characteristics_to_read.insert(bluetooth::TEMPERATURE.to_string());
    characteristics_to_read.insert(bluetooth::SETTINGS.to_string());
    characteristics_to_read.insert(bluetooth::SCHEDULE_1.to_string());
    characteristics_to_read.insert(bluetooth::SCHEDULE_2.to_string());
    characteristics_to_read.insert(bluetooth::SCHEDULE_3.to_string());

    let characteristic_values = peripheral.read_characteristics(characteristics_to_read).unwrap();

    let secret = match secret {
        Some(s) => s.clone(),
        None => characteristic_values.get(&bluetooth::SECRET_KEY.to_string()).unwrap().clone()
    };
    let name = characteristic_values.get(&bluetooth::DEVICE_NAME.to_string()).unwrap().clone();
    let battery_level = characteristic_values.get(&bluetooth::BATTERY_LEVEL.to_string()).unwrap().clone();
    let temperature = characteristic_values.get(&bluetooth::TEMPERATURE.to_string()).unwrap().clone();
    let settings = characteristic_values.get(&bluetooth::SETTINGS.to_string()).unwrap().clone();
    let schedule_1 = characteristic_values.get(&bluetooth::SCHEDULE_1.to_string()).unwrap().clone();
    let schedule_2 = characteristic_values.get(&bluetooth::SCHEDULE_2.to_string()).unwrap().clone();
    let schedule_3 = characteristic_values.get(&bluetooth::SCHEDULE_3.to_string()).unwrap().clone();

    Thermostat {
        serial: serial.clone(),
        secret,
        name,
        battery_level,
        temperature,
        settings,
        schedule_1,
        schedule_2,
        schedule_3,

        ..Default::default()
    }
}