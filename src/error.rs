use std::fmt;

use twilight_http::response::DeserializeBodyError;
use twilight_http::Error as HttpError;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    NameExists,
    NotFound(&'static str),
    LimitExceeded(&'static str),
    Permission(String),
    Database(sqlx::Error),
    Discord(String),
    InvalidInput(String),
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NameExists => write!(f, "A panel with this name already exists."),
            AppError::NotFound(resource) => write!(f, "{} not found.", resource),
            AppError::LimitExceeded(resource) => write!(f, "{} limit exceeded.", resource),
            AppError::Permission(msg) => write!(f, "{}", msg),
            AppError::Database(e) => write!(f, "Database error: {}", e),
            AppError::Discord(msg) => write!(f, "Discord error: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "{}", msg),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Database(e) => Some(e),
            _ => None,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<HttpError> for AppError {
    fn from(err: HttpError) -> Self {
        AppError::Discord(err.to_string())
    }
}

impl From<DeserializeBodyError> for AppError {
    fn from(err: DeserializeBodyError) -> Self {
        AppError::Discord(err.to_string())
    }
}

impl AppError {
    pub fn user_message(&self) -> &str {
        match self {
            AppError::NameExists => "この名前のパネルは既に存在します。",
            AppError::NotFound(resource) => match *resource {
                "Panel" => "パネルが見つかりませんでした。",
                "Role" => "ロールが見つかりませんでした。",
                _ => "リソースが見つかりませんでした。",
            },
            AppError::LimitExceeded(_) => "1パネルあたり最大25ロールまでです。",
            AppError::Permission(msg) => msg,
            AppError::Database(_) => "エラーが発生しました。もう一度お試しください。",
            AppError::Discord(_) => "エラーが発生しました。もう一度お試しください。",
            AppError::InvalidInput(msg) => msg,
            AppError::Internal(_) => "エラーが発生しました。もう一度お試しください。",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_exists() {
        let err = AppError::NameExists;
        assert_eq!(format!("{}", err), "A panel with this name already exists.");
    }

    #[test]
    fn display_not_found() {
        let err = AppError::NotFound("Panel");
        assert_eq!(format!("{}", err), "Panel not found.");

        let err = AppError::NotFound("Role");
        assert_eq!(format!("{}", err), "Role not found.");
    }

    #[test]
    fn display_limit_exceeded() {
        let err = AppError::LimitExceeded("Role");
        assert_eq!(format!("{}", err), "Role limit exceeded.");
    }

    #[test]
    fn display_permission() {
        let err = AppError::Permission("You need admin rights".to_string());
        assert_eq!(format!("{}", err), "You need admin rights");
    }

    #[test]
    fn display_discord() {
        let err = AppError::Discord("API error".to_string());
        assert_eq!(format!("{}", err), "Discord error: API error");
    }

    #[test]
    fn display_invalid_input() {
        let err = AppError::InvalidInput("Invalid name".to_string());
        assert_eq!(format!("{}", err), "Invalid name");
    }

    #[test]
    fn display_internal() {
        let err = AppError::Internal("Something broke".to_string());
        assert_eq!(format!("{}", err), "Internal error: Something broke");
    }

    #[test]
    fn user_message_name_exists() {
        let err = AppError::NameExists;
        assert_eq!(err.user_message(), "この名前のパネルは既に存在します。");
    }

    #[test]
    fn user_message_not_found_panel() {
        let err = AppError::NotFound("Panel");
        assert_eq!(err.user_message(), "パネルが見つかりませんでした。");
    }

    #[test]
    fn user_message_not_found_role() {
        let err = AppError::NotFound("Role");
        assert_eq!(err.user_message(), "ロールが見つかりませんでした。");
    }

    #[test]
    fn user_message_not_found_unknown() {
        let err = AppError::NotFound("Something");
        assert_eq!(err.user_message(), "リソースが見つかりませんでした。");
    }

    #[test]
    fn user_message_limit_exceeded() {
        let err = AppError::LimitExceeded("Role");
        assert_eq!(err.user_message(), "1パネルあたり最大25ロールまでです。");
    }

    #[test]
    fn user_message_permission_shows_custom_message() {
        let err = AppError::Permission("Custom permission error".to_string());
        assert_eq!(err.user_message(), "Custom permission error");
    }

    #[test]
    fn user_message_database_hides_details() {
        // User should not see database error details
        let err = AppError::Database(sqlx::Error::RowNotFound);
        assert_eq!(
            err.user_message(),
            "エラーが発生しました。もう一度お試しください。"
        );
    }

    #[test]
    fn user_message_discord_hides_details() {
        let err = AppError::Discord("Detailed API error".to_string());
        assert_eq!(
            err.user_message(),
            "エラーが発生しました。もう一度お試しください。"
        );
    }

    #[test]
    fn user_message_invalid_input_shows_message() {
        let err = AppError::InvalidInput("Name too long".to_string());
        assert_eq!(err.user_message(), "Name too long");
    }

    #[test]
    fn user_message_internal_hides_details() {
        let err = AppError::Internal("Stack trace...".to_string());
        assert_eq!(
            err.user_message(),
            "エラーが発生しました。もう一度お試しください。"
        );
    }

    #[test]
    fn error_source_for_database_error() {
        use std::error::Error;
        let err = AppError::Database(sqlx::Error::RowNotFound);
        assert!(err.source().is_some());
    }

    #[test]
    fn error_source_for_other_errors() {
        use std::error::Error;
        let err = AppError::NameExists;
        assert!(err.source().is_none());

        let err = AppError::Discord("test".to_string());
        assert!(err.source().is_none());
    }
}
