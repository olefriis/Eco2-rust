use std::collections::HashSet;

use crate::bluetooth;
use crate::commands::read::read_from_connected_peripheral;
use crate::models::thermostat_names::*;
use crate::models::thermostats::Thermostats;
use crate::models::parsed_thermostat::update_set_point_temperature;

pub fn execute(arguments: Vec<String>) {
    if arguments.len() != 1 {
        panic!("Expected just one parameter to sync. Got {}.", arguments.len());
    }
    let serial = &arguments[0];

    let mut thermostats = Thermostats::load().unwrap();
    let thermostat = thermostats.get(serial).expect("Unknown thermostat serial. You need to do a read first.");
    let secret = &thermostat.secret;

    let connected_peripheral = bluetooth::connect(|name| is_thermostat_name(name) && &stripped_name(name) == serial, false).unwrap();

    if let Some(set_point_temperature) = thermostat.new_set_point_temperature {
        let mut characteristics_to_read = HashSet::new();
        characteristics_to_read.insert(bluetooth::TEMPERATURE.to_string());
        let characteristic_values = connected_peripheral.read_characteristics(characteristics_to_read).unwrap();
        let old_temperature = characteristic_values.get(&bluetooth::TEMPERATURE.to_string()).unwrap();

        let new_temperature = update_set_point_temperature(old_temperature, secret, set_point_temperature);
        connected_peripheral.write_data(bluetooth::TEMPERATURE, &new_temperature).unwrap();
    }

    let thermostat_with_updated_values = read_from_connected_peripheral(&connected_peripheral, serial, Some(secret));
    connected_peripheral.disconnect();

    thermostats.push(thermostat_with_updated_values);
    thermostats.save().unwrap();
}
