//! Repository traits. Services depend on these traits, not on
//! `Surreal<C>` directly. Tests swap a `MockUserRepo` for the same
//! signature without round-tripping a real database.

pub mod matches;
pub mod participant;
pub mod room;
pub mod submission;
pub mod tournament;
pub mod user;
