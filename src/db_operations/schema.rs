use rusqlite::Connection;

pub(super) fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("PRAGMA foreign_keys = ON;", ())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS `Person`(
            `id` INTEGER PRIMARY KEY AUTOINCREMENT,
            `name` TEXT NOT NULL,
            `surname` TEXT NOT NULL,
            `rank_level` INTEGER NOT NULL,
            `methodology` INTEGER NOT NULL
        );",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS `Group`(
            `id` INTEGER PRIMARY KEY AUTOINCREMENT,
            `name` TEXT NOT NULL
        );",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS `GroupMembers`(
            `group_id` INTEGER NOT NULL REFERENCES `Group`(`id`) ON DELETE CASCADE,
            `person_id` INTEGER NOT NULL REFERENCES `Person`(`id`) ON DELETE CASCADE,
            PRIMARY KEY (group_id, person_id)
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS `Log`(
            `id` INTEGER PRIMARY KEY AUTOINCREMENT,
            `entity_type` INTEGER NOT NULL CHECK(`entity_type` IN (0, 1)),
            `entity_id` INTEGER NOT NULL,
            `is_inside` BOOLEAN NOT NULL DEFAULT 0,
            `timestamp` TEXT DEFAULT (datetime('now'))
        );",
        (),
    )?;

    seed_default_groups(conn)?;

    Ok(())
}

fn seed_default_groups(conn: &Connection) -> rusqlite::Result<()> {
    let count: i32 = conn.query_row("SELECT COUNT(`id`) FROM `Group`;", [], |row| row.get(0))?;

    if count == 0 {
        conn.execute(
            "INSERT INTO `Group`(`id`, `name`) VALUES
                (1, 'Camp'),
                (2, 'Cub'),
                (3, 'Scout'),
                (4, 'Venture Scout'),
                (5, 'Rover');",
            (),
        )?;
    }

    Ok(())
}
