//! This module is responsible for all database queries
//! via [`sqlx`]. Each submodule represent a database entity
//! and defines functions for CRUD operations.
//!
//! Since theses functions are meant to be used by the [`crate::routes`]
//! module, they should assume validation has already occured and return
//! only database errors on failures.

mod data;
mod viz;

use crate::error::{ApiError, ApiResult};
pub use {data::*, viz::*};

pub enum Relation {
    Owned,
    NotOwned,
    Absent,
}

impl Relation {
    pub fn to_api_result(self) -> ApiResult<()> {
        match self {
            Relation::Owned => Ok(()),
            Relation::NotOwned => Err(ApiError::Forbidden),
            Relation::Absent => Err(ApiError::NotFound),
        }
    }
}
