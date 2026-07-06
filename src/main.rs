use clap::Parser;
use defrag_simulator_rs::app;
use std::io::Result;

#[cfg(feature = "graphical")]
use defrag_simulator_rs::{constants::defrag_type::DefragStyle, graphics};

#[cfg(not(feature = "graphical"))]
use defrag_simulator_rs::ui;

fn main() -> Result<()> {
    let args = app::Args::parse();
    let (width, height) = app::parse_size(&args.size).unwrap_or((78, 16));
    let ui_style = args.get_ui_style();

    // Check if we should use graphical mode for Win98/Win95
    #[cfg(feature = "graphical")]
    if matches!(ui_style, DefragStyle::Windows98 | DefragStyle::Windows95) {
        let mut app = app::App::new(width, height, args.fill, args.sound, args.drive, ui_style);

        if let Err(e) = graphics::win98_renderer::run_win98_graphical(&mut app) {
            eprintln!("Graphical mode failed: {}", e);
            std::process::exit(1);
        } else {
            return Ok(());
        }
    }

    // Terminal mode (MS-DOS style)
    let mut tui = ui::TuiWrapper::new()?;

    let (tx, rx) = std::sync::mpsc::channel();
    ctrlc::set_handler(move || {
        tx.send(()).expect("Could not send signal on channel.");
    })
    .expect("Error setting Ctrl-C handler");

    let mut app = app::App::new(width, height, args.fill, args.sound, args.drive, ui_style);
    app.run(&mut tui, rx)?;

    tui.cleanup()?;
    Ok(())
}
