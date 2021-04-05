extern crate chrono;
use chrono::offset::Local;

use crate::models::thermostats::Thermostats;
use crate::models::parsed_thermostat::ParsedThermostat;

pub fn execute(arguments: Vec<String>) {
    if arguments.len() != 1 {
        panic!(
            "Expected just one parameter to read. Got {}.",
            arguments.len()
        );
    }
    let name = &arguments[0];

    let thermostats = Thermostats::load().expect("Could not read thermostat data");
    let thermostat = match thermostats.thermostats.into_iter().find(|t| t.serial == *name) {
        None => { panic!("Don't know about any thermostats with name '{}'. Have you called the 'read' command first?", name) },
        Some(t) => t,
    };

    let parsed_thermostat = ParsedThermostat::from_thermostat(&thermostat);

    println!("Name: {}", parsed_thermostat.name);
    println!("");
    println!("Set-point/room temperature: {} / {}", parsed_thermostat.set_point_temperature, parsed_thermostat.room_temperature);
    println!("Vacation/frost protection temperature: {} / {}", parsed_thermostat.vacation_temperature, parsed_thermostat.frost_protection_temperature);
    println!("");
    println!("Schedule mode: {}", parsed_thermostat.schedule_mode);
    if let Some((vacation_start, vacation_end)) = parsed_thermostat.vacation_period {
        println!("Vacation: {} - {}", vacation_start.with_timezone(&Local), vacation_end.with_timezone(&Local));
    }
    println!("");
    println!("Daily Schedules");
    println!("Monday: {}", parsed_thermostat.schedule_monday);
    println!("Tuesday: {}", parsed_thermostat.schedule_tuesday);
    println!("Wednesday: {}", parsed_thermostat.schedule_wednesday);
    println!("Thursday: {}", parsed_thermostat.schedule_thursday);
    println!("Friday: {}", parsed_thermostat.schedule_friday);
    println!("Saturday: {}", parsed_thermostat.schedule_saturday);
    println!("Sunday: {}", parsed_thermostat.schedule_sunday);

    if let Some(new_set_point_temperature) = thermostat.new_set_point_temperature {
        println!("");
        println!("Properties to be written back to thermostat:");
        println!("Set-point temperature: {}", new_set_point_temperature);
    }
}
