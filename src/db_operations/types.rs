use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
    ToSql,
};
use serde::{Deserialize, Serialize};
use slint::Color;

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
                        format!("Invalid value {} for enum {}", other, stringify!($enum_type)).into(),
                    )),
                }
            }
        }
    };
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RankLevel {
    RankNone = 0,
    RankFirstM = 1,
    RankFirstF = 2,
    RankSecondM = 3,
    RankSecondF = 4,
    RankThirdM = 5,
    RankThirdF = 6,
    RankFourthM = 7,
    RankFourthF = 8,
    RankFifth = 9,
    RankSixth = 10,
}

impl_sql_enum_for!(RankLevel {
    RankNone = 0,
    RankFirstM = 1,
    RankFirstF = 2,
    RankSecondM = 3,
    RankSecondF = 4,
    RankThirdM = 5,
    RankThirdF = 6,
    RankFourthM = 7,
    RankFourthF = 8,
    RankFifth = 9,
    RankSixth = 10,
});

impl core::convert::TryFrom<i32> for RankLevel {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RankLevel::RankNone),
            1 => Ok(RankLevel::RankFirstM),
            2 => Ok(RankLevel::RankFirstF),
            3 => Ok(RankLevel::RankSecondM),
            4 => Ok(RankLevel::RankSecondF),
            5 => Ok(RankLevel::RankThirdM),
            6 => Ok(RankLevel::RankThirdF),
            7 => Ok(RankLevel::RankFourthM),
            8 => Ok(RankLevel::RankFourthF),
            9 => Ok(RankLevel::RankFifth),
            10 => Ok(RankLevel::RankSixth),
            _ => Err("invalid RankLevel"),
        }
    }
}

impl RankLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RankLevel::RankNone => "",
            RankLevel::RankFirstM => "RANK_FIRST_MALE",
            RankLevel::RankFirstF => "RANK_FIRST_FEMALE",
            RankLevel::RankSecondM => "RANK_SECOND_MALE",
            RankLevel::RankSecondF => "RANK_SECOND_FEMALE",
            RankLevel::RankThirdM => "RANK_THIRD_MALE",
            RankLevel::RankThirdF => "RANK_THIRD_FEMALE",
            RankLevel::RankFourthM => "RANK_FOURTH_MALE",
            RankLevel::RankFourthF => "RANK_FOURTH_FEMALE",
            RankLevel::RankFifth => "RANK_FIFTH",
            RankLevel::RankSixth => "RANK_SIXTH",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum Methodology {
    Cub = 0,
    Scout = 1,
    VentureScout = 2,
    Rover = 3,
}

impl_sql_enum_for!(Methodology {
    Cub = 0,
    Scout = 1,
    VentureScout = 2,
    Rover = 3,
});

impl Methodology {
    pub fn as_color(&self) -> Color {
        match self {
            Methodology::Cub => Color::from_rgb_u8(255, 189, 89),
            Methodology::Scout => Color::from_rgb_u8(175, 203, 7),
            Methodology::VentureScout => Color::from_rgb_u8(18, 64, 147),
            Methodology::Rover => Color::from_rgb_u8(227, 6, 19),
        }
    }
}

impl core::convert::TryFrom<i32> for Methodology {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Methodology::Cub),
            1 => Ok(Methodology::Scout),
            2 => Ok(Methodology::VentureScout),
            3 => Ok(Methodology::Rover),
            _ => Err("invalid Methodology"),
        }
    }
}

impl Ord for Methodology {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as i32).cmp(&(*other as i32))
    }
}

impl PartialOrd for Methodology {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(in crate::db_operations) enum EntityType {
    Person = 0,
    Group = 1,
}

impl_sql_enum_for!(EntityType {
    Person = 0,
    Group = 1,
});

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(in crate::db_operations) enum IsInside {
    Out = 0,
    In = 1,
}

impl_sql_enum_for!(IsInside {
    Out = 0,
    In = 1,
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub surname: String,
    pub rank_level: RankLevel,
    pub methodology: Methodology,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupWithMembers {
    pub id: i32,
    pub name: String,
    pub members: Vec<Person>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub(in crate::db_operations) id: i32,
    pub(in crate::db_operations) entity_type: EntityType,
    pub(in crate::db_operations) entity_id: i32,
    pub(in crate::db_operations) is_inside: IsInside,
    pub(in crate::db_operations) time: DateTime<Utc>,
}

#[allow(dead_code)]
pub enum DatabaseRecord {
    Person(Person),
    Group(Group),
    GroupMembers(i32, i32),
    Log(Log),
}

pub(in crate::db_operations) fn parse_db_datetime(time_str: &str) -> rusqlite::Result<DateTime<Utc>> {
    let naive = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
    Ok(Utc.from_utc_datetime(&naive))
}
