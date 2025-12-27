use std::error::Error;

use rusqlite::Connection;

use super::{DatabaseRecord, Group, Log, Person};

pub fn insert_to_db(conn: &Connection, record: DatabaseRecord) -> Result<(), Box<dyn Error>> {
    match record {
        DatabaseRecord::Person(p) => insert_person(conn, &p),
        DatabaseRecord::Group(g) => insert_group(conn, &g),
        DatabaseRecord::GroupMembers(gid, pid) => insert_group_member(conn, gid, pid),
        DatabaseRecord::Log(l) => insert_log(conn, &l),
    }
}

fn insert_person(conn: &Connection, person: &Person) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "INSERT INTO `Person`(`name`, `surname`, `rank_level`, `methodology`) VALUES(?1, ?2, ?3, ?4);",
        (
            &person.name,
            &person.surname,
            person.rank_level as i32,
            person.methodology as i32,
        ),
    )?;

    let person_id: i32 = conn.last_insert_rowid() as i32;
    let group_id: i32 = person.methodology as i32 + 2; // +2 because db starts from 1 and 1 is reserved for everyone

    conn.execute(
        "INSERT INTO `GroupMembers`(`group_id`, `person_id`) VALUES (?1, ?2);",
        (group_id, person_id),
    )?;
    conn.execute(
        "INSERT INTO `GroupMembers`(`group_id`, `person_id`) VALUES (1, ?1);",
        (person_id,),
    )?;

    Ok(())
}

fn insert_group(conn: &Connection, group: &Group) -> Result<(), Box<dyn Error>> {
    conn.execute("INSERT INTO `Group`(`name`) VALUES(?1);", (&group.name,))?;
    Ok(())
}

fn insert_group_member(conn: &Connection, group_id: i32, person_id: i32) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "INSERT INTO `GroupMembers`(`group_id`, `person_id`) VALUES(?1, ?2);",
        (group_id, person_id),
    )?;
    Ok(())
}

fn insert_log(conn: &Connection, log: &Log) -> Result<(), Box<dyn Error>> {
    let time_str = log.time.format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO `Log`(`entity_type`, `entity_id`, `is_inside`, `timestamp`) VALUES(?1, ?2, ?3, ?4)",
        (&log.entity_type, &log.entity_id, &log.is_inside, &time_str),
    )?;
    Ok(())
}
