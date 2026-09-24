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
    pub id: PrimaryKeyInteger,

    #[geekorm(unique)]
    pub username: String,

    #[geekorm(hash)]
    pub password: String,

    #[geekorm(new = "UserRole::Guest")]
    pub role: UserRole,

    #[geekorm(new = "true")]
    pub active: bool,

    #[geekorm(new = "chrono::Utc::now()")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    #[geekorm(foreign_key = "UserSessions.id")]
    pub session: ForeignKey<i32, UserSessions>,
}

/// Users session and tokens
#[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct UserSessions {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKeyInteger,

    #[geekorm(rand)]
    pub token: String,

    #[geekorm(new = "chrono::Utc::now()")]
    pub last: chrono::DateTime<chrono::Utc>,
}

impl Users {
    pub async fn create(
        connection: &Connection<'_>,
        username: impl Into<String>,
        password: impl Into<String>,
        role: UserRole,
    ) -> Result<Users> {
        let mut session = UserSessions::new();
        session.save(connection).await?;

        let mut user = Users::new(username, password, session.id);
        user.role = role;
        user.save(connection).await?;
        Ok(user)
    }

    /// Create a login function
    pub async fn login(
        connection: &Connection<'_>,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<Users> {
        let username = username.into();
        let password = password.into();
        let mut user = Users::fetch_by_username(connection, username).await?;

        // Check password using the helper
        if !user.check_password(password)? {
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
