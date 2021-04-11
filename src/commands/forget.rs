use crate::models::thermostats::Thermostats;

pub fn execute(arguments: Vec<String>) {
    if arguments.len() != 1 {
        panic!("Expected just one parameter to forget. Got {}.", arguments.len());
    }

    let serial = &arguments[0];

    let mut thermostats = Thermostats::load().expect("Could not read thermostat data");

    if thermostats.get(serial).is_none() {
        eprintln!("Uknown thermostat with serial {}", serial);
        std::process::exit(1);
    }

    thermostats.delete(serial);
    thermostats.save().unwrap();
}
