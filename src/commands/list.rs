use crate::models::thermostats::Thermostats;
use crate::models::parsed_thermostat::ParsedThermostat;

pub fn execute(arguments: Vec<String>) {
    if arguments.len() != 0 {
        panic!("Expected no parameters to list. Got {}.", arguments.len());
    }

    let thermostats = Thermostats::load().expect("Could not read thermostat data");
    let mut thermostats = thermostats.thermostats;
    thermostats.sort_by(|t1, t2| t1.name.cmp(&t2.name));

    for thermostat in thermostats {
        let parsed_thermostat = ParsedThermostat::from_thermostat(&thermostat);
        println!("{} {} {}%", thermostat.serial, parsed_thermostat.name, parsed_thermostat.battery_percentage);
    }
}
