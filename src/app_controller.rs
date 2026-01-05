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
use std::collections::{HashMap, HashSet};

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

    // Main screen selection state
    let checked_person_ids: Rc<RefCell<HashSet<i32>>> = Rc::new(RefCell::new(HashSet::new()));
    let out_person_ids: Rc<RefCell<HashSet<i32>>> = Rc::new(RefCell::new(HashSet::new()));
    let group_members_by_id: Rc<RefCell<HashMap<i32, Vec<i32>>>> = Rc::new(RefCell::new(HashMap::new()));

    let refresh_groups = refresh::make_refresh_groups(
        app.as_weak(),
        conn.clone(),
        selection_groups.clone(),
        all_persons_for_selection.clone(),
        checked_person_ids.clone(),
        out_person_ids.clone(),
        group_members_by_id.clone(),
    );

    refresh_groups();

    handlers::wire_group_selection_changed(
        app,
        selection_groups.clone(),
        all_persons_for_selection.clone(),
    );

    handlers::wire_main_person_toggled(
        app,
        all_persons_for_selection.clone(),
        checked_person_ids.clone(),
        out_person_ids.clone(),
    );
    handlers::wire_main_group_clicked(
        app,
        all_persons_for_selection.clone(),
        checked_person_ids.clone(),
        out_person_ids.clone(),
        group_members_by_id,
    );
    handlers::wire_main_get_in(
        app,
        conn.clone(),
        all_persons_for_selection.clone(),
        checked_person_ids.clone(),
        out_person_ids.clone(),
        refresh_groups.clone(),
    );
    handlers::wire_main_get_out(
        app,
        conn.clone(),
        all_persons_for_selection.clone(),
        checked_person_ids.clone(),
        out_person_ids,
        refresh_groups.clone(),
    );

    handlers::wire_add_person_request(app, conn.clone(), refresh_groups.clone());
    handlers::wire_add_group_request(app, conn.clone(), refresh_groups.clone());
    handlers::wire_add_person_to_group_request(app, conn, refresh_groups);
}
