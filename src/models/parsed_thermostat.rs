extern crate chrono;
use chrono::DateTime;
use chrono::offset::Utc;
use chrono::TimeZone;

use crate::models::thermostats::Thermostat;
use std::fmt;

#[path = "../decryption.rs"]
mod decryption;
use decryption::{decrypt, encrypt};

pub fn update_set_point_temperature(encrypted_temperature: &Vec<u8>, secret: &Vec<u8>, set_point_temperature: f32) -> Vec<u8> {
    let temperature = Temperature::from_degrees_celcius(set_point_temperature);

    let mut decrypted_temperature = decrypt(secret, encrypted_temperature);
    decrypted_temperature[0] = temperature.value;

    encrypt(secret, &decrypted_temperature)
}

pub struct ParsedThermostat {
    pub name: String,
    pub set_point_temperature: Temperature,
    pub room_temperature: Temperature,
    pub vacation_temperature: Temperature,
    pub frost_protection_temperature: Temperature,
    pub schedule_mode: ScheduleMode,
    pub vacation_period: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub schedule_monday: DailySchedule,
    pub schedule_tuesday: DailySchedule,
    pub schedule_wednesday: DailySchedule,
    pub schedule_thursday: DailySchedule,
    pub schedule_friday: DailySchedule,
    pub schedule_saturday: DailySchedule,
    pub schedule_sunday: DailySchedule,
}

impl ParsedThermostat {
    pub fn from_thermostat(thermostat: &Thermostat) -> ParsedThermostat {
        let decrypted_name = decrypt(&thermostat.secret, &thermostat.name);
        let decrypted_temperature = decrypt(&thermostat.secret, &thermostat.temperature);
        let decrypted_settings = decrypt(&thermostat.secret, &thermostat.settings);
        let decrypted_schedule_1 = decrypt(&thermostat.secret, &thermostat.schedule_1);
        let decrypted_schedule_2 = decrypt(&thermostat.secret, &thermostat.schedule_2);
        let decrypted_schedule_3 = decrypt(&thermostat.secret, &thermostat.schedule_3);
    
        let set_point_temperature = Temperature::from_byte(decrypted_temperature[0]);
        let room_temperature = Temperature::from_byte(decrypted_temperature[1]);
        let vacation_temperature = Temperature::from_byte(decrypted_settings[5]);
        let frost_protection_temperature = Temperature::from_byte(decrypted_settings[3]);

        let schedule_mode = match decrypted_settings[4] {
            0 => ScheduleMode::Manual,
            1 => ScheduleMode::Scheduled,
            3 => ScheduleMode::Vacation,
            _ => panic!("Unknown schedule mode: {}", decrypted_settings[4]),
        };

        let start_vacation = ParsedThermostat::decode_datetime(&decrypted_settings[6..10]);
        let end_vacation = ParsedThermostat::decode_datetime(&decrypted_settings[10..14]);
        let vacation_period = match (start_vacation, end_vacation) {
            (Some(start), Some(end)) => Some((start, end)),
            _ => None,
        };

        let schedule_monday = ParsedThermostat::decode_daily_schedule(&decrypted_schedule_1[2..8]);
        let schedule_tuesday = ParsedThermostat::decode_daily_schedule(&decrypted_schedule_1[8..14]);
        let schedule_wednesday = ParsedThermostat::decode_daily_schedule(&decrypted_schedule_1[14..20]);
        let schedule_thursday = ParsedThermostat::decode_daily_schedule(&decrypted_schedule_2[0..6]);
        let schedule_friday = ParsedThermostat::decode_daily_schedule(&decrypted_schedule_2[6..12]);
        let schedule_saturday = ParsedThermostat::decode_daily_schedule(&decrypted_schedule_3[0..6]);
        let schedule_sunday = ParsedThermostat::decode_daily_schedule(&decrypted_schedule_3[6..12]);

        ParsedThermostat {
            name: ParsedThermostat::decode_name(&decrypted_name),
            set_point_temperature,
            room_temperature,
            vacation_temperature,
            frost_protection_temperature,
            schedule_mode,
            vacation_period,
            schedule_monday,
            schedule_tuesday,
            schedule_wednesday,
            schedule_thursday,
            schedule_friday,
            schedule_saturday,
            schedule_sunday,
        }
    }

    fn decode_name(encoded_name: &Vec<u8>) -> String {
        let mut result = String::new();
        encoded_name.iter()
            .filter(|i| **i != 0)
            .for_each(|i| result.push((*i as u8) as char));
        result
    }

    fn decode_datetime(bytes: &[u8]) -> Option<DateTime<Utc>> {
        let mut seconds_since_epoch: i64 = 0;
        for byte in bytes {
            seconds_since_epoch *= 256;
            seconds_since_epoch += *byte as i64;
        }

        if seconds_since_epoch == 0 {
            None
        } else {
            Some(Utc.timestamp(seconds_since_epoch, 0))
        }
    }

    fn decode_daily_schedule(bytes: &[u8]) -> DailySchedule {
        let mut away = true;
        let mut intervals = vec![];
        let mut last_byte = bytes[0];
        for byte in bytes {
            last_byte = *byte;

            // Skip the first part of the day if it's 0 minutes long
            if last_byte != 0 {
                intervals.push(TimeInterval {
                    // The byte represents the number of 30-minute increments from Midnight
                    ends_at_hour: last_byte / 2,
                    ends_at_minute: (last_byte % 2) * 30,
                    // ...and home/away switches with each time interval
                    setting: if away { TimeSetting::Away } else { TimeSetting::Home },
                });
            }

            away = !away;
            if last_byte == 48 {
                // End early if we hit Midnight
                break;
            }
        }

        // Patch with "Away until 24:00" if we haven't filled out the whole day yet
        if last_byte < 48 {
            intervals.push(TimeInterval {
                ends_at_hour: 24,
                ends_at_minute: 0,
                setting: TimeSetting::Away,
            });
        }

        DailySchedule {
            intervals
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct Temperature {
    value: u8,
}

impl Temperature {
    fn from_byte(byte: u8) -> Temperature {
        Temperature { value: byte.clone() }
    }

    fn from_degrees_celcius(degrees_celcius: f32) -> Temperature {
        Temperature { value: (degrees_celcius * 2.0) as u8 }
    }

    pub fn in_degrees_celcius(&self) -> f32 {
        (self.value as f32) / 2.0
    }
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}°C", self.in_degrees_celcius())
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum ScheduleMode {
    Manual,
    Scheduled,
    Vacation
}

impl fmt::Display for ScheduleMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScheduleMode::Manual => write!(f, "Manual"),
            ScheduleMode::Scheduled => write!(f, "Scheduled"),
            ScheduleMode::Vacation => write!(f, "Vacation"),
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct DailySchedule {
    pub intervals: Vec<TimeInterval>,
}

impl fmt::Display for DailySchedule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let intervals_as_string = self.intervals.iter().map(|interval| interval.to_string()).collect::<Vec<String>>().join(" - ");
        write!(f, "{}", intervals_as_string)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct TimeInterval {
    pub ends_at_hour: u8,
    pub ends_at_minute: u8,
    pub setting: TimeSetting,
}

impl fmt::Display for TimeInterval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let setting = match self.setting {
            TimeSetting::Home => "Home",
            TimeSetting::Away => "Away",
        };
        write!(f, "{} until {:02}:{:02}", setting, self.ends_at_hour, self.ends_at_minute)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum TimeSetting {
    Home,
    Away,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_update_set_point_temperature() {
        let secret = vec![215u8, 91, 125, 126, 14, 118, 62, 143, 121, 48, 110, 175, 112, 218, 245, 65];
        let encrypted_temperature = vec![87u8, 121, 70, 227, 189, 210, 0, 110];
        let old_decrypted_temperature = decrypt(&secret, &encrypted_temperature);

        let updated_encrypted_temperature = update_set_point_temperature(&encrypted_temperature, &secret, 18.5);
        let decrypted_temperature = decrypt(&secret, &updated_encrypted_temperature);

        // The written value should be 18.5 * 2
        assert_eq!(37u8, decrypted_temperature[0]);
        assert_eq!(old_decrypted_temperature[1..], decrypted_temperature[1..]);
    }

    #[test]
    fn it_can_decrypt_and_decode_name() {
        assert_eq!("Alrum opgang".to_string(), create_parsed_thermostat().name);
    }

    #[test]
    fn it_can_decrypt_and_decode_set_point_temperature() {
        assert_eq!(Temperature::from_degrees_celcius(23.0), create_parsed_thermostat().set_point_temperature);
    }

    #[test]
    fn it_can_decrypt_and_decode_room_temperature() {
        assert_eq!(Temperature::from_degrees_celcius(23.0), create_parsed_thermostat().room_temperature);
    }

    #[test]
    fn it_can_decrypt_and_decode_vacation_temperature() {
        assert_eq!(Temperature::from_degrees_celcius(17.0), create_parsed_thermostat().vacation_temperature);
    }

    #[test]
    fn it_can_decrypt_and_decode_frost_protection_temperature() {
        assert_eq!(Temperature::from_degrees_celcius(6.0), create_parsed_thermostat().frost_protection_temperature);
    }

    #[test]
    fn it_can_decrypt_and_decode_manual_schedule_mode() {
        assert_eq!(ScheduleMode::Manual, create_parsed_thermostat().schedule_mode);
    }

    #[test]
    fn it_can_decrypt_and_decode_scheduled_schedule_mode() {
        assert_eq!(ScheduleMode::Scheduled, create_parsed_thermostat_with_schedule().schedule_mode);
    }

    #[test]
    fn it_can_decrypt_and_decode_vacation_schedule_mode() {
        assert_eq!(ScheduleMode::Vacation, create_parsed_thermostat_with_vacation_schedule().schedule_mode);
    }

    #[test]
    fn it_knows_when_vacation_period_is_not_present() {
        assert_eq!(None, create_parsed_thermostat().vacation_period);
    }

    #[test]
    fn it_can_decrypt_and_decode_vacation_period() {
        assert_eq!(Utc.ymd(2021, 4, 14).and_hms(8, 0, 0), create_parsed_thermostat_with_planned_vacation().vacation_period.unwrap().0);
        assert_eq!(Utc.ymd(2021, 5, 14).and_hms(12, 0, 0), create_parsed_thermostat_with_planned_vacation().vacation_period.unwrap().1);
    }

    #[test]
    fn it_can_decrypt_and_decode_schedules() {
        // Monday: Away until 05:00 - Home until 20:00 - Away until 24:00
        assert_eq!(
            DailySchedule {
                intervals: vec![
                    TimeInterval {
                        ends_at_hour: 5,
                        ends_at_minute: 0,
                        setting: TimeSetting::Away,
                    },
                    TimeInterval {
                        ends_at_hour: 20,
                        ends_at_minute: 0,
                        setting: TimeSetting::Home,
                    },
                    TimeInterval {
                        ends_at_hour: 24,
                        ends_at_minute: 0,
                        setting: TimeSetting::Away,
                    },
                ]
            }, create_parsed_thermostat_with_schedule().schedule_monday);

        // Tuesday: Away until 04:30 - Home until 24:00
        assert_eq!(
            DailySchedule {
                intervals: vec![
                    TimeInterval {
                        ends_at_hour: 4,
                        ends_at_minute: 30,
                        setting: TimeSetting::Away,
                    },
                    TimeInterval {
                        ends_at_hour: 24,
                        ends_at_minute: 0,
                        setting: TimeSetting::Home,
                    },
                ]
            }, create_parsed_thermostat_with_schedule().schedule_tuesday);

        // Wednesday: Away until 08:00 - Home until 10:00 - Away until 14:00 - Home until 17:30 - Away until 24:00
        assert_eq!(
            DailySchedule {
                intervals: vec![
                    TimeInterval {
                        ends_at_hour: 8,
                        ends_at_minute: 0,
                        setting: TimeSetting::Away,
                    },
                    TimeInterval {
                        ends_at_hour: 10,
                        ends_at_minute: 0,
                        setting: TimeSetting::Home,
                    },
                    TimeInterval {
                        ends_at_hour: 14,
                        ends_at_minute: 0,
                        setting: TimeSetting::Away,
                    },
                    TimeInterval {
                        ends_at_hour: 17,
                        ends_at_minute: 30,
                        setting: TimeSetting::Home,
                    },
                    TimeInterval {
                        ends_at_hour: 24,
                        ends_at_minute: 0,
                        setting: TimeSetting::Away,
                    },
                ]
            }, create_parsed_thermostat_with_schedule().schedule_wednesday);

        // Thursday: Away until 05:30 - Home until 07:30 - Away until 09:30 - Home until 12:00 - Away until 14:30 - Home until 18:00 - Away until 24:00
        assert_eq!(
            DailySchedule {
                intervals: vec![
                    TimeInterval {
                        ends_at_hour: 5,
                        ends_at_minute: 30,
                        setting: TimeSetting::Away,
                    },
                    TimeInterval {
                        ends_at_hour: 7,
                        ends_at_minute: 30,
                        setting: TimeSetting::Home,
                    },
                    TimeInterval {
                        ends_at_hour: 9,
                        ends_at_minute: 30,
                        setting: TimeSetting::Away,
                    },
                    TimeInterval {
                        ends_at_hour: 12,
                        ends_at_minute: 0,
                        setting: TimeSetting::Home,
                    },
                    TimeInterval {
                        ends_at_hour: 14,
                        ends_at_minute: 30,
                        setting: TimeSetting::Away,
                    },
                    TimeInterval {
                        ends_at_hour: 18,
                        ends_at_minute: 0,
                        setting: TimeSetting::Home,
                    },
                    TimeInterval {
                        ends_at_hour: 24,
                        ends_at_minute: 0,
                        setting: TimeSetting::Away,
                    },
                ]
            }, create_parsed_thermostat_with_schedule().schedule_thursday);

        // Friday: Away until 24:00
        assert_eq!(
            DailySchedule {
                intervals: vec![
                    TimeInterval {
                        ends_at_hour: 24,
                        ends_at_minute: 0,
                        setting: TimeSetting::Away,
                    },
                ]
            }, create_parsed_thermostat_with_schedule().schedule_friday);

        // Saturday: Home until 24:00
        assert_eq!(
            DailySchedule {
                intervals: vec![
                    TimeInterval {
                        ends_at_hour: 24,
                        ends_at_minute: 0,
                        setting: TimeSetting::Home,
                    },
                ]
            }, create_parsed_thermostat_with_schedule().schedule_saturday);

        // Sunday: Home until 03:30 - Away until 20:30 - Home until 24:00
        assert_eq!(
            DailySchedule {
                intervals: vec![
                    TimeInterval {
                        ends_at_hour: 3,
                        ends_at_minute: 30,
                        setting: TimeSetting::Home,
                    },
                    TimeInterval {
                        ends_at_hour: 20,
                        ends_at_minute: 30,
                        setting: TimeSetting::Away,
                    },
                    TimeInterval {
                        ends_at_hour: 24,
                        ends_at_minute: 0,
                        setting: TimeSetting::Home,
                    },
                ]
            }, create_parsed_thermostat_with_schedule().schedule_sunday);
    }

    fn create_parsed_thermostat() -> ParsedThermostat {
        let thermostat = Thermostat {
            serial: "0:04:2F:06:24:D1".to_string(),
            secret: vec![215u8, 91, 125, 126, 14, 118, 62, 143, 121, 48, 110, 175, 112, 218, 245, 65],
            name: vec![177u8, 174, 159, 196, 58, 140, 76, 22, 18, 192, 117, 144, 240, 100, 45, 250],
            temperature: vec![7u8, 148, 108, 151, 150, 177, 75, 43],
            settings: vec![23u8, 243, 171, 192, 165, 81, 175, 118, 209, 79, 41, 151, 155, 212, 21, 255],
            schedule_1: vec![10u8, 152, 79, 196, 233, 136, 156, 34, 203, 230, 55, 201, 151, 192, 235, 253, 190, 155, 204, 38],
            schedule_2: vec![197u8, 163, 198, 34, 14, 212, 18, 186, 82, 212, 133, 156],
            schedule_3: vec![197u8, 163, 198, 34, 14, 212, 18, 186, 82, 212, 133, 156],

            ..Default::default()
        };

        ParsedThermostat::from_thermostat(&thermostat)
    }

    fn create_parsed_thermostat_with_schedule() -> ParsedThermostat {
        let thermostat = Thermostat {
            serial: "0:04:2F:06:24:D1".to_string(),
            secret: vec![215u8, 91, 125, 126, 14, 118, 62, 143, 121, 48, 110, 175, 112, 218, 245, 65],
            name: vec![177u8, 174, 159, 196, 58, 140, 76, 22, 18, 192, 117, 144, 240, 100, 45, 250],
            temperature: vec![206u8, 158, 231, 129, 243, 102, 119, 22],
            settings: vec![180u8, 249, 230, 196, 18, 146, 189, 34, 145, 102, 24, 26, 151, 111, 192, 189],
            schedule_1: vec![177u8, 191, 223, 32, 127, 196, 137, 136, 213, 11, 205, 247, 71, 30, 49, 92, 247, 241, 236, 206],
            schedule_2: vec![220u8, 194, 171, 34, 228, 17, 4, 228, 108, 49, 152, 155],
            schedule_3: vec![98u8, 242, 118, 159, 179, 69, 44, 123, 193, 42, 33, 37],

            ..Default::default()
        };

        ParsedThermostat::from_thermostat(&thermostat)
    }

    fn create_parsed_thermostat_with_vacation_schedule() -> ParsedThermostat {
        let thermostat = Thermostat {
            serial: "0:04:2F:06:24:D1".to_string(),
            secret: vec![215u8, 91, 125, 126, 14, 118, 62, 143, 121, 48, 110, 175, 112, 218, 245, 65],
            name: vec![177u8, 174, 159, 196, 58, 140, 76, 22, 18, 192, 117, 144, 240, 100, 45, 250],
            temperature: vec![87u8, 121, 70, 227, 189, 210, 0, 110],
            settings: vec![37u8, 26, 221, 64, 72, 6, 76, 45, 16, 198, 251, 77, 55, 244, 69, 9],
            schedule_1: vec![177u8, 191, 223, 32, 127, 196, 137, 136, 213, 11, 205, 247, 71, 30, 49, 92, 247, 241, 236, 206],
            schedule_2: vec![220u8, 194, 171, 34, 228, 17, 4, 228, 108, 49, 152, 155],
            schedule_3: vec![98u8, 242, 118, 159, 179, 69, 44, 123, 193, 42, 33, 37],

            ..Default::default()
        };

        ParsedThermostat::from_thermostat(&thermostat)
    }


    fn create_parsed_thermostat_with_planned_vacation() -> ParsedThermostat {
        let thermostat = Thermostat {
            serial: "0:04:2F:06:24:D1".to_string(),
            secret: vec![215u8, 91, 125, 126, 14, 118, 62, 143, 121, 48, 110, 175, 112, 218, 245, 65],
            name: vec![177u8, 174, 159, 196, 58, 140, 76, 22, 18, 192, 117, 144, 240, 100, 45, 250],
            temperature: vec![206u8, 158, 231, 129, 243, 102, 119, 22],
            settings: vec![38u8, 253, 23, 96, 139, 92, 198, 149, 168, 5, 146, 197, 239, 37, 35, 118],
            schedule_1: vec![177u8, 191, 223, 32, 127, 196, 137, 136, 213, 11, 205, 247, 71, 30, 49, 92, 247, 241, 236, 206],
            schedule_2: vec![220u8, 194, 171, 34, 228, 17, 4, 228, 108, 49, 152, 155],
            schedule_3: vec![98u8, 242, 118, 159, 179, 69, 44, 123, 193, 42, 33, 37],

            ..Default::default()
        };

        ParsedThermostat::from_thermostat(&thermostat)
    }
}
