// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

mod db_operations;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let conn: rusqlite::Connection = db_operations::get_db()?;

    let ui = MainWindow::new()?;

    slint::select_bundled_translation("en")?;

    ui.run()?;

    Ok(())
}
