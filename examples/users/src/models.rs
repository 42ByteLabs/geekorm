use anyhow::Result;
use chrono::{DateTime, Utc};
use geekorm::{Connection, ConnectionManager, prelude::*};

/// User Role in the app
#[derive(Data, Debug, Clone, Default)]
pub enum UserRole {
    Admin,
    Moderator,
    User,
    #[default]
    Guest,
}

/// User table
#[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Users {
    #[geekorm(primary_key, auto_increment)]
    id: PrimaryKeyInteger,

    #[geekorm(unique)]
    username: String,

    #[geekorm(hash)]
    password: String,

    #[geekorm(new = "UserType::Guest")]
    role: UserRole,

    #[geekorm(new = true)]
    active: bool,

    #[geekorm(new = "chrono::Utc::now()")]
    created_at: chrono::DateTime<chrono::Utc>,

    #[geekorm(foreign_key = "UserSessions.id")]
    session: UserSessions,
}

/// Users session and tokens
#[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UserSessions {
    #[geekorm(primary_key, auto_increment)]
    id: PrimaryKeyInteger,

    #[geekorm(rand)]
    token: String,

    #[geekorm(new = "chrono::Utc::now()")]
    last: chrono::DateTime<chrono::Utc>,
}

impl Users {
    /// Create a login function
    pub async fn login(
        connection: &Connection<'_>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Users> {
        let mut user = Users::fetch_by_username(connection, username).await?;

        // Check password using the helper
        if !user.check_password(password.into())? {
            Err(anyhow::anyhow!("Username and password do not match"))
        } else {
            if !user.active {
                return Err(anyhow::anyhow!("User is disabled"));
            }

            // TODO

            Ok(user)
        }
    }
}
