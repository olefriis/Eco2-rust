extern crate serde;
#[macro_use]
extern crate log;

mod commands;
mod models;
use commands::{read, scan, show};

fn main() {
    env_logger::init();
    info!("starting up");

    let mut command_line_arguments = std::env::args();
    let program = command_line_arguments.next().expect("Program missing...");
    let command = command_line_arguments
        .next()
        .expect("No command given. Try using help.");
    let command_arguments: Vec<String> = command_line_arguments.collect();
    match command.as_str() {
        "scan" => scan::execute(command_arguments),
        "read" => read::execute(command_arguments),
        "write" => commands::write(command_arguments),
        "forget" => commands::forget(command_arguments),
        "list" => commands::list(command_arguments),
        "show" => show::execute(command_arguments),
        "set" => commands::set(command_arguments),
        "help" => quit_with_usage(program.as_str(), 0),
        _ => {
            println!("Unknown command {}", command);
            quit_with_usage(program.as_str(), 1)
        }
    }
}

fn quit_with_usage(program: &str, exit_code: i32) {
    println!("Usage: {} command [arguments]", program);
    println!("");
    println!("Commands:");
    println!("scan - scan nearby devices for 120 seconds (Ctrl-C to stop)");
    println!("read name - connect to and read specific thermostat");
    println!("write name - connect to specific thermostat and write all values");
    println!("forget name - forget about a specific thermostat");
    println!("list - show all of the previously read thermostats");
    println!("show name - output all previously read values from a thermostat");
    println!("set name attribute value - set the given attribute to the provided value");

    std::process::exit(exit_code)
}
