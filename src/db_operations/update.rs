use std::error::Error;

use rusqlite::Connection;

use super::{DatabaseRecord, Group, Log, Person};
use super::IsInside;

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
        "UPDATE `Person` SET `name` = ?2, `surname` = ?3, `rank_level` = ?4, `methodology` = ?5, `is_inside` = ?6 WHERE `id` = ?1;",
        (
            &person.id,
            &person.name,
            &person.surname,
            &(person.rank_level as i32),
            &(person.methodology as i32),
            &person.is_inside,
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

pub fn set_person_is_inside(conn: &Connection, person_id: i32, is_inside: IsInside) -> Result<(), Box<dyn Error>> {
    // Persist current state on the Person row
    conn.execute(
        "UPDATE `Person` SET `is_inside` = ?2 WHERE `id` = ?1;",
        (&person_id, &is_inside),
    )?;

    // Append an audit log row (timestamp defaults to now)
    conn.execute(
        "INSERT INTO `Log`(`entity_type`, `entity_id`, `is_inside`) VALUES(0, ?1, ?2);",
        (&person_id, &is_inside),
    )?;

    Ok(())
}
