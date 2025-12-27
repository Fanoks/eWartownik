//! UI-side application controller.
//!
//! This module is responsible for wiring Slint callbacks to the database layer and
//! for translating DB models into Slint models.
//!
//! Design notes:
//! - We keep small in-memory caches (`selection_groups`, `all_persons_for_selection`) to
//!   compute filtered lists quickly when the user changes the selected group.
//! - The heavy “refresh everything from DB” work is encapsulated in `refresh::make_refresh_groups()`.

use std::{cell::RefCell, rc::Rc};

use rusqlite::Connection;
use slint::ComponentHandle;

use crate::{GroupData, MainWindow, PersonData};

mod filter;
mod handlers;
mod refresh;

pub fn install(app: &MainWindow, conn: Rc<RefCell<Connection>>) {
    // Data caches for filtering persons by selected group
    let selection_groups: Rc<RefCell<Vec<GroupData>>> = Rc::new(RefCell::new(Vec::new()));
    let all_persons_for_selection: Rc<RefCell<Vec<PersonData>>> = Rc::new(RefCell::new(Vec::new()));

    let refresh_groups = refresh::make_refresh_groups(
        app.as_weak(),
        conn.clone(),
        selection_groups.clone(),
        all_persons_for_selection.clone(),
    );

    refresh_groups();

    handlers::wire_group_selection_changed(
        app,
        selection_groups.clone(),
        all_persons_for_selection.clone(),
    );

    handlers::wire_add_person_request(app, conn.clone(), refresh_groups.clone());
    handlers::wire_add_group_request(app, conn.clone(), refresh_groups.clone());
    handlers::wire_add_person_to_group_request(app, conn, refresh_groups);
}
