/// Queries relating to a particular customer

use crate::Query;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use std::borrow::Cow;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use anyhow::Error;
use std::fmt;
use std::convert::TryFrom;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct Data<T> {
    data: T,
}

impl<T> Data<T> {
    pub fn unwrap(self) -> T {
        self.data
    }
}

impl<T> Data<T> {
    fn proxy<'de, D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        Data::deserialize(deserializer).map(|data: Self| data.data)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Services {
    #[serde(deserialize_with = "Data::proxy")]
    broadband: Vec<BroadbandService>,
    #[serde(deserialize_with = "Data::proxy")]
    mobile: Vec<MobileService>,
    #[serde(deserialize_with = "Data::proxy")]
    phone: Vec<PhoneService>,
    #[serde(deserialize_with = "Data::proxy")]
    voip: Vec<VoipService>,
}

pub(crate) struct GetServices;

impl Query for GetServices {
    type Body = ();
    type Response = Data<Services>;

    fn path(&self) -> Cow<str> {
        "/service".into()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Service {
    id: u64,
    description: String,
    monthly_charge: Price,
    #[serde(serialize_with = "unparse_date")]
    #[serde(deserialize_with = "parse_date")]
    contract_start_date: NaiveDate,
    #[serde(serialize_with = "unparse_date")]
    #[serde(deserialize_with = "parse_date")]
    contract_end_date: NaiveDate,
    current_contract: u64,
    billing_cycle_progress_percentage: Percentage,
    in_contract: bool,
    payment_via: String,
    payment_expiry: Option<String>,
    plan_change: bool,
    service_number: String,
    service_type: String,
    #[serde(serialize_with = "unparse_short_date")]
    #[serde(deserialize_with = "parse_short_date")]
    next_billing_cycle_start: NaiveDate,
    #[serde(flatten)]
    rest: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BroadbandService {
    #[serde(flatten)]
    service: Service,
}

impl AsRef<Service> for BroadbandService {
    fn as_ref(&self) -> &Service {
        &self.service
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MobileService {
    #[serde(flatten)]
    service: Service,
}

impl AsRef<Service> for MobileService {
    fn as_ref(&self) -> &Service {
        &self.service
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PhoneService {
    #[serde(flatten)]
    service: Service,
}

impl AsRef<Service> for PhoneService {
    fn as_ref(&self) -> &Service {
        &self.service
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VoipService {
    #[serde(flatten)]
    service: Service,
}

impl AsRef<Service> for VoipService {
    fn as_ref(&self) -> &Service {
        &self.service
    }
}

/// A monetary price
#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(try_from = "String")]
#[serde(into = "String")]
pub struct Price(u32);

impl FromStr for Price {
    type Err = Error;

    fn from_str(mut text: &str) -> Result<Self, Self::Err> {
        if text.len() >= 1 && &text[0..1] == "$" {
            text = &text[1..];
        }

        let mut value = 0;
        let mut amounts = text.split('.').collect::<Vec<_>>();
        if amounts.len() == 1 {
            amounts.push("0");
        }
        for amount in amounts.iter().take(2) {
            value *= 100;
            value += amount.parse::<u32>()?;
        };

        Ok(Price(value))
    }
}

impl TryFrom<String> for Price {
    type Error = Error;

    fn try_from(text: String) -> Result<Self, Error> {
        text.parse()
    }
}

impl Into<String> for Price {
    fn into(self) -> String {
        format!("{}", self)
    }
}

impl fmt::Display for Price {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "${}.{:02}", self.0 / 100, self.0 % 100)
    }
}

impl fmt::Debug for Price {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

fn parse_short_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let text: &str = Deserialize::deserialize(deserializer)?;
    NaiveDate::parse_from_str(text, "%e %b %y").map_err(|err| D::Error::custom(format!("{}", err)))
}

fn parse_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let text: &str = Deserialize::deserialize(deserializer)?;
    NaiveDate::parse_from_str(text, "%e %b %Y").map_err(|err| D::Error::custom(format!("{}", err)))
}

fn unparse_short_date<S: Serializer>(
    date: &NaiveDate,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("{}", date.format("%e %b %y")))
}

fn unparse_date<S: Serializer>(
    date: &NaiveDate,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&format!("{}", date.format("%e %b %Y")))
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(from = "u32")]
#[serde(into = "u32")]
pub struct Percentage(u32);

impl From<u32> for Percentage {
    fn from(percent: u32) -> Self {
        Percentage(percent)
    }
}

impl Into<u32> for Percentage {
    fn into(self) -> u32 {
        self.0
    }
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl fmt::Debug for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
