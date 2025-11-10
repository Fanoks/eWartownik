// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use slint::{ModelRc, VecModel, SharedString};
use std::rc::Rc;
use std::cell::RefCell;
use rusqlite::Connection;

mod db_operations;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let conn: Rc<RefCell<Connection>> = db_operations::get_db()?;

    let app = MainWindow::new()?;

    slint::select_bundled_translation("en")?;

    let refresh_personel = {
        let app_weak = app.as_weak();
        let conn_rc = conn.clone();

        move || {
            let conn_ref = conn_rc.borrow();
            let app = app_weak.unwrap();
            if let Ok(mut persons) = db_operations::get_person(&conn_ref) {
                // Sort in Rust: methodology, surname (case-insensitive), name (case-insensitive)
                persons.sort_by(|a, b| {
                    use std::cmp::Ordering;
                    let meth_cmp = (a.methodology as i32).cmp(&(b.methodology as i32));
                    if meth_cmp != Ordering::Equal { return meth_cmp; }
                    let sur_cmp = a.surname.to_lowercase().cmp(&b.surname.to_lowercase());
                    if sur_cmp != Ordering::Equal { return sur_cmp; }
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                });

                let model: Vec<_> = persons.into_iter().map(|p| PersonData {
                    id: p.id,
                    name: SharedString::from(p.name),
                    surname: SharedString::from(p.surname),
                    rank: SharedString::from(p.rank_level.as_str()),
                    methodology: p.methodology.as_color(),
                }).collect();
                app.set_people(ModelRc::new(VecModel::from(model)));
            }
        }
    };

    let refresh_groups = {
        let app_weak = app.as_weak();
        let conn_rc = conn.clone();

        move || {
            let conn_ref = conn_rc.borrow();
            let app = app_weak.unwrap();
            if let Ok(mut groups) = db_operations::get_group_with_members(&conn_ref) {
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
                        methodology: p.methodology.as_color(),
                    }).collect();
                    GroupData { id: g.id, name: SharedString::from(g.name), members: ModelRc::new(VecModel::from(members_vec)) }
                }).collect();
                app.set_groups(ModelRc::new(VecModel::from(groups_model)));
            }
        }
    };

    refresh_personel();
    refresh_groups();

    {
    let conn_rc = conn.clone();
    let refresh_clone = refresh_personel.clone();
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

                refresh_clone();
                // Members assignments may change groups content
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

    app.run()?;

    Ok(())
}
