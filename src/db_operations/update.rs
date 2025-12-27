use std::error::Error;

use rusqlite::Connection;

use super::{DatabaseRecord, Group, Log, Person};

#[allow(dead_code)]
pub fn update_db(conn: &Connection, record: DatabaseRecord) -> Result<(), Box<dyn Error>> {
    match record {
        DatabaseRecord::Person(p) => update_person(conn, &p),
        DatabaseRecord::Group(g) => update_group(conn, &g),
        DatabaseRecord::GroupMembers(_gid, _pid) => Ok(()),
        DatabaseRecord::Log(l) => update_log(conn, &l),
    }
}

fn update_person(conn: &Connection, person: &Person) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "UPDATE `Person` SET `name` = ?2, `surname` = ?3, `rank_level` = ?4, `methodology` = ?5 WHERE `id` = ?1;",
        (
            &person.id,
            &person.name,
            &person.surname,
            &(person.rank_level as i32),
            &(person.methodology as i32),
        ),
    )?;
    Ok(())
}

fn update_group(conn: &Connection, group: &Group) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "UPDATE `Group` SET `name` = ?2 WHERE `id` = ?1;",
        (&group.id, &group.name),
    )?;
    Ok(())
}

fn update_log(conn: &Connection, log: &Log) -> Result<(), Box<dyn Error>> {
    let time_str = log.time.format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "UPDATE `Log` SET `entity_type` = ?2, `entity_id` = ?3, `is_inside` = ?4, `timestamp` = ?5 WHERE `id` = ?1",
        (&log.id, &log.entity_type, &log.entity_id, &log.is_inside, &time_str),
    )?;
    Ok(())
}
