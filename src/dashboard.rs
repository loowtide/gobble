use crossterm::style::{Color, Stylize, style};
use std::env;
use std::error::Error;
use sysinfo::System;
pub fn print_dashboard() -> Result<(), Box<dyn Error>> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let hostname = System::host_name().unwrap();
    let os = System::name().unwrap();
    let kernel = System::kernel_version().unwrap();
    let uptime = System::uptime();
    let cpu = sys
        .cpus()
        .get(0)
        .map(|c| c.brand().to_string())
        .unwrap_or("Unknown CPU".into());
    let total_mem = sys.total_memory() / 1024;
    let used_mem = sys.used_memory() / 1024;

    let hrs = uptime / 3600;
    let mins = (uptime % 3600) / 60;

    let art = [
        style("      ████████").with(Color::Cyan).to_string(),
        style("    ███      ███").with(Color::Blue).to_string(),
        style("   ███        ███").with(Color::Blue).to_string(),
        style("   ███  Gobble ███")
            .with(Color::Magenta)
            .bold()
            .to_string(),
        style("   ███        ███").with(Color::Blue).to_string(),
        style("    ███      ███").with(Color::Blue).to_string(),
        style("      ████████").with(Color::Cyan).to_string(),
    ];

    println!();
    println!();

    let info = vec![
        (style(" Host:").with(Color::Green), hostname),
        (
            style(" OS:").with(Color::Green),
            format!("{} (kernel {})", os, kernel),
        ),
        (
            style(" Uptime:").with(Color::Green),
            format!("{}h {}m", hrs, mins),
        ),
        (style(" Shell:").with(Color::Green), "GobbleShell".into()),
        (style(" CPU:").with(Color::Green), cpu),
        (
            style(" Memory:").with(Color::Green),
            format!("{}MB / {}MB", used_mem, total_mem),
        ),
        (
            style(" Directory:").with(Color::Green),
            env::current_dir().unwrap().display().to_string(),
        ),
    ];

    for (i, line) in art.iter().enumerate() {
        if i < info.len() {
            println!("{:<25} {} {}", line, info[i].0, info[i].1);
        } else {
            println!("{}", line);
        }
    }

    println!();
    println!(
        "{}",
        style("Welcome to Gobble Shell!").with(Color::Green).bold()
    );
    println!();
    return Ok(());
}
