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

#[derive(Serialize, Deserialize, Debug, Clone)]
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
        };
        Thermostats {
            thermostats: vec![thermostat1, thermostat2],
        }
    }
}
