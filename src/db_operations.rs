//! Database access layer.
//!
//! This is a facade module that keeps the original `db_operations::*` API stable
//! while the implementation is split across smaller files in `src/db_operations/`.
//!
//! Split overview:
//! - `types.rs`: DB-facing domain types (Person/Group/...) and enum ↔ SQL glue
//! - `schema.rs`: schema creation + initial seed data
//! - `queries.rs`: read/query helpers
//! - `insert.rs`, `update.rs`, `delete.rs`: write helpers
//! - `path.rs`: DB location
//! - `connection.rs`: open connection + run schema

mod connection;
mod delete;
mod insert;
mod path;
mod queries;
mod schema;
mod types;
mod update;

pub use connection::get_db;
#[allow(unused_imports)]
pub use delete::delete_from_db;
pub use insert::insert_to_db;
#[allow(unused_imports)]
pub use queries::{get_group, get_group_member, get_group_with_members, get_person, get_log};
pub use types::{IsInside, Methodology, RankLevel, Person, Group, GroupWithMembers, DatabaseRecord};
#[allow(unused_imports)]
pub use update::update_db;
pub use update::set_person_is_inside;

// Internal-only items shared across db submodules.
pub(in crate::db_operations) use types::Log;
pub(in crate::db_operations) use types::parse_db_datetime;
