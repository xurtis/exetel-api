//! Tools for authenticating to the API

use serde::{Serialize, Deserialize};
use anyhow::Error;
use std::time::{Duration, SystemTime};
use std::convert::TryInto;
use reqwest::Client;
use reqwest::header::{ORIGIN, REFERER, ACCEPT, CONTENT_TYPE};
use serde_json;
use mime::APPLICATION_JSON;
use std::fmt;

/// Encapsulation of the authentication tokens used with the API
#[derive(Debug, Serialize, Deserialize)]
pub struct Authorization {
    last_refreshed: SystemTime,
    last_response: Response,
}

impl Authorization {
    const LOGIN_URL: &'static str = "https://my.exetel.com.au/api/auth/postLogin";
    const ORIGIN_URL: &'static str = "https://my.exetel.com.au";
    const REFERER_URL: &'static str = "https://my.exetel.com.au/login";

    /// Authenticate a user with a username and password
    pub async fn authenticate(username: &str, password: &str) -> Result<Self, Error> {
        Self::authenticate_with(&mut Client::new(), username, password).await
    }

    /// Authenticate a user with an existing client
    pub async fn authenticate_with(
        client: &mut Client,
        username: &str,
        password: &str,
    ) -> Result<Self, Error> {
        let query = Query::new(username, password);
        let query = serde_json::to_string(&query)?;

        let response = client
            .post(Self::LOGIN_URL)
            .header(ORIGIN, Self::ORIGIN_URL)
            .header(REFERER, Self::REFERER_URL)
            .header(ACCEPT, APPLICATION_JSON.essence_str())
            .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
            .body(query)
            .send()
            .await?
            .text()
            .await?;

        let refreshed = SystemTime::now();
        let response = serde_json::from_str(&response)?;

        let auth = Authorization {
            last_response: response,
            last_refreshed: refreshed,
        };
        Ok(auth)
    }

    pub fn access_token(&self) -> &impl fmt::Display {
        &self.last_response.access_token
    }

    pub fn into_client(self) -> Result<crate::Client, Error> {
        self.try_into()
    }
}

impl Authorization {
    /// Refresh if the token has les than 5 minutes remaining
    const REFRESH_WINDOW: Duration = Duration::from_secs(5 * 60);

    /// Get the expiry time of the authorization
    fn expires_at(&self) -> SystemTime {
        self.last_refreshed + Duration::from_secs(self.last_response.expires_in)
    }

    /// Check if the authorization needs refreshing
    pub fn should_refresh(&self) -> bool {
        self.expires_at() + Self::REFRESH_WINDOW > SystemTime::now()
    }
}

/// Authentication request
#[derive(Default, Debug, Serialize)]
struct Query<'c> {
    #[serde(rename = "accessToken")]
    #[serde(skip_serializing_if = "Option::is_some")]
    access_token: Option<Token>,
    password: &'c str,
    #[serde(rename = "persistLogin")]
    persist_login: bool,
    username: &'c str,
}

impl<'c> Query<'c> {
    fn new(username: &'c str, password: &'c str) -> Self {
        Query {
            username,
            password,
            ..Default::default()
        }
    }
}

/// Authentication response
#[derive(Debug, Serialize, Deserialize)]
struct Response {
    /// The type of token being used
    token_type: TokenType,
    /// Number of seconds until the token expires and should be refreshed
    expires_in: u64,
    /// The actual authorization token
    access_token: Token,
    /// A token that can be used to refresh the authorization
    refresh_token: Token,
    #[serde(rename = "persistLogin")]
    persist_login: bool,
}

/// A descriptor of the manner in which the token should be used
#[derive(Debug, Serialize, Deserialize)]
enum TokenType {
    /// The token should be applied using a 'Bearer' header
    Bearer,
}

/// An authorization token
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
struct Token(String);

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
