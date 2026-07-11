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
            use std::io::Write;

            let mut auto_route = false;
            let mut override_ssh = None;

            if args.len() > 2 {
                match args[2].as_str() {
                    "--auto" | "-a" | "--non-interactive" => {
                        auto_route = true;
                    }
                    "--host" if args.len() > 3 => {
                        let target_host = &args[3];
                        if let Some(ssh) = &config.ssh {
                            if ssh.host == *target_host {
                                override_ssh = Some(ssh.clone());
                            }
                        }
                        if override_ssh.is_none() {
                            if let Some(targets) = &config.ssh_targets {
                                for t in targets {
                                    if t.host == *target_host || t.match_pattern.as_ref().map_or(false, |p| p == target_host) {
                                        override_ssh = Some(t.clone());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                // Interactive Selection Menu
                if let Some(targets) = &config.ssh_targets {
                    if !targets.is_empty() {
                        println!("=== img2cli SSH Host Selection ===");
                        println!("Detected SSH hosts in your configuration:");
                        
                        let mut choices = Vec::new();
                        
                        if let Some(default_ssh) = &config.ssh {
                            if default_ssh.enabled {
                                println!("  [1] {} ({}:{}) [Default]", default_ssh.host, default_ssh.host, default_ssh.port.unwrap_or(22));
                                choices.push((Some(default_ssh.clone()), false));
                            }
                        }
                        
                        for target in targets {
                            let idx = choices.len() + 1;
                            println!("  [{}] {} ({}:{})", idx, target.match_pattern.as_ref().unwrap_or(&target.host), target.host, target.port.unwrap_or(22));
                            choices.push((Some(target.clone()), false));
                        }
                        
                        let auto_idx = choices.len() + 1;
                        println!("  [{}] [Auto Route] Automatically switch based on active window title", auto_idx);
                        choices.push((None, true));
                        
                        print!("Select target host to use (1-{}) [default: {}]: ", auto_idx, auto_idx);
                        std::io::stdout().flush().unwrap();
                        
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();
                        let choice = input.trim().parse::<usize>().unwrap_or(auto_idx);
                        
                        if choice >= 1 && choice <= choices.len() {
                            let (selected_ssh, auto) = &choices[choice - 1];
                            override_ssh = selected_ssh.clone();
                            auto_route = *auto;
                        } else {
                            auto_route = true;
                        }
                        
                        if auto_route {
                            println!("Starting in [Auto Route] mode...");
                        } else if let Some(ssh) = &override_ssh {
                            println!("Starting and locked to host: {}", ssh.host);
                        }
                    } else {
                        auto_route = true;
                    }
                } else {
                    auto_route = true;
                }
            }

            println!("Running img2cli in foreground...");
            if let Err(e) = daemon::run_service(&config, override_ssh, auto_route) {
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
