use std::collections::HashSet;

use crate::bluetooth;
use crate::commands::read::read_from_connected_peripheral;
use crate::models::thermostat_names::*;
use crate::models::thermostats::{Thermostat, Thermostats};
use crate::models::parsed_thermostat::{update_schedule_mode, update_set_point_temperature, update_vacation_period};

pub fn execute(arguments: Vec<String>) {
    if arguments.len() != 1 {
        panic!("Expected just one parameter to sync. Got {}.", arguments.len());
    }
    let serial = &arguments[0];

    let mut thermostats = Thermostats::load().unwrap();
    let thermostat = thermostats.get(serial).expect("Unknown thermostat serial. You need to do a read first.");
    let secret = &thermostat.secret;

    let connected_peripheral = bluetooth::connect(|name| is_thermostat_name(name) && &stripped_name(name) == serial, false).unwrap();

    update_characteristics(&thermostat, &secret, &connected_peripheral);
    let thermostat_with_updated_values = read_from_connected_peripheral(&connected_peripheral, serial, Some(secret));
    connected_peripheral.disconnect();

    thermostats.push(thermostat_with_updated_values);
    thermostats.save().unwrap();
}

fn update_characteristics(thermostat: &Thermostat, secret: &Vec<u8>, connected_peripheral: &bluetooth::ConnectedBluetoothPeripheral) {
    let alter_temperature = thermostat.new_set_point_temperature.is_some();
    let alter_settings = thermostat.new_vacation_period.is_some() || thermostat.new_schedule_mode.is_some();

    // First, find out which characteristics we want to update
    let mut characteristics_to_alter = HashSet::new();
    if alter_temperature {
        characteristics_to_alter.insert(bluetooth::TEMPERATURE.to_string());
    }
    if alter_settings {
        characteristics_to_alter.insert(bluetooth::SETTINGS.to_string());
    }

    // Do nothing if we don't need to update any characteristics
    if characteristics_to_alter.len() == 0 {
        return;
    }

    // Read the characteristics we want to update
    let mut characteristic_values = connected_peripheral.read_characteristics(characteristics_to_alter).unwrap();

    // Update the characteristics we just read
    if let Some(set_point_temperature) = thermostat.new_set_point_temperature {
        let temperature = characteristic_values.get_mut(&bluetooth::TEMPERATURE.to_string()).unwrap();
        *temperature = update_set_point_temperature(temperature, secret, set_point_temperature);
    }
    if let Some((vacation_period_start, vacation_period_end)) = thermostat.new_vacation_period {
        let settings = characteristic_values.get_mut(&bluetooth::SETTINGS.to_string()).unwrap();
        *settings = update_vacation_period(settings, secret, vacation_period_start, vacation_period_end);
    }
    if let Some(schedule_mode) = thermostat.new_schedule_mode {
        let settings = characteristic_values.get_mut(&bluetooth::SETTINGS.to_string()).unwrap();
        *settings = update_schedule_mode(settings, secret, schedule_mode);
    }

    // ...and finally write back the updated characteristics
    for (characteristic_name, characteristic_value) in characteristic_values.iter() {
        connected_peripheral.write_data(&characteristic_name[..], characteristic_value).unwrap();
    }
}
