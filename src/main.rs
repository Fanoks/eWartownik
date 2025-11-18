// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use slint::{ModelRc, VecModel, SharedString, Model};
use std::collections::HashSet;
use std::rc::Rc;
use std::cell::RefCell;
use rusqlite::Connection;

mod db_operations;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let conn: Rc<RefCell<Connection>> = db_operations::get_db()?;

    let app = MainWindow::new()?;

    slint::select_bundled_translation("en")?;

    // Data caches for filtering persons by selected group
    let selection_groups: Rc<RefCell<Vec<GroupData>>> = Rc::new(RefCell::new(Vec::new()));
    let all_persons_for_selection: Rc<RefCell<Vec<PersonData>>> = Rc::new(RefCell::new(Vec::new()));

    let refresh_groups = {
        let app_weak = app.as_weak();
        let conn_rc = conn.clone();
        let selection_groups_rc = selection_groups.clone();
        let all_persons_rc = all_persons_for_selection.clone();

        move || {
            let conn_ref = conn_rc.borrow();
            let app = app_weak.unwrap();
            if let Ok(mut groups) = db_operations::get_group_with_members(&conn_ref) {
                // We'll populate persons_list from the special group with id = 1 ("Camp") which contains all persons.
                // groups_list will contain all group names ordered by id.
                let mut persons_list: Vec<PersonData> = Vec::new();
                let mut groups_list: Vec<GroupData> = Vec::new();
                let mut group_names: Vec<SharedString> = Vec::new();

                // Order groups by id
                groups.sort_by_key(|g| g.id);
                let groups_model: Vec<_> = groups.into_iter().map(|mut g| {
                    // Sort each group's members by methodology, surname, name
                    g.members.sort_by(|a, b| {
                        use std::cmp::Ordering;
                        let meth_cmp = (a.methodology as i32).cmp(&(b.methodology as i32));
                        if meth_cmp != Ordering::Equal { return meth_cmp; }
                        let sur_cmp = a.surname.to_lowercase().cmp(&b.surname.to_lowercase());
                        if sur_cmp != Ordering::Equal { return sur_cmp; }
                        a.name.to_lowercase().cmp(&b.name.to_lowercase())
                    });
                    let members_vec: Vec<_> = g.members.into_iter().map(|p| PersonData {
                        id: p.id,
                        name: SharedString::from(p.name),
                        surname: SharedString::from(p.surname),
                        rank: SharedString::from(p.rank_level.as_str()),
                        methodology: p.methodology.as_color()
                    }).collect();

                    // Capture global persons list from group id 1 if not yet filled
                    if g.id == 1 && persons_list.is_empty() {
                        // Copy all members into persons_list preserving order and colors
                        for m in &members_vec {
                            persons_list.push(PersonData {
                                id: m.id,
                                name: m.name.clone(),
                                surname: m.surname.clone(),
                                rank: m.rank.clone(),
                                methodology: m.methodology,
                            });
                        }
                    }

                    // Add user-manageable group (id > 5) to selection list WITH members for filtering
                    if g.id > 5 {
                        let members_clone: Vec<PersonData> = members_vec.iter().map(|m| PersonData {
                            id: m.id,
                            name: m.name.clone(),
                            surname: m.surname.clone(),
                            rank: m.rank.clone(),
                            methodology: m.methodology
                        }).collect();
                        groups_list.push(GroupData {
                            id: g.id,
                            name: SharedString::from(g.name.clone()),
                            members: ModelRc::new(VecModel::from(members_clone))
                        });
                        group_names.push(SharedString::from(g.name.clone()));
                    }
                    GroupData { id: g.id, name: SharedString::from(g.name), members: ModelRc::new(VecModel::from(members_vec)) }
                }).collect();
                // Cache selection data
                *selection_groups_rc.borrow_mut() = groups_list.clone();
                *all_persons_rc.borrow_mut() = persons_list.clone();

                // Pre-filter persons when form is first displayed: exclude members of the first selectable group (index 0) if any.
                let initial_filtered: Vec<PersonData> = if let Some(first_group) = groups_list.get(0) {
                    // Collect member ids of first selectable group
                    let mut member_ids: HashSet<i32> = HashSet::new();
                    for i in 0..first_group.members.row_count() {
                        if let Some(pd) = first_group.members.row_data(i) {
                            member_ids.insert(pd.id);
                        }
                    }
                    persons_list
                        .iter()
                        .filter(|p| !member_ids.contains(&p.id))
                        .cloned()
                        .collect()
                } else {
                    persons_list.clone()
                };
                app.set_filtered_persons_to_group(ModelRc::new(VecModel::from(initial_filtered)));
                app.set_groups(ModelRc::new(VecModel::from(groups_model)));
                app.set_persons_to_group(ModelRc::new(VecModel::from(persons_list)));
                app.set_groups_to_group(ModelRc::new(VecModel::from(groups_list)));
                app.set_groups_to_group_names(ModelRc::new(VecModel::from(group_names)));
            }
        }
    };

    refresh_groups();

    // Handle group selection changes for filtering persons (no timer; custom dropdown triggers callback)
    {
        let app_weak = app.as_weak();
        let selection_groups_rc = selection_groups.clone();
        let all_persons_rc = all_persons_for_selection.clone();
        app.on_group_selection_changed(move |group_index| {
            let app = app_weak.unwrap();
            let groups_vec = selection_groups_rc.borrow();
            let persons_vec = all_persons_rc.borrow();
            if group_index >= 0 && (group_index as usize) < groups_vec.len() {
                let selected_group = &groups_vec[group_index as usize];
                let mut member_ids: HashSet<i32> = HashSet::new();
                for i in 0..selected_group.members.row_count() {
                    if let Some(pd) = selected_group.members.row_data(i) {
                        member_ids.insert(pd.id);
                    }
                }
                let filtered: Vec<PersonData> = persons_vec
                    .iter()
                    .filter(|p| !member_ids.contains(&p.id))
                    .cloned()
                    .collect();
                app.set_filtered_persons_to_group(ModelRc::new(VecModel::from(filtered)));
            } else {
                app.set_filtered_persons_to_group(ModelRc::new(VecModel::from(persons_vec.clone())));
            }
        });
    }

    {
    let conn_rc = conn.clone();
    let refresh_groups_clone = refresh_groups.clone();

        app.on_add_person_request({
            move |name, surname, rank, methodology| {
                // Convert integer rank/methodology from UI into enums safely
                let rank_enum = match db_operations::RankLevel::try_from(rank as i32) {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("Invalid rank value: {}", rank);
                        return;
                    }
                };
                let methodology_enum = match db_operations::Methodology::try_from(methodology as i32) {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("Invalid methodology value: {}", methodology);
                        return;
                    }
                };

                let person: db_operations::Person = db_operations::Person {
                    id: 0,
                    name: name.to_string(),
                    surname: surname.to_string(),
                    rank_level: rank_enum,
                    methodology: methodology_enum
                };

                {
                    let mut conn_ref = conn_rc.borrow_mut();
                    if let Err(e) = db_operations::insert_to_db(&mut *conn_ref, db_operations::DatabaseRecord::Person(person)) {
                        eprintln!("Error during insertion person: {}", e);
                        return;
                    }
                }

                refresh_groups_clone();
            }
        });
    }

    {
        let conn_rc = conn.clone();
        let refresh_groups_clone = refresh_groups.clone();

        app.on_add_group_request({
            move |name| {
                let group = db_operations::Group { id: 0, name: name.to_string() };

                {
                    let mut conn_ref = conn_rc.borrow_mut();
                    if let Err(e) = db_operations::insert_to_db(&mut *conn_ref, db_operations::DatabaseRecord::Group(group)) {
                        eprintln!("Error during insertion group: {}", e);
                        return;
                    }
                }

                refresh_groups_clone();
            }
        });
    }

    {
        let conn_rc = conn.clone();
        let refresh_groups_clone = refresh_groups.clone();

        app.on_add_person_to_group_request({
            move |person_id, group_id| {
                {
                    let mut conn_ref = conn_rc.borrow_mut();
                    // Duplicate membership guard
                    let exists: rusqlite::Result<Option<i32>> = conn_ref.query_row(
                        "SELECT 1 FROM `GroupMembers` WHERE `group_id` = ?1 AND `person_id` = ?2 LIMIT 1;",
                        (group_id, person_id),
                        |row| row.get(0)
                    ).map(Some).or_else(|e| {
                        if let rusqlite::Error::QueryReturnedNoRows = e { Ok(None) } else { Err(e) }
                    });

                    match exists {
                        Ok(Some(_)) => {
                            eprintln!("Relation already exists: person {} in group {}", person_id, group_id);
                            return; // Skip duplicate insert
                        }
                        Ok(None) => {
                            if let Err(e) = db_operations::insert_to_db(&mut *conn_ref, db_operations::DatabaseRecord::GroupMembers(group_id, person_id)) {
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

                refresh_groups_clone();
            }
        });
    }

    app.run()?;

    Ok(())
}
