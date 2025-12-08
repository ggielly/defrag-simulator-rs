use clap::Parser;
use defrag_simulator_rs::{app, ui};
use std::io::Result;

use defrag_simulator_rs::{constants::defrag_type::DefragStyle, graphics};

fn main() -> Result<()> {
    let args = app::Args::parse();
    let (width, height) = app::parse_size(&args.size).unwrap_or((78, 16));
    let ui_style = args.get_ui_style();

    // Check if we should use graphical mode for Win98/Win95
    if matches!(
        ui_style,
        defrag_simulator_rs::constants::defrag_type::DefragStyle::Windows98
            | defrag_simulator_rs::constants::defrag_type::DefragStyle::Windows95
    ) {
        // Run graphical mode (required for Win98/Win95)

        let mut app = app::App::new(width, height, args.fill, args.sound, args.drive, ui_style);

        if let Err(e) = graphics::win98_renderer::run_win98_graphical(&mut app) {
            eprintln!("Graphical mode failed: {}", e);
            std::process::exit(1); // Exit with error as Win98/Win95 requires graphical mode
        } else {
            return Ok(());
        }
    }

    // Terminal mode (MS-DOS style)
    use defrag_simulator_rs::ui;

    // Setup terminal
    let mut tui = ui::TuiWrapper::new()?;

    // Setup Ctrl+C handler
    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        tx.send(()).expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");

    // Create and run app with selected UI style
    let mut app = app::App::new(width, height, args.fill, args.sound, args.drive, ui_style);
    app.run(&mut tui, rx)?;

    // Restore terminal
    tui.cleanup()?;
    Ok(())
}
