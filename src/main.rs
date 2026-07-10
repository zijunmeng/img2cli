mod config;
mod clipboard;
mod keyboard;
mod daemon;
mod utils;

use config::Config;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return;
    }

    let config = Config::load();

    match args[1].as_str() {
        "start" => {
            if let Err(e) = daemon::start_daemon(&config) {
                eprintln!("Error starting daemon: {}", e);
                std::process::exit(1);
            }
        }
        "run" => {
            println!("Running img2cli in foreground...");
            if let Err(e) = daemon::run_service(&config) {
                eprintln!("Error running service: {}", e);
                std::process::exit(1);
            }
        }
        "stop" => {
            if let Err(e) = daemon::stop_daemon() {
                eprintln!("Error stopping daemon: {}", e);
                std::process::exit(1);
            }
        }
        "status" => {
            if let Err(e) = daemon::check_status() {
                eprintln!("Error checking status: {}", e);
                std::process::exit(1);
            }
        }
        "clean" => {
            let save_dir = config.get_save_dir();
            println!("Cleaning files older than 24 hours in {:?}...", save_dir);
            match utils::clean_old_files(&save_dir, 24) {
                Ok(count) => println!("Cleaned {} file(s).", count),
                Err(e) => eprintln!("Error during clean: {}", e),
            }
        }
        "help" | "-h" | "--help" => {
            print_help();
        }
        unknown => {
            eprintln!("Unknown command: {}", unknown);
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(
        "img2cli - Clipboard screenshot helper for AI Agent CLIs\n\n\
Usage:\n  \
  img2cli <command>\n\n\
Commands:\n  \
  start      Start the monitoring service in the background (as a daemon)\n  \
  run        Run the monitoring service in the foreground\n  \
  stop       Stop the background daemon\n  \
  status     Check the status of the daemon process\n  \
  clean      Manually clean temporary image files older than 24 hours\n  \
  help       Show this help information\n\n\
Configuration path: ~/.config/img2cli/config.toml"
    );
}
