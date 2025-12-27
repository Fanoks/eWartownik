use std::error::Error;

use rusqlite::Connection;

use super::{parse_db_datetime, Group, GroupWithMembers, Person, RankLevel, Methodology};

#[allow(dead_code)]
pub fn get_person(conn: &Connection) -> Result<Vec<Person>, Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "SELECT `id`, `name`, `surname`, `rank_level`, `methodology` FROM `Person`;",
    )?;

    let person_iter = stmt.query_map([], |row| {
        Ok(Person {
            id: row.get(0)?,
            name: row.get(1)?,
            surname: row.get(2)?,
            rank_level: row.get(3)?,
            methodology: row.get(4)?,
        })
    })?;

    let persons: rusqlite::Result<Vec<Person>> = person_iter.collect();
    Ok(persons?)
}

#[allow(dead_code)]
pub fn get_group(conn: &Connection) -> Result<Vec<Group>, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT `id`, `name` FROM `Group`;")?;

    let group_iter = stmt.query_map([], |row| {
        Ok(Group {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;

    let groups: rusqlite::Result<Vec<Group>> = group_iter.collect();
    Ok(groups?)
}

#[allow(dead_code)]
pub fn get_group_member(conn: &Connection) -> Result<Vec<(i32, i32)>, Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT `group_id`, `person_id` FROM `GroupMembers`;")?;

    let group_members_iter = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let group_members: rusqlite::Result<Vec<(i32, i32)>> = group_members_iter.collect();

    Ok(group_members?)
}

pub fn get_group_with_members(conn: &Connection) -> Result<Vec<GroupWithMembers>, Box<dyn Error>> {
    // Implementation note:
    // We build a map of groups first, then populate members with a join query.
    // The returned vector order is not guaranteed (HashMap iteration), so callers that
    // care about ordering should sort (the UI does).
    let mut groups_stmt = conn.prepare("SELECT `id`, `name` FROM `Group`;")?;
    let groups_iter = groups_stmt.query_map([], |row| {
        Ok((row.get::<_, i32>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut groups_map: std::collections::HashMap<i32, GroupWithMembers> = std::collections::HashMap::new();

    for group in groups_iter {
        let (id, name) = group?;
        groups_map.insert(
            id,
            GroupWithMembers {
                id,
                name,
                members: Vec::new(),
            },
        );
    }

    let mut members_stmt = conn.prepare(
        "SELECT `gm`.`group_id`, `p`.`id`, `p`.`name`, `p`.`surname`, `p`.`rank_level`, `p`.`methodology`
         FROM `GroupMembers` `gm`
         JOIN `Person` `p` ON `gm`.`person_id` = `p`.`id`;",
    )?;

    let members_iter = members_stmt.query_map([], |row| {
        Ok((
            row.get::<_, i32>(0)?,
            Person {
                id: row.get(1)?,
                name: row.get(2)?,
                surname: row.get(3)?,
                rank_level: RankLevel::try_from(row.get::<_, i32>(4)?).unwrap_or(RankLevel::RankNone),
                methodology: Methodology::try_from(row.get::<_, i32>(5)?).unwrap_or(Methodology::Cub),
            },
        ))
    })?;

    for member in members_iter {
        let (group_id, person) = member?;
        if let Some(group) = groups_map.get_mut(&group_id) {
            group.members.push(person);
        }
    }

    Ok(groups_map.into_values().collect())
}

#[allow(dead_code)]
pub fn get_log(conn: &Connection) -> Result<Vec<super::Log>, Box<dyn Error>> {
    let mut stmt = conn.prepare(
        "SELECT `id`, `entity_type`, `entity_id`, `is_inside`, `timestamp` FROM `Log`;",
    )?;

    let log_iter = stmt.query_map([], |row| {
        let time_str: String = row.get(4)?;
        let datetime_utc = parse_db_datetime(&time_str)?;

        Ok(super::Log {
            id: row.get(0)?,
            entity_type: row.get(1)?,
            entity_id: row.get(2)?,
            is_inside: row.get(3)?,
            time: datetime_utc,
        })
    })?;

    let logs: rusqlite::Result<Vec<super::Log>> = log_iter.collect();

    Ok(logs?)
}
