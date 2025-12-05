use std::io::Result;
use clap::Parser;

mod resources;
mod audio;
mod app;
mod ui;
mod models;
pub mod constants;

// -- Main Application Logic ---------------------------------------------------

fn main() -> Result<()> {
    let args = app::Args::parse();
    let (width, height) = app::parse_size(&args.size).unwrap_or((78, 16));

    // Setup terminal
    let mut tui = ui::TuiWrapper::new()?;

    // Setup Ctrl+C handler
    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        tx.send(()).expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");

    // Create and run app
    let mut app = app::App::new(width, height, args.fill, args.sound, args.drive);
    app.run(&mut tui, rx)?;

    // Restore terminal
    tui.cleanup()?;
    Ok(())
}