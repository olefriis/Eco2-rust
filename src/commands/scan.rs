use std::time::Duration;

#[path = "../bluetooth.rs"]
mod bluetooth;

pub fn execute(arguments: Vec<String>) {
    eprintln!("Scanning for 2 minutes. Please wait.");
    let peripherals = bluetooth::scan(Duration::from_secs(120));

    let mut peripheral_found = false;
    for peripheral in peripherals.iter() {
        if peripheral.name.ends_with(";eTRV") {
            println!("{} - address: {}", peripheral.name, peripheral.address);
            peripheral_found = true
        }
    }

    if !peripheral_found {
        eprintln!("No thermostats found");
        std::process::exit(1)
    }
}
