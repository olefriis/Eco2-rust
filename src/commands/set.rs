use crate::models::thermostats::{Thermostats, Thermostat};

const SET_POINT_TEMPERATURE: &str = "set-point-temperature";

pub fn execute(arguments: Vec<String>) {
    if arguments.len() != 3 {
        panic!(
          "Expected three arguments: The thermostat serial, the property to set (currently just set-point-temperature supported), and the value. Got {} arguments.",
          arguments.len()
      );
    }
    let serial = &arguments[0];

    let mut thermostats = Thermostats::load().expect("Could not read thermostat data");
    let thermostat = thermostats.get(serial);

    let mut thermostat = match thermostat {
        Some(t) => t.clone(),
        None => panic!("Thermostat with serial {} not found. Have you run the read command first?", serial),
    };

    let property = &arguments[1][..];
    let remaining_arguments = &arguments[2..];
    match property {
        SET_POINT_TEMPERATURE => set_set_point_temperature(&mut thermostat, remaining_arguments),
        _ => panic!("Unknown property: {}. Only set-point-temperature supported supported for now.", property),
    }

    thermostats.push(thermostat);
    thermostats.save().unwrap();
}

fn set_set_point_temperature(thermostat: &mut Thermostat, arguments: &[String]) {
    if arguments.len() != 1 {
        panic!("Expected just one argument as set-point temperature, got {}", arguments.len());
    }
    let new_set_point_temperature = arguments[0].parse::<f32>().expect("Cannot parse supplied set-point temperature");

    thermostat.new_set_point_temperature = Some(new_set_point_temperature);
}
