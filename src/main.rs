




use config::get_config;
use pen::hippowm::run;
mod config;
mod pen;
mod bar;


fn main() {
    let config = get_config();
    run(config).unwrap();
    //// Initialize logging with env_logger
    //if let Err(err) = env_logger::init() {
        //eprintln!("Failed to initialize logger: {}", err);
        //process::exit(libc::EXIT_FAILURE);
    //}
//
    //// Create WindowManager instance using pattern matching
    //match Xwm::create() {
        //Ok(window_manager) => {
            //// Run the WindowManager
            //window_manager.run();
        //}
        //Err(error) => {
            //eprintln!("Failed to initialize window manager: {}", error);
            //process::exit(libc::EXIT_FAILURE)
        //}
    //}
}
