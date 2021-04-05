use crate::commands::read::read_from_connected_peripheral;
use crate::models::thermostats::Thermostats;
use crate::models::thermostat_names::*;
use crate::bluetooth;

pub fn execute(arguments: Vec<String>) {
  if arguments.len() != 1 {
    panic!("Expected just one parameter to sync. Got {}.", arguments.len());
  }
  let serial = &arguments[0];

  let mut thermostats = Thermostats::load().unwrap();
  let thermostat = thermostats.get(serial).expect("Unknown thermostat serial. You need to do a read first.");
  let secret = &thermostat.secret;

  let connected_peripheral = bluetooth::connect(|name| { is_thermostat_name(name) && &stripped_name(name) == serial }, false).unwrap();
  let thermostat_with_original_values = read_from_connected_peripheral(&connected_peripheral, serial, Some(secret));

  // TODO: Update properties

  let thermostat_with_updated_values = read_from_connected_peripheral(&connected_peripheral, serial, Some(secret));
  connected_peripheral.disconnect();

  thermostats.push(thermostat_with_updated_values);
  thermostats.save().unwrap();
}
