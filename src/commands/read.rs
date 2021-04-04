use std::collections::HashSet;

#[path = "../bluetooth.rs"]
mod bluetooth;

use crate::models::thermostats::{Thermostats, Thermostat};
use crate::models::thermostat_names::*;

pub fn execute(arguments: Vec<String>) {
    if arguments.len() != 1 {
        panic!(
            "Expected just one parameter to read. Got {}.",
            arguments.len()
        );
    }
    let serial = &arguments[0];

    let thermostats = Thermostats::load().unwrap();
    let existing_thermostat = thermostats.thermostats.iter().find(|thermostat| &thermostat.serial == serial);
    let mut other_thermostats: Vec<Thermostat> = thermostats.thermostats.iter().filter(|thermostat| &thermostat.serial != serial).cloned().collect();

    let first_connection = existing_thermostat.is_none();
    if first_connection {
        eprintln!("Reading from {} for the first time...", serial);
    } else {
        eprintln!("Reading from {}...", serial);
    }
    let connected_peripheral = bluetooth::connect(|name| { is_thermostat_name(name) && &stripped_name(name) == serial }, first_connection).unwrap();

    let mut characteristics_to_read = HashSet::new();
    if first_connection {
        characteristics_to_read.insert(bluetooth::SECRET_KEY.to_string());
    }
    characteristics_to_read.insert(bluetooth::DEVICE_NAME.to_string());
    // Issues with battery level... getting this error from btleplug:
    // thread '<unnamed>' panicked at 'called `Result::unwrap()` on an `Err` value: Error(Parser(InvalidLength { expected: Any([36, 32]), found: 4 }))', /Users/olefriis/.cargo/registry/src/github.com-1ecc6299db9ec823/btleplug-0.6.0/src/corebluetooth/central_delegate.rs:433:90
    //characteristics_to_read.insert(bluetooth::BATTERY_LEVEL.to_string());
    characteristics_to_read.insert(bluetooth::TEMPERATURE.to_string());
    characteristics_to_read.insert(bluetooth::SETTINGS.to_string());
    characteristics_to_read.insert(bluetooth::SCHEDULE_1.to_string());
    characteristics_to_read.insert(bluetooth::SCHEDULE_2.to_string());
    characteristics_to_read.insert(bluetooth::SCHEDULE_3.to_string());
    let characteristic_values = connected_peripheral.read_characteristics(characteristics_to_read).unwrap();

    let secret = match existing_thermostat {
        Some(thermostat) => thermostat.secret.clone(),
        None => characteristic_values.get(&bluetooth::SECRET_KEY.to_string()).unwrap().clone()
    };
    let name = characteristic_values.get(&bluetooth::DEVICE_NAME.to_string()).unwrap().clone();
    //let battery_level = characteristic_values.get(&bluetooth::BATTERY_LEVEL.to_string()).unwrap().clone();
    let temperature = characteristic_values.get(&bluetooth::TEMPERATURE.to_string()).unwrap().clone();
    let settings = characteristic_values.get(&bluetooth::SETTINGS.to_string()).unwrap().clone();
    let schedule_1 = characteristic_values.get(&bluetooth::SCHEDULE_1.to_string()).unwrap().clone();
    let schedule_2 = characteristic_values.get(&bluetooth::SCHEDULE_2.to_string()).unwrap().clone();
    let schedule_3 = characteristic_values.get(&bluetooth::SCHEDULE_3.to_string()).unwrap().clone();

    let new_thermostat = Thermostat {
        serial: serial.clone(),
        secret,
        name,
        //battery_level,
        temperature,
        settings,
        schedule_1,
        schedule_2,
        schedule_3,
    };

    // This really should be handled by the Thermostats struct itself. This is a bit hacky.
    let mut new_thermostats_vec = vec![new_thermostat];
    new_thermostats_vec.append(&mut other_thermostats);
    let thermostats = Thermostats {
        thermostats: new_thermostats_vec,
    };
    thermostats.save().unwrap();
}
