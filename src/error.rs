use serde::ser::StdError;
use serde::Deserialize;
use sqlx::Error as SQLError;
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct DBError {
    error_message: String,
}

impl DBError {
    pub fn new(error_message: String) -> DBError {
        DBError { error_message }
    }
}

impl StdError for DBError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}
impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.error_message.as_str())
    }
}

impl From<SQLError> for DBError {
    fn from(error: SQLError) -> DBError {
        match error {
            SQLError::RowNotFound => DBError::new("Account not found".to_string()),
            SQLError::Database(err) => DBError::new(err.message().to_string()),
            err => DBError::new(format!("Unknown database error: {}", err)),
        }
    }
}
