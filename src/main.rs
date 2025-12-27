// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::rc::Rc;
use std::cell::RefCell;
use rusqlite::Connection;

mod db_operations;
mod app_controller;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let conn: Rc<RefCell<Connection>> = db_operations::get_db()?;

    let app = MainWindow::new()?;

    slint::select_bundled_translation("en")?;

    app_controller::install(&app, conn.clone());

    app.run()?;

    Ok(())
}
