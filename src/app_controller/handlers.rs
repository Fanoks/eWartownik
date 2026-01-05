use std::{
    cell::RefCell,
    rc::Rc,
};

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{MainWindow, PersonData};

use crate::db_operations;

use super::filter::filter_persons_excluding_group;

#[cfg(debug_assertions)]
macro_rules! main_debug {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    };
}

#[cfg(not(debug_assertions))]
macro_rules! main_debug {
    ($($arg:tt)*) => {};
}

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
            is_inside: db_operations::IsInside::Out,
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

pub(super) fn wire_main_person_toggled(
    app: &MainWindow,
    all_persons_for_main: Rc<RefCell<Vec<PersonData>>>,
    checked_person_ids: Rc<RefCell<HashSet<i32>>>,
    out_person_ids: Rc<RefCell<HashSet<i32>>>,
) {
    let app_weak = app.as_weak();
    app.on_main_person_toggled(move |person_id| {
        let Some(app) = app_weak.upgrade() else {
            return;
        };

        let now_checked = {
            let mut set = checked_person_ids.borrow_mut();
            if set.contains(&person_id) {
                set.remove(&person_id);
                false
            } else {
                set.insert(person_id);
                true
            }
        };

        main_debug!("[main] person toggled: id={} -> checked={}", person_id, now_checked);
        set_main_people_models(
            &app,
            &all_persons_for_main.borrow(),
            &out_person_ids.borrow(),
            &checked_person_ids.borrow(),
        );
    });
}

pub(super) fn wire_main_group_clicked(
    app: &MainWindow,
    all_persons_for_main: Rc<RefCell<Vec<PersonData>>>,
    checked_person_ids: Rc<RefCell<HashSet<i32>>>,
    out_person_ids: Rc<RefCell<HashSet<i32>>>,
    group_members_by_id: Rc<RefCell<HashMap<i32, Vec<i32>>>>,
) {
    let app_weak = app.as_weak();
    app.on_main_group_clicked(move |group_id| {
        let Some(app) = app_weak.upgrade() else {
            return;
        };

        let member_ids = {
            let map = group_members_by_id.borrow();
            map.get(&group_id).cloned().unwrap_or_default()
        };

        main_debug!(
            "[main] group clicked: id={} members={}",
            group_id,
            member_ids.len()
        );

        {
            let mut set = checked_person_ids.borrow_mut();
            set.clear();
            for id in member_ids {
                set.insert(id);
            }
            main_debug!("[main] checked_person_ids size={}", set.len());
        }

        set_main_people_models(
            &app,
            &all_persons_for_main.borrow(),
            &out_person_ids.borrow(),
            &checked_person_ids.borrow(),
        );
    });
}

pub(super) fn wire_main_get_in(
    app: &MainWindow,
    conn: Rc<RefCell<Connection>>,
    all_persons_for_main: Rc<RefCell<Vec<PersonData>>>,
    checked_person_ids: Rc<RefCell<HashSet<i32>>>,
    out_person_ids: Rc<RefCell<HashSet<i32>>>,
    refresh_groups: impl Fn() + Clone + 'static,
) {
    let app_weak = app.as_weak();
    app.on_main_get_in(move || {
        let Some(app) = app_weak.upgrade() else {
            return;
        };

        let selected: Vec<i32> = checked_person_ids.borrow().iter().copied().collect();

        // Persist DB state
        {
            let conn_ref = conn.borrow();
            for id in &selected {
                if let Err(e) = db_operations::set_person_is_inside(&*conn_ref, *id, db_operations::IsInside::In) {
                    eprintln!("Error updating person is_inside (GET_IN) for id {}: {}", id, e);
                }
            }
        }

        {
            let mut out = out_person_ids.borrow_mut();
            for id in &selected {
                out.remove(id);
            }
        }
        checked_person_ids.borrow_mut().clear();

        main_debug!("[main] GET_IN moved {} ids", selected.len());
        refresh_groups();
        set_main_people_models(&app, &all_persons_for_main.borrow(), &out_person_ids.borrow(), &checked_person_ids.borrow());
    });
}

pub(super) fn wire_main_get_out(
    app: &MainWindow,
    conn: Rc<RefCell<Connection>>,
    all_persons_for_main: Rc<RefCell<Vec<PersonData>>>,
    checked_person_ids: Rc<RefCell<HashSet<i32>>>,
    out_person_ids: Rc<RefCell<HashSet<i32>>>,
    refresh_groups: impl Fn() + Clone + 'static,
) {
    let app_weak = app.as_weak();
    app.on_main_get_out(move || {
        let Some(app) = app_weak.upgrade() else {
            return;
        };

        let selected: Vec<i32> = checked_person_ids.borrow().iter().copied().collect();

        // Persist DB state
        {
            let conn_ref = conn.borrow();
            for id in &selected {
                if let Err(e) = db_operations::set_person_is_inside(&*conn_ref, *id, db_operations::IsInside::Out) {
                    eprintln!("Error updating person is_inside (GET_OUT) for id {}: {}", id, e);
                }
            }
        }

        {
            let mut out = out_person_ids.borrow_mut();
            for id in &selected {
                out.insert(*id);
            }
        }
        checked_person_ids.borrow_mut().clear();

        main_debug!("[main] GET_OUT moved {} ids", selected.len());
        refresh_groups();
        set_main_people_models(&app, &all_persons_for_main.borrow(), &out_person_ids.borrow(), &checked_person_ids.borrow());
    });
}

fn set_main_people_models(
    app: &MainWindow,
    persons_all: &[PersonData],
    out_set: &HashSet<i32>,
    checked_set: &HashSet<i32>,
) {
    let mut people_in: Vec<PersonData> = Vec::new();
    let mut people_out: Vec<PersonData> = Vec::new();
    for p in persons_all {
        if out_set.contains(&p.id) {
            people_out.push(p.clone());
        } else {
            people_in.push(p.clone());
        }
    }

    let checked_in: Vec<bool> = people_in.iter().map(|p| checked_set.contains(&p.id)).collect();
    let checked_out: Vec<bool> = people_out.iter().map(|p| checked_set.contains(&p.id)).collect();

    app.set_people_checked(ModelRc::new(VecModel::from(checked_in)));
    app.set_people_out_checked(ModelRc::new(VecModel::from(checked_out)));

    // Recreate delegates to restore one-way bindings after user toggles.
    app.set_people(ModelRc::new(VecModel::from(people_in)));
    app.set_people_out(ModelRc::new(VecModel::from(people_out)));
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
