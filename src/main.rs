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
    let cli = Cli::parse();

    let state_manager = state::manager::Manager::new(cli.interval, cli.concurrency);
    let (command_manger, command_rx) = command::manager::Manager::new(cli.command);
    let (ui_manager, action_rx) = ui::manager::Manager::new();

    let handles = [
        command_manger.run(state_manager.state.clone()),
        ui_manager.run(state_manager.state.clone()),
        state_manager.run(action_rx, command_rx),
    ];
    for handle in handles {
        handle.join().expect("unable to join thread");
    }
    Ok(())

    // let stdin = io::stdin();
    // if !stdin.is_tty() {
    //     return Err(error::BodaError::Custom(
    //         "Not an interactive terminal".to_string(),
    //     ));
    // }

    // let mut w = io::stdout();
    // execute!(w, terminal::EnterAlternateScreen)?;
    // terminal::enable_raw_mode()?;

    // let tick = tick(Duration::from_millis((interval * 1000.0).round() as u64));
    // let (end_tx, end_rx) = bounded::<bool>(1);

    // let key_handler = thread::spawn(move || {
    //     handle_keys(end_tx).unwrap();
    // });

    // loop {
    //     select! {
    //         recv(end_rx) -> _ => {
    //             execute!(
    //                 w,
    //                 cursor::SetCursorStyle::DefaultUserShape,
    //             )?;
    //             break;
    //         },
    //         recv(tick) -> _ => {
    //             let now = chrono::Local::now();
    //             let output = match command::run(shell.clone(), cli.command.clone()) {
    //                 Ok(output) => output,
    //                 Err(e) => format!("error: {}", e),
    //             };
    //             queue!(
    //                 w,
    //                 style::ResetColor,
    //                 terminal::Clear(terminal::ClearType::All),
    //                 cursor::Hide,
    //                 cursor::MoveTo(0, 0),
    //                 style::Print(now),
    //                 cursor::MoveToNextLine(1),
    //                 style::Print("==============".to_string()),
    //             )?;

    //             for line in output.lines() {
    //                 queue!(
    //                     w,
    //                     cursor::MoveToNextLine(1),
    //                     style::Print(line.to_string()),
    //                 )?;
    //             };

    //             w.flush()?;
    //         }
    //     }
    // }

    // execute!(
    //     w,
    //     style::ResetColor,
    //     cursor::Show,
    //     terminal::LeaveAlternateScreen
    // )?;
    // terminal::disable_raw_mode()?;
    // if let Err(_) = key_handler.join() {
    //     return Err(error::BodaError::Custom(
    //         "failed to join key_handler".to_string(),
    //     ));
    // };

    // Ok(())
}
