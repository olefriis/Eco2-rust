use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Thermostats {
    pub thermostats: Vec<Thermostat>,
}

impl Thermostats {
    pub fn save(&self) -> std::io::Result<()> {
        let serialized_thermostats = serde_json::to_string(&self)?;
        let file_path = Thermostats::file_path()?;
        fs::write(&file_path[..], &serialized_thermostats[..])?;
        Ok(())
    }

    pub fn load() -> std::io::Result<Thermostats> {
        let file_path = Thermostats::file_path()?;
        if Path::new(&file_path[..]).exists() {
            let serialized_thermostats = fs::read_to_string(&file_path[..])?;
            Ok(serde_json::from_str(&serialized_thermostats[..])?)
        } else {
            Ok(Thermostats {
                thermostats: vec![]
            })
        }
    }

    pub fn get(&self, serial: &String) -> Option<&Thermostat> {
        self.thermostats.iter().find(|thermostat| &thermostat.serial == serial)
    }

    pub fn push(&mut self, thermostat: Thermostat) {
        // Get rid of existing thermostats with the same serial
        self.thermostats.retain(|t| t.serial != thermostat.serial);

        // Then add the new thermostat
        self.thermostats.push(thermostat);
    }

    pub fn delete(&mut self, serial: &String) {
        self.thermostats.retain(|t| &t.serial != serial);
    }

    #[cfg(test)]
    fn file_path() -> Result<String, std::io::Error> {
        Ok("./.test-thermostats.json".to_string())
    }

    #[cfg(not(test))]
    fn file_path() -> Result<String, std::io::Error> {
        match env::home_dir() {
            Some(path) => Ok(format!(
                "{}/.thermostats.json",
                path.into_os_string().into_string().unwrap()
            )),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not find home directory",
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Thermostat {
    pub serial: String,
    pub secret: Vec<u8>,
    pub name: Vec<u8>,
    //pub battery_level: Vec<u8>,
    pub temperature: Vec<u8>,
    pub settings: Vec<u8>,
    pub schedule_1: Vec<u8>,
    pub schedule_2: Vec<u8>,
    pub schedule_3: Vec<u8>,

    // New values that haven't yet been saved to the thermostat
    pub new_set_point_temperature: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_serialize_and_deserialize() -> std::io::Result<()> {
        let thermostats = create_test_data();
        let serialized_thermostats = serde_json::to_string(&thermostats)?;

        let deserialized_thermostats: Thermostats = serde_json::from_str(&serialized_thermostats)?;
        assert_eq!(2, deserialized_thermostats.thermostats.len());

        let deserialized_thermostat = &deserialized_thermostats.thermostats[0];
        assert_eq!("12345", deserialized_thermostat.serial);
        assert_eq!(vec![1u8, 2, 255, 97], deserialized_thermostat.secret);

        Ok(())
    }

    #[test]
    fn it_can_save_and_load() -> std::io::Result<()> {
        let thermostats = create_test_data();
        thermostats.save()?;

        let loaded_thermostats = Thermostats::load()?;
        assert_eq!(2, loaded_thermostats.thermostats.len());

        Ok(())
    }

    #[test]
    fn it_can_give_existing_thermostat_with_serial() {
        let thermostats = create_test_data();

        let thermostat = thermostats.get(&"67890".to_string());
        assert_eq!(true, thermostat.is_some());
        assert_eq!(vec![5u8, 4, 253, 91], thermostat.unwrap().name);
    }

    #[test]
    fn it_gives_none_if_getting_a_nonexistent_thermostat() {
        let thermostats = create_test_data();

        assert_eq!(true, thermostats.get(&"19293".to_string()).is_none());
    }

    #[test]
    fn it_can_add_new_thermostat() {
        let mut thermostats = create_test_data();
        let new_thermostat = Thermostat {
            serial: "98765".to_string(),
            secret: vec![93u8, 1],
            name: vec![5u8, 4],
            temperature: vec![1u8, 2],
            settings: vec![1u8, 2],
            schedule_1: vec![1u8, 2],
            schedule_2: vec![1u8, 2],
            schedule_3: vec![1u8, 2],

            ..Default::default()
        };
        thermostats.push(new_thermostat);

        assert_eq!(3, thermostats.thermostats.len());
        let serials: Vec<String> = thermostats.thermostats.iter().map(|thermostat| thermostat.serial.clone()).collect();
        assert_eq!(true, serials.contains(&"12345".to_string()));
        assert_eq!(true, serials.contains(&"67890".to_string()));
        assert_eq!(true, serials.contains(&"98765".to_string()));
    }

    #[test]
    fn it_can_update_existing_thermostat() {
        let mut thermostats = create_test_data();

        let mut thermostat = thermostats.get(&"67890".to_string()).unwrap().clone();
        thermostat.secret = vec![7u8, 8, 9];
        thermostats.push(thermostat);

        assert_eq!(2, thermostats.thermostats.len());
        let serials: Vec<String> = thermostats.thermostats.iter().map(|thermostat| thermostat.serial.clone()).collect();
        assert_eq!(true, serials.contains(&"12345".to_string()));
        assert_eq!(true, serials.contains(&"67890".to_string()));
        assert_eq!(vec![7u8, 8, 9], thermostats.get(&"67890".to_string()).unwrap().secret);
    }

    #[test]
    fn it_can_delete_existing_thermostat() {
        let mut thermostats = create_test_data();

        thermostats.delete(&"12345".to_string());

        assert_eq!(1, thermostats.thermostats.len());
        assert_eq!(false, thermostats.get(&"12345".to_string()).is_some());
        assert_eq!(true, thermostats.get(&"67890".to_string()).is_some());
    }

    fn create_test_data() -> Thermostats {
        let thermostat1 = Thermostat {
            serial: "12345".to_string(),
            secret: vec![1u8, 2, 255, 97],
            name: vec![3u8, 3, 254, 95],
            temperature: vec![1u8, 2, 3, 4],
            settings: vec![1u8, 2, 3, 4],
            schedule_1: vec![1u8, 2, 3, 4],
            schedule_2: vec![1u8, 2, 3, 4],
            schedule_3: vec![1u8, 2, 3, 4],

            ..Default::default()
        };
        let thermostat2 = Thermostat {
            serial: "67890".to_string(),
            secret: vec![93u8, 1, 9],
            name: vec![5u8, 4, 253, 91],
            temperature: vec![1u8, 2, 3, 4],
            settings: vec![1u8, 2, 3, 4],
            schedule_1: vec![1u8, 2, 3, 4],
            schedule_2: vec![1u8, 2, 3, 4],
            schedule_3: vec![1u8, 2, 3, 4],

            ..Default::default()
        };
        Thermostats {
            thermostats: vec![thermostat1, thermostat2],
        }
    }
}
