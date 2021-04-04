use std::time::Duration;

#[path = "../bluetooth.rs"]
mod bluetooth;

use crate::models::thermostat_names::*;

pub fn execute(arguments: Vec<String>) {
    eprintln!("Scanning for 2 minutes. Please wait.");
    let peripherals = bluetooth::scan(Duration::from_secs(120));

    let mut peripheral_found = false;
    for peripheral in peripherals.iter() {
        if is_thermostat_name(&peripheral.name) {
            println!("{}", stripped_name(&peripheral.name));
            peripheral_found = true
        }
    }

    if !peripheral_found {
        eprintln!("No thermostats found");
        std::process::exit(1)
    }
}
