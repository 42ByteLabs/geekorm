use anyhow::Result;
use geekorm::{ConnectionManager, GEEKORM_BANNER, GEEKORM_VERSION, prelude::*};

#[derive(Table, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Users {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKey<i32>,

    #[geekorm(unique)]
    pub username: String,

    #[geekorm(new = "TwoFactorAuth::new()")]
    pub tfa: TwoFactorAuth,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}     v{}\n", GEEKORM_BANNER, GEEKORM_VERSION);

    // Generate an initial TwoFactorAuth
    let totp = TwoFactorAuth::new();
    let value: geekorm::Value = totp.into();
    assert!(matches!(value, geekorm::Value::Json(_)));

    // Initialize an in-memory database
    let db = ConnectionManager::in_memory().await?;
    let connection = db.acquire().await;

    Users::create_table(&connection).await?;

    // Create a new user with a TwoFactorAuth
    let mut user = Users::new("geekmasher");
    user.fetch_or_create(&connection).await?;

    println!("User({}, '{}') :: {}\n", user.id, user.username, user.tfa);

    // Generate a new TOTP token
    let totp_token = user.tfa.generate_current()?;
    println!("Token :: {}", totp_token);

    // Check the TOTP token (this example will always return true)
    if user.tfa.check(totp_token.as_str())? {
        println!("Token is valid");
    } else {
        println!("Token is invalid");
    }

    Ok(())
}
