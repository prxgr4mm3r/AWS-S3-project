use serde::Deserialize;
use sqlx::Error as SQLError;
use std::fmt;
use serde::ser::StdError;

#[derive(Debug, Deserialize)]
pub struct DBError {
    status_code: u16,
    error_message: String,
}

impl DBError {
    pub fn new(status_code: u16, error_message: String) -> DBError {
        DBError {
            status_code,
            error_message,
        }
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
            SQLError::RowNotFound => DBError::new(404, "Account not found".to_string()),
            SQLError::Database(err) => DBError::new(409, err.message().to_string()),
            err => DBError::new(500, format!("Unknown database error: {}", err)),
        }
    }
}