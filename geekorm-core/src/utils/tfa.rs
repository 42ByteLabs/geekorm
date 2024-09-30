//! # Two Factor Authentication
//!
//! ```rust
//! # #[cfg(feature = "two-factor-auth")] {
//! use geekorm::prelude::*;
//!
//! let tfa = TwoFactorAuth::new();
//!
//! let code = tfa.generate_current().unwrap();
//! # assert_eq!(code.len(), 6);
//!
//! let value: Value = tfa.into();
//! # assert!(matches!(value, geekorm::Value::Json(_)));
//!
//! let totp2: TwoFactorAuth = value.into();
//! # }
//! ```

use crate::Value;
use std::fmt::Display;
use totp_rs::{Algorithm, Secret, TOTP};

/// Two Factor Authentication
#[derive(Debug, Clone, serde::Serialize)]
pub struct TwoFactorAuth {
    totp: TOTP,
}

impl TwoFactorAuth {
    /// Create a new TwoFactorAuth
    ///
    /// If `qr` feature is enabled, it will use the `KONARR_TFA_ISSUER` and `KONARR_TFA_ACCOUNT_NAME` environment variables
    pub fn new() -> Self {
        #[cfg(feature = "two-factor-auth-qr")]
        let issuer = match std::env::var("GEEKORM_TFA_ISSUER") {
            Ok(issuer) => Some(issuer),
            Err(_) => Some(env!("CARGO_PKG_NAME").to_string()),
        };
        #[cfg(feature = "two-factor-auth-qr")]
        let account_name = match std::env::var("GEEKORM_TFA_ACCOUNT_NAME") {
            Ok(account_name) => account_name,
            Err(_) => env!("CARGO_PKG_NAME").to_string(),
        };

        Self {
            totp: totp_rs::TOTP {
                secret: Secret::generate_secret().to_bytes().unwrap(),
                algorithm: Algorithm::SHA256,
                digits: 6,
                skew: 1,
                step: 30,
                #[cfg(feature = "two-factor-auth-qr")]
                issuer,
                #[cfg(feature = "two-factor-auth-qr")]
                account_name,
            },
        }
    }

    /// Create a new TwoFactorAuth with an issuer and account name
    #[cfg(feature = "two-factor-auth-qr")]
    pub fn new_with_issuer(issuer: impl Into<String>, account_name: impl Into<String>) -> Self {
        Self {
            totp: totp_rs::TOTP {
                algorithm: Algorithm::SHA256,
                digits: 6,
                skew: 1,
                step: 30,
                secret: Secret::generate_secret().to_bytes().unwrap(),
                issuer: Some(issuer.into()),
                account_name: account_name.into(),
            },
        }
    }

    /// Generate a new TOTP
    pub fn generate_current(&self) -> Result<String, crate::Error> {
        self.totp
            .generate_current()
            .map_err(|e| crate::Error::TotpError(e.to_string()))
    }

    /// Check the one-time passcode is valid
    pub fn check<'a>(&self, code: impl Into<&'a str>) -> Result<bool, crate::Error> {
        Ok(self.totp.check_current(code.into())?)
    }
}

impl Display for TwoFactorAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "TwoFactorAuth({}, {}, {})",
            self.totp.algorithm, self.totp.skew, self.totp.step
        )
    }
}

impl From<TwoFactorAuth> for Value {
    fn from(value: TwoFactorAuth) -> Self {
        serde_json::to_vec(&value.totp)
            .map(|s| Value::Json(s))
            .unwrap_or(Value::Null)
    }
}

impl From<&TwoFactorAuth> for Value {
    fn from(value: &TwoFactorAuth) -> Self {
        serde_json::to_vec(&value.totp)
            .map(|s| Value::Json(s))
            .unwrap_or(Value::Null)
    }
}

impl From<Value> for TwoFactorAuth {
    fn from(value: Value) -> Self {
        match value {
            Value::Blob(s) | Value::Json(s) => serde_json::from_slice(&s).unwrap(),
            Value::Text(t) => serde_json::from_str(&t).unwrap(),
            _ => {
                panic!("Error parsing unknown type")
            }
        }
    }
}

impl<'de> serde::de::Deserialize<'de> for TwoFactorAuth {
    fn deserialize<D>(deserializer: D) -> Result<TwoFactorAuth, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        /// Custom vistor for TOTP
        pub struct TFAVisitor;

        impl<'de> serde::de::Visitor<'de> for TFAVisitor {
            type Value = TwoFactorAuth;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a TwoFactorAuth struct")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                serde_json::from_str(value).map_err(serde::de::Error::custom)
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let totp: totp_rs::TOTP = serde::de::Deserialize::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?;

                Ok(TwoFactorAuth { totp })
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let v: Vec<u8> = serde::de::Deserialize::deserialize(
                    serde::de::value::SeqAccessDeserializer::new(seq),
                )?;
                self.visit_bytes(&v)
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                serde_json::from_slice(value).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_struct("TwoFactorAuth", &["totp"], TFAVisitor)
    }
}
