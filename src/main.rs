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
            if let Ok(person) = db_operations::get_person(&conn_ref) {
                let model: Vec<_> = person
                    .into_iter()
                    .map(|p| PersonData {
                        id: p.id,
                        name: SharedString::from(p.name),
                        surname: SharedString::from(p.surname),
                        rank: SharedString::from(p.rank_level.as_str()),
                        methodology: p.methodology.as_color()
                    })
                    .collect();

                app.set_people(ModelRc::new(VecModel::from(model)));
            }
        }
    };

    refresh_personel();

    {
        let conn_rc = conn.clone();
        let refresh_clone = refresh_personel.clone();

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

                // Keep copies for logging after `person` is moved into the DatabaseRecord
                let person_name = person.name.clone();
                let person_surname = person.surname.clone();

                {
                    let mut conn_ref = conn_rc.borrow_mut();
                    if let Err(e) = db_operations::insert_to_db(&mut *conn_ref, db_operations::DatabaseRecord::Person(person)) {
                        eprintln!("Error during insertion person: {}", e);
                        return;
                    }
                }

                refresh_clone();
            }
        });
    }

        {
        let conn_rc = conn.clone();
        let refresh_clone = refresh_personel.clone();

        app.on_remove_person_request({
            move |id| {
                let person: db_operations::Person = db_operations::Person {
                    id: id,
                    name: "".to_string(),
                    surname: "".to_string(),
                    rank_level: db_operations::RankLevel::RankNone,
                    methodology: db_operations::Methodology::Cub
                };

                {
                    let mut conn_ref = conn_rc.borrow_mut();
                    if let Err(e) = db_operations::delete_from_db(&mut *conn_ref, db_operations::DatabaseRecord::Person(person)) {
                        eprintln!("Error during deleting person: {}", e);
                        return;
                    }
                }

                refresh_clone();
            }
        });
    }

    app.run()?;

    Ok(())
}
