//! Abstract wrapper around the reqwest client

use reqwest;
use reqwest::{Url, IntoUrl, RequestBuilder};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap};
use mime::{APPLICATION_JSON, TEXT_PLAIN};
use crate::Authorization;
use anyhow::Error;
use std::convert::TryFrom;
use serde::{Serialize, de::DeserializeOwned};
use serde_json;
use std::borrow::Cow;

use crate::customer;

/// A client used to interact with the Exetel API
pub struct Client {
    authorization: Authorization,
    inner: reqwest::Client,
}

impl Client {
    /// Query exetel for the given object
    async fn query<Q: Query>(&self, query: &Q) -> Result<Q::Response, Error> {
        if let Some(body) = query.body() {
            self.post(query.url()?, body).await
        } else {
            self.get(query.url()?).await
        }
    }

    async fn post<Q, R>(&self, url: impl IntoUrl, query: &Q) -> Result<R, Error>
    where
        Q: Serialize,
        R: DeserializeOwned,
    {
        let query = serde_json::to_string(query)?;
        let request = self.inner
            .post(url)
            .body(query)
            .header(CONTENT_TYPE, APPLICATION_JSON.essence_str());
        self.request(request).await
    }

    async fn get<R>(&self, url: impl IntoUrl) -> Result<R, Error>
    where
        R: DeserializeOwned,
    {
        self.request(self.inner.get(url).header(CONTENT_TYPE, TEXT_PLAIN.essence_str())).await
    }

    async fn request<R: DeserializeOwned>(&self, request: RequestBuilder) -> Result<R, Error> {
        let response = request
            .header(ACCEPT, APPLICATION_JSON.essence_str())
            .send()
            .await?
            .text()
            .await?;
        let response = serde_json::from_str(&response)?;
        Ok(response)
    }
}

impl TryFrom<Authorization> for Client {
    type Error = Error;

    fn try_from(authorization: Authorization) -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        let bearer = format!("Bearer {}", authorization.access_token());
        headers.insert(AUTHORIZATION, bearer.parse()?);
        let inner = reqwest::ClientBuilder::new().default_headers(headers).build()?;

        Ok(Client { authorization, inner })
    }
}

impl Client {
    pub async fn services(&self) -> Result<customer::Services, Error> {
        self.query(&customer::GetServices).await.map(|data| data.unwrap())
    }
}

const URL_PREFIX: &'static str = "https://webservices.api.exetel.com.au/v1";

/// An object that can be queried from the Exetel API
pub(crate) trait Query {
    /// Type of object used for query
    type Body: Serialize;

    /// Type of response to produce
    type Response: DeserializeOwned;

    /// URL to use for query
    fn path<'q>(&'q self) -> Cow<str>;

    /// Get the URL for the query
    fn url(&self) -> Result<Url, Error> {
        Ok(format!("{}{}", URL_PREFIX, self.path()).parse()?)
    }

    /// Object to send for query
    fn body(&self) -> Option<&Self::Body> {
        None
    }
}
