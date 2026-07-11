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

            println!("=== img2cli v0.1.4 ===");
            println!("Welcome to img2cli - Clipboard screenshot helper for remote CLIs!\n");

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
                        
                        let new_target_idx = choices.len() + 1;
                        println!("  [{}] [New Target] Add a new SSH host configuration", new_target_idx);
                        
                        print!("Select target host to use (1-{}) [default: {}]: ", new_target_idx, auto_idx);
                        std::io::stdout().flush().unwrap();
                        
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).unwrap();
                        let choice = input.trim().parse::<usize>().unwrap_or(auto_idx);
                        
                        if choice == new_target_idx {
                            println!("\n--- Add New SSH Host Configuration ---");
                            
                            print!("Enter SSH Host (IP, Domain or SSH Alias): ");
                            std::io::stdout().flush().unwrap();
                            let mut host = String::new();
                            std::io::stdin().read_line(&mut host).unwrap();
                            let host = host.trim().to_string();
                            
                            if host.is_empty() {
                                eprintln!("Error: SSH Host cannot be empty!");
                                std::process::exit(1);
                            }
                            
                            print!("Enter SSH Port [default: 22]: ");
                            std::io::stdout().flush().unwrap();
                            let mut port_str = String::new();
                            std::io::stdin().read_line(&mut port_str).unwrap();
                            let port = port_str.trim().parse::<u16>().ok();
                            
                            print!("Enter SSH Username: ");
                            std::io::stdout().flush().unwrap();
                            let mut username = String::new();
                            std::io::stdin().read_line(&mut username).unwrap();
                            let username_opt = if username.trim().is_empty() { None } else { Some(username.trim().to_string()) };
                            
                            print!("Enter Window Title Match Pattern (e.g. S97): ");
                            std::io::stdout().flush().unwrap();
                            let mut pattern = String::new();
                            std::io::stdin().read_line(&mut pattern).unwrap();
                            let pattern_opt = if pattern.trim().is_empty() { Some(host.clone()) } else { Some(pattern.trim().to_string()) };
                            
                            let default_remote_dir = config.ssh.as_ref().map(|s| s.remote_dir.clone()).unwrap_or_else(|| "/tmp/img2cli".to_string());
                            print!("Enter Remote Directory [default: {}]: ", default_remote_dir);
                            std::io::stdout().flush().unwrap();
                            let mut remote_dir = String::new();
                            std::io::stdin().read_line(&mut remote_dir).unwrap();
                            let remote_dir = if remote_dir.trim().is_empty() { default_remote_dir } else { remote_dir.trim().to_string() };
                            
                            let new_ssh = crate::config::SshConfig {
                                enabled: true,
                                host: host.clone(),
                                port,
                                username: username_opt,
                                remote_dir,
                                match_pattern: pattern_opt,
                            };
                            
                            let mut updated_config = config.clone();
                            let mut targets = updated_config.ssh_targets.unwrap_or_default();
                            targets.push(new_ssh.clone());
                            updated_config.ssh_targets = Some(targets);
                            
                            let config_path = crate::config::Config::config_file_path();
                            if let Ok(toml_str) = toml::to_string(&updated_config) {
                                if std::fs::write(&config_path, toml_str).is_ok() {
                                    println!("New SSH host successfully saved to config.toml!");
                                } else {
                                    eprintln!("Warning: Failed to save new host to config file.");
                                }
                            }
                            
                            override_ssh = Some(new_ssh);
                            auto_route = false;
                        } else if choice >= 1 && choice <= choices.len() {
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
