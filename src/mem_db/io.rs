use std::env;

use diesel::prelude::*;

use super::MemoryDBError;

pub fn establish_connection() -> Result<SqliteConnection, MemoryDBError> {
    let database_url =
        env::var("DATABASE_URL").map_err(|_| MemoryDBError::DatabaseUrlNotSpecified)?;

    Ok(SqliteConnection::establish(&database_url)?)
}
