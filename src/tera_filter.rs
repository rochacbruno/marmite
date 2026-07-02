use std::str::FromStr;

use tera::{Kwargs, State, TeraResult, Value};

pub struct DefaultDateFormat {
    pub date_format: String,
}

impl tera::Filter<&Value, TeraResult<Value>> for DefaultDateFormat {
    fn call(&self, value: &Value, _: Kwargs, _: &State) -> TeraResult<Value> {
        let date_str = value
            .as_str()
            .ok_or(tera::Error::message("Missing date string"))?;
        let date = chrono::NaiveDateTime::from_str(date_str)
            .map_err(|e| tera::Error::message(e.to_string()))?;
        let formatted_date = date.format(self.date_format.as_str()).to_string();

        Ok(Value::from(formatted_date))
    }
}

pub struct Slugify;

impl tera::Filter<&Value, TeraResult<Value>> for Slugify {
    fn call(&self, value: &Value, _: Kwargs, _: &State) -> TeraResult<Value> {
        let s = value
            .as_str()
            .ok_or_else(|| tera::Error::message("Expected a string for slugify filter"))?;
        Ok(Value::from(crate::slugify::slugify(s)))
    }
}

pub struct RemoveDraft;

impl tera::Filter<&Value, TeraResult<Value>> for RemoveDraft {
    fn call(&self, value: &Value, _: Kwargs, _: &State) -> TeraResult<Value> {
        let items = value
            .as_array()
            .ok_or_else(|| tera::Error::message("Expected an array"))?;

        let filtered: Vec<Value> = items
            .iter()
            .filter(|item| {
                // Check if the item has a stream field that equals "draft"
                if let Some(stream) = item.get_from_path("stream") {
                    if let Some(stream_str) = stream.as_str() {
                        return stream_str != "draft";
                    }
                }
                // If no stream field or stream is not a string, include the item
                true
            })
            .cloned()
            .collect();

        Ok(Value::from_serializable(&filtered))
    }
}

/// Tera 1.x `date` filter - moved to tera-contrib in Tera 2.0
#[allow(clippy::needless_pass_by_value)]
pub fn date(val: &Value, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
    use chrono::TimeZone;
    let format: &str = kwargs.get::<&str>("format")?.unwrap_or("%Y-%m-%d");
    let date_str = val
        .as_str()
        .ok_or_else(|| tera::Error::message("date filter requires a string value"))?;
    let parse_naive = |s: &str| -> Option<chrono::NaiveDateTime> {
        chrono::NaiveDateTime::from_str(s).ok().or_else(|| {
            chrono::NaiveDate::from_str(s)
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
        })
    };
    if let Some(dt) = parse_naive(date_str) {
        let utc = chrono::Utc.from_utc_datetime(&dt);
        return Ok(Value::from(utc.format(format).to_string()));
    }
    Err(tera::Error::message(format!("Invalid date: '{date_str}'")))
}

/// Tera 1.x `striptags` filter - removed in Tera 2.0
pub fn striptags(val: &str, _: Kwargs, _: &State) -> String {
    let re = regex::Regex::new(r"<[^>]*>").expect("Invalid HTML tag pattern");
    re.replace_all(val, "").to_string()
}

/// Tera 1.x `trim_start_matches` filter - renamed to `trim_start` in Tera 2.0
#[allow(clippy::needless_pass_by_value)]
pub fn trim_start_matches(val: &str, kwargs: Kwargs, _: &State) -> TeraResult<String> {
    if let Some(pat) = kwargs.get::<&str>("pat")? {
        Ok(val.trim_start_matches(pat).to_string())
    } else {
        Ok(val.trim_start().to_string())
    }
}

/// Tera 1.x `slice` filter - removed in Tera 2.0
#[allow(
    clippy::needless_pass_by_value,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn slice(val: &Value, kwargs: Kwargs, _: &State) -> TeraResult<Value> {
    let arr = val
        .as_array()
        .ok_or_else(|| tera::Error::message("slice filter requires an array"))?;
    let start = kwargs.get::<i64>("start")?.unwrap_or(0).max(0) as usize;
    let end = kwargs
        .get::<i64>("end")?
        .map_or(arr.len(), |e| e.max(0) as usize)
        .min(arr.len());
    let sliced: Vec<Value> = arr[start..end].to_vec();
    Ok(Value::from_serializable(&sliced))
}

#[cfg(test)]
#[path = "tests/tera_filter.rs"]
mod tests;
