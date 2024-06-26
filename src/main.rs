use config::get_config;
use pen::hippowm::run;
mod bar;
mod config;
mod pen;
use x::xwm::Xwm;
mod x;
use env_logger;
use libc;
use std::env::args;
use std::{env, process};
use std::fmt::Pointer;
use glib::property::PropertyGet;

fn main() {
    //let config = get_config();
    //run(config).unwrap();
    //This is for the x implementation
    // Initialize logging with env_logger

    env_logger::init();

    // Create WindowManager instance using pattern matching
    let window_manager = match Xwm::create("") {
        Ok(window_manager) => window_manager,
        Err(error) => {
            eprintln!("Failed to initialize window manager: {}", error);
            process::exit(libc::EXIT_FAILURE)
        }
    };

    window_manager.borrow().run();
}
