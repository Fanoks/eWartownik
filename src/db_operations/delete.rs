use std::error::Error;

use rusqlite::Connection;

use super::{DatabaseRecord, Group, Log, Person};

#[allow(dead_code)]
pub fn delete_from_db(conn: &Connection, record: DatabaseRecord) -> Result<(), Box<dyn Error>> {
    match record {
        DatabaseRecord::Person(p) => delete_person(conn, &p),
        DatabaseRecord::Group(g) => delete_group(conn, &g),
        DatabaseRecord::GroupMembers(gid, pid) => delete_group_member(conn, gid, pid),
        DatabaseRecord::Log(l) => delete_log(conn, &l),
    }
}

fn delete_person(conn: &Connection, person: &Person) -> Result<(), Box<dyn Error>> {
    conn.execute("DELETE FROM `Person` WHERE `id` = ?1;", (&person.id,))?;
    Ok(())
}

fn delete_group(conn: &Connection, group: &Group) -> Result<(), Box<dyn Error>> {
    conn.execute("DELETE FROM `Group` WHERE `id` = ?1;", (&group.id,))?;
    Ok(())
}

fn delete_group_member(conn: &Connection, group_id: i32, person_id: i32) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "DELETE FROM `GroupMembers` WHERE `group_id` = ?1 AND `person_id` = ?2;",
        (group_id, person_id),
    )?;
    Ok(())
}

fn delete_log(conn: &Connection, log: &Log) -> Result<(), Box<dyn Error>> {
    conn.execute("DELETE FROM `Log` WHERE `id` = ?1;", (&log.id,))?;
    Ok(())
}
