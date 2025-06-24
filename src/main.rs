mod command;
mod error;
mod state;
mod ui;
mod util;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short = 'n', long, default_value_t = 1.0)]
    interval: f64,

    #[arg(short, long, default_value_t = 1)]
    concurrency: u8,

    #[arg(last = true)]
    command: Vec<String>,
}

fn main() -> error::BodaResult<()> {
    util::log::setup();
    let cli = Cli::parse();

    let tempdir = std::env::temp_dir();
    let tempfile = ulid::Ulid::new().to_string();
    let filepath = tempdir.join(format!("{}.sqlite", tempfile));

    let state_manager = state::manager::Manager::new(cli, &filepath);
    let (command_manger, command_action_rx) = command::manager::Manager::new();
    let (ui_manager, ui_action_rx) = ui::manager::Manager::new();

    let handles = [
        command_manger.run(state_manager.state.clone()),
        ui_manager.run(state_manager.state.clone()),
        state_manager.run(ui_action_rx, command_action_rx),
    ];
    for handle in handles {
        handle.join().expect("unable to join thread");
    }
    println!("Backup at {:?}", filepath);
    Ok(())
}
