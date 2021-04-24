use chrono::prelude::*;

use crate::models::thermostats::Thermostats;
use crate::models::parsed_thermostat::ParsedThermostat;

pub fn execute(arguments: Vec<String>) {
    if arguments.len() != 1 {
        panic!("Expected just one parameter to show. Got {}.", arguments.len());
    }
    let name = &arguments[0];

    let thermostats = Thermostats::load().expect("Could not read thermostat data");
    let thermostat = match thermostats.thermostats.into_iter().find(|t| t.serial == *name) {
        None => { panic!("Don't know about any thermostats with name '{}'. Have you called the 'read' command first?", name) },
        Some(t) => t,
    };

    let parsed_thermostat = ParsedThermostat::from_thermostat(&thermostat);

    println!("Name: {}", parsed_thermostat.name);
    println!("{}% battery", parsed_thermostat.battery_percentage);
    println!("");
    println!("Set-point/room temperature: {} / {}", parsed_thermostat.set_point_temperature, parsed_thermostat.room_temperature);
    println!("Vacation/frost protection temperature: {} / {}", parsed_thermostat.vacation_temperature, parsed_thermostat.frost_protection_temperature);
    println!("");
    println!("Schedule mode: {}", parsed_thermostat.schedule_mode);
    if let Some((vacation_start, vacation_end)) = parsed_thermostat.vacation_period {
        println!("Vacation: {} - {}", formatted_date(vacation_start), formatted_date(vacation_end));
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

    if thermostat.new_set_point_temperature.is_some() || thermostat.new_vacation_period.is_some() {
        println!("");
        println!("Properties to be written back to thermostat:");

        if let Some(new_set_point_temperature) = thermostat.new_set_point_temperature {
            println!("Set-point temperature: {}", new_set_point_temperature);
        }
        if let Some((new_vacation_start, new_vacation_end)) = thermostat.new_vacation_period {
            if new_vacation_start == 0 {
                println!("Reset vacation");
            } else {
                let new_vacation_start = formatted_date(Utc.timestamp(new_vacation_start, 0));
                let new_vacation_end = formatted_date(Utc.timestamp(new_vacation_end, 0));
                println!("Vacation: {} - {}", new_vacation_start, new_vacation_end);
            }
        }
    }
}

fn formatted_date(t: chrono::DateTime<Utc>) -> String {
    t.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string()
}
