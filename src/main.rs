use std::io::Result;
use clap::Parser;

mod audio;
mod app;
mod ui;
mod models;
pub mod constants;
pub mod win98;

#[cfg(feature = "graphical")]
pub mod graphics;

// -- Main Application Logic ---------------------------------------------------

fn main() -> Result<()> {
    let args = app::Args::parse();
    let (width, height) = app::parse_size(&args.size).unwrap_or((78, 16));
    let ui_style = args.get_ui_style();

    // Check if we should use graphical mode for Win98/Win95
    #[cfg(feature = "graphical")]
    {
        use crate::constants::defrag_type::DefragStyle;
        
        if matches!(ui_style, DefragStyle::Windows98 | DefragStyle::Windows95) {
            // Run graphical mode
            let mut app = app::App::new(width, height, args.fill, args.sound, args.drive, ui_style);
            
            if let Err(e) = graphics::win98_renderer::run_win98_graphical(&mut app) {
                eprintln!("Graphical mode failed: {}", e);
                eprintln!("Falling back to terminal mode...");
                // Fall through to terminal mode
            } else {
                return Ok(());
            }
        }
    }

    // Terminal mode (MS-DOS style or fallback)
    
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