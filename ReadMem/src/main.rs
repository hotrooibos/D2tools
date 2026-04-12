mod d2r;
mod player;
mod process;

use process::wait_for_process;

use std::io::{self, Write};
use std::time::Duration;

const REFRESH_RATE_MS: u64 = 100;
const PROCESS_NAMES: &[&str] = &["D2R", "D2R.exe", "Diablo II Resurrected"];

fn main() {
    println!("==============================================");
    println!("  Diablo 2 Resurrected - Health Reader");
    println!("==============================================");
    println!();
    println!("Looking for D2R...");
    println!("(Make sure the game is running!)");
    println!();

    let process = match find_d2r_process() {
        Some(p) => p,
        None => {
            eprintln!("ERROR: Could not find D2R process.");
            eprintln!("Make sure Diablo 2 Resurrected is running.");
            eprintln!("(Searching for: {:?})", PROCESS_NAMES);
            std::process::exit(1);
        }
    };

    println!("Found D2R process!");
    println!();
    println!("Press Ctrl+C to exit");
    println!("==============================================");
    println!();

    loop {
        match read_and_display(&process) {
            Ok(in_game) => {
                if !in_game {
                    print_status("Not in game - return to town or a game");
                }
            }
            Err(e) => {
                print_status(&format!("Error: {}", e));
            }
        }

        std::thread::sleep(Duration::from_millis(REFRESH_RATE_MS));
    }
}

fn read_and_display(process: &process::Process) -> Result<bool, String> {
    let version = d2r::detect_version(process);
    
    let player = match version {
        Some(v) => {
            match player::read_player(process, v) {
                Some(p) => p,
                None => return Err("Could not read player data".to_string()),
            }
        }
        None => return Err("Could not detect D2R version".to_string()),
    };

    if !player.in_game {
        return Ok(false);
    }

    let hp_pct = if player.max_hp > 0 {
        (player.current_hp as f64 / player.max_hp as f64) * 100.0
    } else {
        0.0
    };

    let mana_pct = if player.max_mana > 0 {
        (player.current_mana as f64 / player.max_mana as f64) * 100.0
    } else {
        0.0
    };

    print!("\r");

    print!("[{}] ", player.name);
    
    print!("HP: {}/{} ({:5.1}%) | ", 
           player.current_hp, player.max_hp, hp_pct);
    
    print!("MP: {}/{} ({:5.1}%)", 
           player.current_mana, player.max_mana, mana_pct);

    io::stdout().flush().ok();

    Ok(true)
}

fn print_status(msg: &str) {
    print!("\r{:60}\r{}", "", msg);
    io::stdout().flush().ok();
}

fn find_d2r_process() -> Option<process::Process> {
    println!("Searching for D2R processes...");
    
    for name in PROCESS_NAMES {
        if let Some(p) = wait_for_process(name, Some(2)) {
            println!("Found process matching '{}'", name);
            return Some(p);
        }
    }
    
    println!("Generic D2R search...");
    if let Some(p) = wait_for_process("D2R", Some(60)) {
        println!("Found D2R process!");
        return Some(p);
    }
    
    None
}
