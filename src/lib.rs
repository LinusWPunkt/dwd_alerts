use chrono::Utc;
use reqwest::blocking;
use serde::Deserialize;

const API_URL: &str = "https://www.dwd.de/DWD/warnungen/warnapp/json/warnings.json";

#[derive(Debug)]
pub enum Error {
    DeserializationError(serde_json::Error),
    ResponseProcessingError,
    RequestResponseError(reqwest::Error),
    DateParsingError,
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        return Error::RequestResponseError(value);
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        return Error::DeserializationError(value);
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct WarningRaw {
    state: String,
    #[serde(rename = "type")]
    category: u8,
    level: u8,
    start: i64,
    end: Option<i64>,
    region_name: String,
    event: String,
    headline: String,
    instruction: String,
    description: String,
    state_short: String,
    altitude_start: Option<i64>,
    altitude_end: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
struct WarningResponse {
    time: i64,
    warnings: std::collections::HashMap<String, Vec<WarningRaw>>,
    vorab_information: std::collections::HashMap<(), ()>,
    copyright: String,
}
#[derive(Debug)]
#[allow(unused)]
pub struct Warning {
    state: String,
    category: u8,
    level: u8,
    start: chrono::DateTime<Utc>,
    end: Option<chrono::DateTime<Utc>>,
    region_name: String,
    event: String,
    headline: String,
    instruction: String,
    description: String,
    state_short: String,
    altitude_start: Option<i64>,
    altitude_end: Option<i64>,
}

impl From<WarningRaw> for Warning {
    fn from(value: WarningRaw) -> Self {
        let start = chrono::NaiveDateTime::from_timestamp_millis(value.start).unwrap();
        let start = chrono::DateTime::<Utc>::from_utc(start, Utc);
        let end = value.end;
        let end = if let Some(c) = end {
            let t = chrono::NaiveDateTime::from_timestamp_millis(c).unwrap();
            Some(chrono::DateTime::<Utc>::from_utc(t, Utc))
        } else {
            None
        };

        Warning {
            state: value.state,
            category: value.category,
            level: value.level,
            start,
            end,
            region_name: value.region_name,
            event: value.event,
            headline: value.headline,
            instruction: value.instruction,
            description: value.description,
            state_short: value.state_short,
            altitude_start: value.altitude_start,
            altitude_end: value.altitude_end,
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
pub struct WarningList {
    time: chrono::DateTime<Utc>,
    warnings: Vec<Warning>,
    copyright: String,
}

impl WarningList {
    pub fn get_new() -> Result<WarningList, Error> {
        let raw_response = blocking::get(API_URL)?.text()?;
        let data = match raw_response.strip_prefix("warnWetter.loadWarnings(") {
            Some(s) => s,
            None => return Err(Error::ResponseProcessingError),
        };
        let data = match data.strip_suffix(");") {
            Some(s) => s,
            None => return Err(Error::ResponseProcessingError),
        };
        let warnings = serde_json::from_str::<WarningResponse>(&data)?;
        Ok(WarningList::try_from(warnings)?)
    }
}

impl TryFrom<WarningResponse> for WarningList {
    type Error = Error;
    fn try_from(value: WarningResponse) -> Result<Self, Error> {
        let time = match chrono::NaiveDateTime::from_timestamp_millis(value.time) {
            Some(c) => c,
            None => return Err(Error::DateParsingError),
        };

        let time = chrono::DateTime::from_utc(time, chrono::Utc);

        let mut raw_warnings = Vec::new();

        for (_, inst) in value.warnings {
            for warning in inst {
                raw_warnings.push(warning);
            }
        }

        let mut warnings = Vec::new();

        raw_warnings
            .into_iter()
            .map(|w| warnings.push(Warning::from(w)))
            .for_each(drop);

        return Ok(WarningList {
            time,
            warnings,
            copyright: value.copyright,
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        assert!(WarningList::get_new().is_ok());
    }

    #[test]
    fn returns_at_least_1_warning() {
        assert!(WarningList::get_new().unwrap().warnings.len() >= 1);
    }

    #[test]
    #[should_panic]
    fn oob_date_fails() {
        let w_dw = WarningRaw {
            state: String::new(),
            category: 69,
            start: 7346982752374653336,
            end: None,
            region_name: String::new(),
            level: 0,
            event: String::new(),
            headline: String::new(),
            instruction: String::new(),
            description: String::new(),
            state_short: String::new(),
            altitude_start: None,
            altitude_end: None,
        };

        let _ = Warning::from(w_dw);
    }
}
