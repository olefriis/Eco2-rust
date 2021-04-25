use crate::models::thermostats::{Thermostats, Thermostat};
use chrono::prelude::*;

const SET_POINT_TEMPERATURE: &str = "set-point-temperature";
const VACATION_PERIOD: &str = "vacation-period";
const SCHEDULE_MODE: &str = "schedule-mode";

pub fn execute(arguments: Vec<String>) {
    if arguments.len() < 3 {
        panic!(
          "Expected at least three arguments: The thermostat serial, the property to set, and the value(s). Got {} arguments.",
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
        VACATION_PERIOD => set_vacation_period(&mut thermostat, remaining_arguments),
        SCHEDULE_MODE => set_schedule_mode(&mut thermostat, remaining_arguments),
        _ => panic!("Unknown property: {}. Expected one of set-point-temperature, vacation-period, and schedule-mode", property),
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

fn set_vacation_period(thermostat: &mut Thermostat, arguments: &[String]) {
    // Assume that we want to clear the vacation period if we only have one argument.
    // Is this a bit too hacky?
    if arguments.len() == 1 {
        clear_vacation_period(thermostat, arguments);
        return;
    }

    if arguments.len() != 2 {
        panic!("Expected two arguments as vacation period, got {}", arguments.len());
    }
    let new_vacation_period_start = parse_date_time(&arguments[0]);
    let new_vacation_period_end = parse_date_time(&arguments[1]);

    thermostat.new_vacation_period = Some((new_vacation_period_start, new_vacation_period_end));
}

fn clear_vacation_period(thermostat: &mut Thermostat, arguments: &[String]) {
    if arguments[0] != "clear" {
        panic!("Expected either a start and end date, or just 'clear'");
    }

    thermostat.new_vacation_period = Some((0, 0));
}

fn set_schedule_mode(thermostat: &mut Thermostat, arguments: &[String]) {
    if arguments.len() != 1 {
        panic!("Expected just one argument as schedule mode, got {}", arguments.len());
    }

    thermostat.new_schedule_mode = Some(parse_schedule_mode(&arguments[0]));
}

fn parse_date_time(arg: &str) -> i64 {
    let parsed_date_time = Local.datetime_from_str(arg, "%Y-%m-%d %H:%M").expect("Could not parse date time. Should be in format YYYY-mm-dd HH:MM");
    let minutes = parsed_date_time.minute();
    if minutes != 0 {
        panic!("Only minutes of 00 are supported, got {:02}: {}", minutes, arg);
    }
    parsed_date_time.timestamp()
}

fn parse_schedule_mode(arg: &str) -> u8 {
    match arg {
        "manual" => 0,
        "scheduled" => 1,
        "vacation" => 3,
        _ => panic!("Unknown schedule mode: {}. Expected one of manual, scheduled, and vacation", arg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::parsed_thermostat::{ParsedThermostat, ScheduleMode};

    #[test]
    fn it_can_parse_vacation_date() {
        let parsed_timestamp = parse_date_time("2021-05-24 13:00");
        let expected_timestamp = Local.ymd(2021, 5, 24).and_hms(13, 0, 0).timestamp();
        assert_eq!(parsed_timestamp, expected_timestamp);
    }

    #[test]
    #[should_panic(expected = "Only minutes of 00 are supported, got 07: 2021-05-24 13:07")]
    fn it_disallows_specifying_minutes_in_vacation_date() {
        parse_date_time("2021-05-24 13:07");
    }

    #[test]
    #[should_panic(expected = "Could not parse date time. Should be in format YYYY-mm-dd HH:MM")]
    fn it_panics_when_given_invalid_date_format() {
        parse_date_time("24/5 2021 13:07");
    }

    #[test]
    fn it_can_parse_schedule_mode() {
        assert_eq!(ParsedThermostat::parse_schedule_mode(parse_schedule_mode("manual")), ScheduleMode::Manual);
        assert_eq!(ParsedThermostat::parse_schedule_mode(parse_schedule_mode("scheduled")), ScheduleMode::Scheduled);
        assert_eq!(ParsedThermostat::parse_schedule_mode(parse_schedule_mode("vacation")), ScheduleMode::Vacation);
    }

    #[test]
    #[should_panic(expected = "Unknown schedule mode: abc. Expected one of manual, scheduled, and vacation")]
    fn it_panics_on_unknown_schedule_mode() {
        parse_schedule_mode("abc");
    }
}
