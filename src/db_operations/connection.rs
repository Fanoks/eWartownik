use std::{
    cell::RefCell,
    rc::Rc,
};

use rusqlite::Connection;

use super::{path, schema};

pub fn get_db() -> rusqlite::Result<Rc<RefCell<Connection>>> {
    let conn = Connection::open(path::db_path())?;
    schema::ensure_schema(&conn)?;
    Ok(Rc::new(RefCell::new(conn)))
}
