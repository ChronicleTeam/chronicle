//! This module is responsible for all database queries
//! via [`sqlx`]. Each submodule represent a database entity
//! and defines functions for CRUD operations.
//!
//! Since theses functions are meant to be used by the [`crate::routes`]
//! module, they should assume validation has already occured and not 
//! expect to return any errors.

mod access;
mod data;
mod users;
mod viz;

pub use {access::*, data::*, users::*, viz::*};
