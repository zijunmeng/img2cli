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

    let mut config = Config::load();

    match args[1].as_str() {
        "start" => {
            if let Err(e) = daemon::start_daemon(&config) {
                eprintln!("Error starting daemon: {}", e);
                std::process::exit(1);
            }
        }
        "run" => {
            use std::io::Write;

            println!("=== img2cli v0.3.0 ===");
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
                // Interactive Selection Loop
                let mut current_config = config.clone();
                
                let is_dup = |new_ssh: &crate::config::SshConfig, list: &[crate::config::SshConfig], default_ssh: &Option<crate::config::SshConfig>| -> bool {
                    let check = |item: &crate::config::SshConfig| -> bool {
                        let hosts_match = item.host == new_ssh.host 
                            || item.match_pattern.as_ref() == Some(&new_ssh.host)
                            || Some(&item.host) == new_ssh.match_pattern.as_ref()
                            || (item.match_pattern.is_some() && item.match_pattern == new_ssh.match_pattern);
                        hosts_match && item.port == new_ssh.port
                    };
                    if let Some(def) = default_ssh {
                        if check(def) {
                            return true;
                        }
                    }
                    for item in list {
                        if check(item) {
                            return true;
                        }
                    }
                    false
                };

                loop {
                    println!("\n=== img2cli SSH Host Selection ===");
                    println!("Detected SSH hosts in your configuration:");
                    
                    let mut choices = Vec::new();
                    
                    // 1. Default SSH Config
                    if let Some(default_ssh) = &current_config.ssh {
                        if default_ssh.enabled {
                            println!("  [1] {} ({}:{}) [Default]", default_ssh.host, default_ssh.host, default_ssh.port.unwrap_or(22));
                            choices.push((Some(default_ssh.clone()), false, "default".to_string(), 0));
                        }
                    }
                    
                    // 2. Custom ssh_targets
                    if let Some(targets) = &current_config.ssh_targets {
                        for (idx, target) in targets.iter().enumerate() {
                            let item_idx = choices.len() + 1;
                            println!("  [{}] {} ({}:{})", item_idx, target.match_pattern.as_ref().unwrap_or(&target.host), target.host, target.port.unwrap_or(22));
                            choices.push((Some(target.clone()), false, "target".to_string(), idx));
                        }
                    }
                    
                    let auto_idx = choices.len() + 1;
                    println!("  [{}] [Auto Route] Automatically switch based on active window title", auto_idx);
                    
                    let new_target_idx = choices.len() + 2;
                    println!("  [{}] [New Target] Add a new SSH host configuration", new_target_idx);
                    
                    let load_config_idx = choices.len() + 3;
                    println!("  [{}] [Load Config] Import hosts from local ~/.ssh/config", load_config_idx);
                    
                    let edit_target_idx = choices.len() + 4;
                    println!("  [{}] [Edit Target] Modify an existing host configuration", edit_target_idx);
                    
                    let delete_target_idx = choices.len() + 5;
                    println!("  [{}] [Delete Target] Remove a host configuration", delete_target_idx);
                    
                    print!("Select target host or action (1-{}) [default: {}]: ", delete_target_idx, auto_idx);
                    std::io::stdout().flush().unwrap();
                    
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    let trimmed = input.trim();
                    let mut choice = if trimmed.is_empty() { auto_idx } else { trimmed.parse::<usize>().unwrap_or(0) };
                    
                    while choice == 0 || choice > delete_target_idx {
                        print!("Invalid choice, please select a valid option (1-{}): ", delete_target_idx);
                        std::io::stdout().flush().unwrap();
                        let mut retry_input = String::new();
                        std::io::stdin().read_line(&mut retry_input).unwrap();
                        let retry_trimmed = retry_input.trim();
                        choice = if retry_trimmed.is_empty() { auto_idx } else { retry_trimmed.parse::<usize>().unwrap_or(0) };
                    }
                    
                    if choice == auto_idx {
                        auto_route = true;
                        override_ssh = None;
                        println!("Starting in [Auto Route] mode...");
                        break;
                    }
                    else if choice == new_target_idx {
                        println!("\n--- Add New SSH Host Configuration ---");
                        
                        print!("Enter SSH Host (IP, Domain or SSH Alias): ");
                        std::io::stdout().flush().unwrap();
                        let mut host = String::new();
                        std::io::stdin().read_line(&mut host).unwrap();
                        let host = host.trim().to_string();
                        
                        if host.is_empty() {
                            println!("Error: SSH Host cannot be empty!");
                            continue;
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
                        
                        let default_remote_dir = current_config.ssh.as_ref().map(|s| s.remote_dir.clone()).unwrap_or_else(|| "/tmp/img2cli".to_string());
                        print!("Enter Remote Directory [default: {}]: ", default_remote_dir);
                        std::io::stdout().flush().unwrap();
                        let mut remote_dir = String::new();
                        std::io::stdin().read_line(&mut remote_dir).unwrap();
                        let remote_dir = if remote_dir.trim().is_empty() { default_remote_dir } else { remote_dir.trim().to_string() };
                        
                        let new_ssh = crate::config::SshConfig {
                            enabled: true,
                            host,
                            port,
                            username: username_opt,
                            remote_dir,
                            match_pattern: pattern_opt,
                            remember_password: true,
                        };
                        
                        let mut targets = current_config.ssh_targets.clone().unwrap_or_default();
                        if is_dup(&new_ssh, &targets, &current_config.ssh) {
                            println!("Warning: A configuration for this host already exists. Duplicate skipped!");
                            continue;
                        }
                        targets.push(new_ssh);
                        current_config.ssh_targets = Some(targets);
                        
                        let config_path = crate::config::Config::config_file_path();
                        if let Ok(toml_str) = toml::to_string(&current_config) {
                            if std::fs::write(&config_path, toml_str).is_ok() {
                                println!("New SSH host successfully saved!");
                            }
                        }
                    }
                    else if choice == load_config_idx {
                        println!("\n--- Import Hosts from ~/.ssh/config ---");
                        let parsed_hosts = crate::utils::parse_ssh_config();
                        if parsed_hosts.is_empty() {
                            println!("No hosts found in ~/.ssh/config (or file does not exist).");
                            continue;
                        }
                        
                        println!("Detected hosts in ~/.ssh/config:");
                        for (i, h) in parsed_hosts.iter().enumerate() {
                            println!("  [{}] {} ({}:{})", i + 1, h.host, h.host, h.port.unwrap_or(22));
                        }
                        
                        print!("Select hosts to import (separate by commas, e.g. 1,3 or type 'all'): ");
                        std::io::stdout().flush().unwrap();
                        let mut import_input = String::new();
                        std::io::stdin().read_line(&mut import_input).unwrap();
                        let import_trimmed = import_input.trim();
                        
                        let mut imported_count = 0;
                        let mut targets = current_config.ssh_targets.clone().unwrap_or_default();
                        
                        let to_import = if import_trimmed == "all" {
                            parsed_hosts
                        } else {
                            let mut selected = Vec::new();
                            let indices: Vec<&str> = import_trimmed.split(',').collect();
                            for idx_str in indices {
                                if let Ok(idx) = idx_str.trim().parse::<usize>() {
                                    if idx >= 1 && idx <= parsed_hosts.len() {
                                        selected.push(parsed_hosts[idx - 1].clone());
                                    }
                                }
                            }
                            selected
                        };
                        
                        for h in to_import {
                            if is_dup(&h, &targets, &current_config.ssh) {
                                println!("Skipping duplicate host: {} ({})", h.host, h.match_pattern.as_ref().unwrap_or(&h.host));
                            } else {
                                targets.push(h);
                                imported_count += 1;
                            }
                        }
                        
                        if imported_count > 0 {
                            current_config.ssh_targets = Some(targets);
                            let config_path = crate::config::Config::config_file_path();
                            if let Ok(toml_str) = toml::to_string(&current_config) {
                                let _ = std::fs::write(&config_path, toml_str);
                                println!("Successfully imported {} host(s)!", imported_count);
                            }
                        } else {
                            println!("No new hosts were imported (all selected were duplicates).");
                        }
                    }
                    else if choice == edit_target_idx {
                        println!("\n--- Edit SSH Host Configuration ---");
                        if choices.is_empty() {
                            println!("No configurations to edit.");
                            continue;
                        }
                        
                        println!("Select configuration index to edit (1-{}):", choices.len());
                        for (i, item) in choices.iter().enumerate() {
                            let (ssh_opt, _, config_type, _) = item;
                            if let Some(ssh) = ssh_opt {
                                println!("  [{}] {} ({}) [type: {}]", i + 1, ssh.host, ssh.match_pattern.as_ref().unwrap_or(&ssh.host), config_type);
                            }
                        }
                        
                        print!("Index to edit: ");
                        std::io::stdout().flush().unwrap();
                        let mut edit_input = String::new();
                        std::io::stdin().read_line(&mut edit_input).unwrap();
                        if let Ok(idx) = edit_input.trim().parse::<usize>() {
                            if idx >= 1 && idx <= choices.len() {
                                let (ssh_opt, _, config_type, inner_idx) = &choices[idx - 1];
                                if let Some(ssh) = ssh_opt {
                                    println!("\nEditing: {} (Press Enter to keep existing value)", ssh.host);
                                    
                                    print!("Enter SSH Host [{}]: ", ssh.host);
                                    std::io::stdout().flush().unwrap();
                                    let mut new_host = String::new();
                                    std::io::stdin().read_line(&mut new_host).unwrap();
                                    let host_val = if new_host.trim().is_empty() { ssh.host.clone() } else { new_host.trim().to_string() };
                                    
                                    print!("Enter SSH Port [{}]: ", ssh.port.unwrap_or(22));
                                    std::io::stdout().flush().unwrap();
                                    let mut new_port = String::new();
                                    std::io::stdin().read_line(&mut new_port).unwrap();
                                    let port_val = if new_port.trim().is_empty() { ssh.port } else { new_port.trim().parse::<u16>().ok() };
                                    
                                    print!("Enter SSH Username [{}]: ", ssh.username.as_ref().unwrap_or(&"".to_string()));
                                    std::io::stdout().flush().unwrap();
                                    let mut new_user = String::new();
                                    std::io::stdin().read_line(&mut new_user).unwrap();
                                    let user_val = if new_user.trim().is_empty() { ssh.username.clone() } else { Some(new_user.trim().to_string()) };
                                    
                                    print!("Enter Window Title Match Pattern [{}]: ", ssh.match_pattern.as_ref().unwrap_or(&"".to_string()));
                                    std::io::stdout().flush().unwrap();
                                    let mut new_pattern = String::new();
                                    std::io::stdin().read_line(&mut new_pattern).unwrap();
                                    let pattern_val = if new_pattern.trim().is_empty() { ssh.match_pattern.clone() } else { Some(new_pattern.trim().to_string()) };
                                    
                                    print!("Enter Remote Directory [{}]: ", ssh.remote_dir);
                                    std::io::stdout().flush().unwrap();
                                    let mut new_remote_dir = String::new();
                                    std::io::stdin().read_line(&mut new_remote_dir).unwrap();
                                    let remote_dir_val = if new_remote_dir.trim().is_empty() { ssh.remote_dir.clone() } else { new_remote_dir.trim().to_string() };
                                    
                                    let updated_ssh = crate::config::SshConfig {
                                        enabled: true,
                                        host: host_val,
                                        port: port_val,
                                        username: user_val,
                                        remote_dir: remote_dir_val,
                                        match_pattern: pattern_val,
                                        remember_password: ssh.remember_password,
                                    };
                                    
                                    if config_type == "default" {
                                        current_config.ssh = Some(updated_ssh);
                                    } else {
                                        if let Some(ref mut targets) = current_config.ssh_targets {
                                            targets[*inner_idx] = updated_ssh;
                                        }
                                    }
                                    
                                    let config_path = crate::config::Config::config_file_path();
                                    if let Ok(toml_str) = toml::to_string(&current_config) {
                                        let _ = std::fs::write(&config_path, toml_str);
                                        println!("Configuration successfully updated!");
                                    }
                                }
                            } else {
                                println!("Invalid index.");
                            }
                        }
                    }
                    else if choice == delete_target_idx {
                        println!("\n--- Remove SSH Host Configuration ---");
                        if choices.is_empty() {
                            println!("No configurations to delete.");
                            continue;
                        }
                        
                        println!("Select configuration index to delete (separate by commas, e.g. 1,3 or type 'all'):");
                        for (i, item) in choices.iter().enumerate() {
                            let (ssh_opt, _, config_type, _) = item;
                            if let Some(ssh) = ssh_opt {
                                println!("  [{}] {} ({}) [type: {}]", i + 1, ssh.host, ssh.match_pattern.as_ref().unwrap_or(&ssh.host), config_type);
                            }
                        }
                        
                        print!("Index to delete: ");
                        std::io::stdout().flush().unwrap();
                        let mut delete_input = String::new();
                        std::io::stdin().read_line(&mut delete_input).unwrap();
                        let delete_trimmed = delete_input.trim();
                        
                        let mut indices_to_remove = Vec::new();
                        if delete_trimmed == "all" {
                            for i in 0..choices.len() {
                                indices_to_remove.push(i);
                            }
                        } else {
                            let parts: Vec<&str> = delete_trimmed.split(',').collect();
                            for part in parts {
                                if let Ok(idx) = part.trim().parse::<usize>() {
                                    if idx >= 1 && idx <= choices.len() {
                                        indices_to_remove.push(idx - 1);
                                    }
                                }
                            }
                        }
                        
                        // Sort descending to remove without shifting indices of remaining elements
                        indices_to_remove.sort_by(|a, b| b.cmp(a));
                        indices_to_remove.dedup();
                        
                        let mut deleted_count = 0;
                        for idx in indices_to_remove {
                            let (_, _, config_type, inner_idx) = &choices[idx];
                            if config_type == "default" {
                                current_config.ssh = None;
                                deleted_count += 1;
                            } else {
                                if let Some(ref mut targets) = current_config.ssh_targets {
                                    if *inner_idx < targets.len() {
                                        targets.remove(*inner_idx);
                                        deleted_count += 1;
                                    }
                                }
                            }
                        }
                        
                        if deleted_count > 0 {
                            let config_path = crate::config::Config::config_file_path();
                            if let Ok(toml_str) = toml::to_string(&current_config) {
                                let _ = std::fs::write(&config_path, toml_str);
                                println!("Successfully removed {} host configuration(s)!", deleted_count);
                            }
                        } else {
                            println!("No hosts were removed.");
                        }
                    }
                    else if choice >= 1 && choice <= choices.len() {
                        let (selected_ssh, auto, _, _) = &choices[choice - 1];
                        override_ssh = selected_ssh.clone();
                        auto_route = *auto;
                        if let Some(ssh) = &override_ssh {
                            println!("Starting and locked to host: {}", ssh.host);
                        }
                        break;
                    }
                    else {
                        println!("Invalid choice, please try again.");
                    }
                }
                
                config = current_config;
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
