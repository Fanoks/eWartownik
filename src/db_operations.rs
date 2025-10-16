use std::error::Error;
use std::path::PathBuf;
use std::fs::create_dir_all;
use cfg_if::cfg_if;
use serde::{Serialize, Deserialize};
use rusqlite::{ToSql, Connection, Result, types::{ToSqlOutput, ValueRef, FromSql, FromSqlResult, FromSqlError}};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

macro_rules! impl_sql_enum_for {
    ($enum_type:ident{
        $($variant:ident = $value:expr),* $(,)?
    }) => {
        impl ToSql for $enum_type {
            fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
                Ok((*self as i32).into())
            }
        }

        impl FromSql for $enum_type {
            fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                match value.as_i64()? {
                    $($value => Ok($enum_type::$variant),)*
                    other => Err(FromSqlError::Other(
                        format!("Invalid value {} for enum {}", other, stringify!($enum_type)).into()
                    ))
                }
            }
        }
    };
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RankLevel {
    RankNone = 0,
    RankFirst = 1,
    RankSecond = 2,
    RankThird = 3,
    RankFourth = 4,
    RankFifth = 5,
    RankSixth = 6
}

impl_sql_enum_for!(RankLevel {
    RankNone = 0,
    RankFirst = 1,
    RankSecond = 2,
    RankThird = 3,
    RankFourth = 4,
    RankFifth = 5,
    RankSixth = 6
});

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Methodology {
    Cub = 0,
    Scout = 1,
    VentureScout = 2,
    Rover = 3
}

impl_sql_enum_for!(Methodology {
    Cub = 0,
    Scout = 1,
    VentureScout = 2,
    Rover = 3
});

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum EntityType {
    Person = 0,
    Group = 1
}

impl_sql_enum_for!(EntityType {
    Person = 0,
    Group = 1
});

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum IsInside {
    Out = 0,
    In = 1
}

impl_sql_enum_for!(IsInside {
    Out = 0,
    In = 1
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub surname: String,
    pub rank_level: RankLevel,
    pub methodology: Methodology
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: i32,
    pub name: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    id: i32,
    entity_type: EntityType,
    entity_id: i32,
    is_inside: IsInside,
    time: DateTime<Utc>
}

pub enum DatabaseRecord {
    Person(Person),
    Group(Group),
    GroupMembers(i32, i32),
    Log(Log)
}

fn db_path() -> PathBuf {
    cfg_if! {
        if #[cfg(target_os = "windows")] {
            //let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            let dir = PathBuf::from("C:/Users/fanok/Desktop/programowanie/Rust/eWartownik/db"); //base.join("eWartownik");
        } else if #[cfg(target_os = "linux")] {
            let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            let dir = base.join("eWartownik");
        } else if #[cfg(target_os = "macos")] {
            let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            let dir = base.join("eWartownik");
        } else if #[cfg(target_os = "android")] {
            let dir = PathBuf::from("/data/data/com.ewartownik.app/files/eWartownik");
        } else if #[cfg(target_os = "ios")] {
            // I'm not sure if it will work
            let dir = PathBuf::from("/var/mobile/Containers/Data/Application/Documents/eWartownik");
        } else {
            let dir = PathBuf::from("eWartownik");
        }
    }

    if let Err(e) = create_dir_all(&dir) {
        eprintln!("Couldn't create direction: {e}");
    }

    dir.join("database.db")
}

pub fn get_db() -> Result<Connection> {
    let conn: Connection = Connection::open(db_path())?;

    conn.execute("PRAGMA foreign_keys = ON;", ())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS `Person`(
            `id` INTEGER PRIMARY KEY AUTOINCREMENT,
            `name` TEXT NOT NULL,
            `surname` TEXT NOT NULL,
            `rank_level` INTEGER NOT NULL,
            `methodology` INTEGER NOT NULL
        );",
        ()
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS `Group`(
            `id` INTEGER PRIMARY KEY AUTOINCREMENT,
            `name` TEXT NOT NULL
        );",
        ()
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS `GroupMembers`(
            `group_id` INTEGER NOT NULL REFERENCES `Group`(`id`) ON DELETE CASCADE,
            `person_id` INTEGER NOT NULL REFERENCES `Person`(`id`) ON DELETE CASCADE,
            PRIMARY KEY (group_id, person_id)
        )",
        ()
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS `Log`(
            `id` INTEGER PRIMARY KEY AUTOINCREMENT,
            `entity_type` INTEGER NOT NULL CHECK(`entity_type` IN (0, 1)),
            `entity_id` INTEGER NOT NULL,
            `is_inside` BOOLEAN NOT NULL DEFAULT 0,
            `timestamp` TEXT DEFAULT (datetime('now'))
        );",
        ()
    )?;

    Ok(conn)
}

pub fn insert_to_db(conn: &Connection, record: DatabaseRecord) -> Result<(), Box<dyn Error>> {
    match record {
        DatabaseRecord::Person(p) => insert_person(conn, &p),
        DatabaseRecord::Group(g) => insert_group(conn, &g),
        DatabaseRecord::GroupMembers(gid, pid) => insert_group_member(conn, gid, pid),
        DatabaseRecord::Log(l) => insert_log(conn, &l)
    }
}

fn insert_person(conn: &Connection, person: &Person) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "INSERT INTO `Person`(`name`, `surname`, `rank_level`, `methodology`) VALUES(?1, ?2, ?3, ?4);",
        (&person.name, &person.surname, person.rank_level as i32, person.methodology as i32)
    )?;
    Ok(())
}

fn insert_group(conn: &Connection, group: &Group) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "INSERT INTO `Group`(`name`) VALUES(?1);",
        (&group.name,)
    )?;
    Ok(())
}

fn insert_group_member(conn: &Connection, group_id: i32, person_id: i32) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "INSERT INTO `GroupMembers`(`group_id`, `person_id`) VALUES(?1, ?2);",
        (group_id, person_id)
    )?;
    Ok(())
}

fn insert_log(conn: &Connection, log: &Log) -> Result<(), Box<dyn Error>> {
    let time_str = log.time.format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO `Log`(`entity_type`, `entity_id`, `is_inside`, `timestamp`) VALUES(?1, ?2, ?3, ?4)",
        (&log.entity_type, &log.entity_id, &log.is_inside, &time_str)
    )?;
    Ok(())
}

pub fn update_db(conn: &Connection, record: DatabaseRecord) -> Result<(), Box<dyn Error>> {
    match record {
        DatabaseRecord::Person(p) => update_person(conn, &p),
        DatabaseRecord::Group(g) => update_group(conn, &g),
        DatabaseRecord::GroupMembers(_gid, _pid) => Ok(()),
        DatabaseRecord::Log(l) => update_log(conn, &l)
    }
}

fn update_person(conn: &Connection, person: &Person) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "UPDATE `Person` SET `name` = ?2, `surname` = ?3, `rank_level` = ?4, `methodology` = ?5 WHERE `id` = ?1;",
        (&person.id, &person.name, &person.surname, &(person.rank_level as i32), &(person.methodology as i32))
    )?;
    Ok(())
}

fn update_group(conn: &Connection, group: &Group) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "UPDATE `Group` SET `name` = ?2 WHERE `id` = ?1;",
        (&group.id, &group.name)
    )?;
    Ok(())
}

fn update_log(conn: &Connection, log: &Log) -> Result<(), Box<dyn Error>> {
    let time_str = log.time.format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "UPDATE `Log` SET `entity_type` = ?2, `entity_id` = ?3, `is_inside` = ?4, `timestamp` = ?5 WHERE `id` = ?1",
        (&log.id, &log.entity_type, &log.entity_id, &log.is_inside, &time_str)
    )?;
    Ok(())
}

pub fn delete_from_db(conn: &Connection, record: DatabaseRecord) -> Result<(), Box<dyn Error>> {
    match record {
        DatabaseRecord::Person(p) => delete_person(conn, &p),
        DatabaseRecord::Group(g) => delete_group(conn, &g),
        DatabaseRecord::GroupMembers(gid, pid) => delete_group_member(conn, gid, pid),
        DatabaseRecord::Log(l) => delete_log(conn, &l)
    }
}

fn delete_person(conn: &Connection, person: &Person) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "DELETE FROM `Person` WHERE `id` = ?1;",
        (&person.id,)
    )?;
    Ok(())
}

fn delete_group(conn: &Connection, group: &Group) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "DELETE FROM `Group` WHERE `id` = ?1;",
        (&group.id,)
    )?;
    Ok(())
}

fn delete_group_member(conn: &Connection, group_id: i32, person_id: i32) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "DELETE FROM `GroupMembers` WHERE `group_id` = ?1 AND `person_id` = ?2;",
        (group_id, person_id)
    )?;
    Ok(())
}

fn delete_log(conn: &Connection, log: &Log) -> Result<(), Box<dyn Error>> {
    conn.execute(
        "DELETE FROM `Log` WHERE `id` = ?1;",
        (&log.id,)
    )?;
    Ok(())
}

pub fn get_person(conn: &Connection) -> Result<Vec<Person>, Box<dyn Error>> {
    let mut stmt: rusqlite::Statement<'_> = conn.prepare("SELECT `id`, `name`, `surname`, `rank_level`, `methodology` FROM `Person`;")?;
    let person_iter = stmt.query_map([], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            surname: row.get(2)?,
            rank_level: row.get(3)?,
            methodology: row.get(4)?
        })
    })?;

    let persons: Result<Vec<Person>, _> = person_iter.collect();

    Ok(persons?)
}

pub fn get_group(conn: &Connection) -> Result<Vec<Group>, Box<dyn Error>> {
    let mut stmt: rusqlite::Statement<'_> = conn.prepare("SELECT `id`, `name` FROM `Group`;")?;
    let group_iter = stmt.query_map([], |row| {
        Ok(Group {
            id: row.get(0)?,
            name: row.get(1)?
        })
    })?;

    let groups: Result<Vec<Group>, _> = group_iter.collect();

    Ok(groups?)
}

pub fn get_group_member(conn: &Connection) -> Result<Vec<(i32, i32)>, Box<dyn Error>> {
    let mut stmt: rusqlite::Statement<'_> = conn.prepare("SELECT `group_id`, `person_id` FROM `GroupMembers`;")?;
    let group_members_iter = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?;

    let group_members: Result<Vec<(i32, i32)>, _> = group_members_iter.collect();

    Ok(group_members?)
}

pub fn get_log(conn: &Connection) -> Result<Vec<Log>, Box<dyn Error>> {
    let mut stmt: rusqlite::Statement<'_> = conn.prepare("SELECT `id`, `entity_type`, `entity_id`, `is_inside`, `timestamp` FROM `Log`;")?;
    let log_iter = stmt.query_map([], |row| {
        let time_str: String = row.get(4)?;
        let naive = NaiveDateTime::parse_from_str(&time_str, "%Y-%m-%d %H:%M:%S")
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?;
        let datetime_utc = Utc.from_utc_datetime(&naive);

        Ok(Log {
            id: row.get(0)?,
            entity_type: row.get(1)?,
            entity_id: row.get(2)?,
            is_inside: row.get(3)?,
            time: datetime_utc
        })
    })?;

    let logs: Result<Vec<Log>, _> = log_iter.collect();

    Ok(logs?)
}
