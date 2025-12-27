use std::{
    cell::RefCell,
    rc::Rc,
};

use rusqlite::Connection;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{MainWindow, PersonData};

use crate::db_operations;

use super::filter::filter_persons_excluding_group;

pub(super) fn wire_group_selection_changed(
    app: &MainWindow,
    selection_groups: Rc<RefCell<Vec<crate::GroupData>>>,
    all_persons_for_selection: Rc<RefCell<Vec<PersonData>>>,
) {
    let app_weak = app.as_weak();
    app.on_group_selection_changed(move |group_index| {
        let Some(app) = app_weak.upgrade() else {
            return;
        };

        let groups_vec = selection_groups.borrow();
        let persons_vec = all_persons_for_selection.borrow();

        if group_index >= 0 && (group_index as usize) < groups_vec.len() {
            let selected_group = &groups_vec[group_index as usize];
            let filtered = filter_persons_excluding_group(&persons_vec, selected_group);
            app.set_filtered_persons_to_group(ModelRc::new(VecModel::from(filtered)));
        } else {
            app.set_filtered_persons_to_group(ModelRc::new(VecModel::from(persons_vec.clone())));
        }
    });
}

pub(super) fn wire_add_person_request(
    app: &MainWindow,
    conn: Rc<RefCell<Connection>>,
    refresh_groups: impl Fn() + Clone + 'static,
) {
    app.on_add_person_request(move |name, surname, rank, methodology| {
        let Some(rank_enum) = parse_rank(rank) else {
            eprintln!("Invalid rank value: {}", rank);
            return;
        };
        let Some(methodology_enum) = parse_methodology(methodology) else {
            eprintln!("Invalid methodology value: {}", methodology);
            return;
        };

        let person = db_operations::Person {
            id: 0,
            name: name.to_string(),
            surname: surname.to_string(),
            rank_level: rank_enum,
            methodology: methodology_enum,
        };

        {
            let conn_ref = conn.borrow();
            if let Err(e) = db_operations::insert_to_db(&*conn_ref, db_operations::DatabaseRecord::Person(person)) {
                eprintln!("Error during insertion person: {}", e);
                return;
            }
        }

        refresh_groups();
    });
}

pub(super) fn wire_add_group_request(
    app: &MainWindow,
    conn: Rc<RefCell<Connection>>,
    refresh_groups: impl Fn() + Clone + 'static,
) {
    app.on_add_group_request(move |name| {
        let group = db_operations::Group {
            id: 0,
            name: name.to_string(),
        };

        {
            let conn_ref = conn.borrow();
            if let Err(e) = db_operations::insert_to_db(&*conn_ref, db_operations::DatabaseRecord::Group(group)) {
                eprintln!("Error during insertion group: {}", e);
                return;
            }
        }

        refresh_groups();
    });
}

pub(super) fn wire_add_person_to_group_request(
    app: &MainWindow,
    conn: Rc<RefCell<Connection>>,
    refresh_groups: impl Fn() + Clone + 'static,
) {
    app.on_add_person_to_group_request(move |person_id, group_id| {
        {
            let conn_ref = conn.borrow();

            match relation_exists(&conn_ref, group_id, person_id) {
                Ok(true) => {
                    eprintln!(
                        "Relation already exists: person {} in group {}",
                        person_id, group_id
                    );
                    return;
                }
                Ok(false) => {
                    if let Err(e) = db_operations::insert_to_db(
                        &*conn_ref,
                        db_operations::DatabaseRecord::GroupMembers(group_id, person_id),
                    ) {
                        eprintln!("Error during insertion of relation: {}", e);
                        return;
                    }
                }
                Err(e) => {
                    eprintln!("Error checking existing relation: {}", e);
                    return;
                }
            }
        }

        refresh_groups();
    });
}

fn relation_exists(conn: &Connection, group_id: i32, person_id: i32) -> rusqlite::Result<bool> {
    let exists: rusqlite::Result<Option<i32>> = conn
        .query_row(
            "SELECT 1 FROM `GroupMembers` WHERE `group_id` = ?1 AND `person_id` = ?2 LIMIT 1;",
            (group_id, person_id),
            |row| row.get(0),
        )
        .map(Some)
        .or_else(|e| {
            if let rusqlite::Error::QueryReturnedNoRows = e {
                Ok(None)
            } else {
                Err(e)
            }
        });

    Ok(exists?.is_some())
}

fn parse_rank(rank: i32) -> Option<db_operations::RankLevel> {
    db_operations::RankLevel::try_from(rank).ok()
}

fn parse_methodology(methodology: i32) -> Option<db_operations::Methodology> {
    db_operations::Methodology::try_from(methodology).ok()
}
