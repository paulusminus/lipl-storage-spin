use std::str::from_utf8;

use base64::{prelude::BASE64_STANDARD, Engine};
use spin_sdk::http::{IntoResponse, Response, ResponseBuilder};

use crate::error::AuthenticationError;

pub fn unauthenticated() -> Response {
    ResponseBuilder::new(401)
        .header("WWW-Authenticate", "Basic realm=\"Lipl Api\"")
        .build()
        .into_response()
}

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl std::str::FromStr for Credentials {
    type Err = AuthenticationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = BASE64_STANDARD
            .decode(s)?;
        let decoded_s = from_utf8(decoded.as_slice())?;
        let mut splitted = decoded_s.split(':');
        let username = splitted.next().ok_or(AuthenticationError::Username)?;
        let password = splitted.next().ok_or(AuthenticationError::Password)?;
        Ok(Self {
            username: username.to_owned(),
            password: password.to_owned(),
        })
    }
}

pub enum Authentication {
    Basic(Credentials),
}

impl Authentication {
    pub fn is_valid_user(&self, username: String, password: String) -> bool {
        match self {
            Self::Basic(basic) => basic.username == username && basic.password == password,
        }
    }
}

impl std::fmt::Display for Authentication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic(basic) => write!(f, "Basic {} {}", basic.username, basic.password),
        }
    }
}

impl std::str::FromStr for Authentication {
    type Err = AuthenticationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_prefix("Basic ") {
            stripped.parse::<Credentials>().map(Authentication::Basic)
        } else {
            Err(AuthenticationError::Unsupported)
        }
    }
}

#[cfg(test)]
mod test {
    use std::env::var;

    use base64::{engine::general_purpose::STANDARD, Engine};

    use crate::basic_authentication::Authentication;

    #[test]
    fn calculate_authorization_header() {
        let username = var("LIPL_USERNAME").unwrap();
        let password = var("LIPL_PASSWORD").unwrap();

        let combined = format!("{}:{}", username, password);
        println!("{combined}");
        let encoded = STANDARD.encode(combined);
        println!("{encoded}");

        let header = format!("Basic {}", encoded);

        let credentials = header.parse::<Authentication>().unwrap();

        match credentials {
            Authentication::Basic(credentials) => {
                println!("{}", &credentials.username);
                println!("{}", &credentials.password);
            }
        }
    }
}
