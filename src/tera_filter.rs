use std::str::FromStr;

use tera::{to_value, Filter};

pub struct DefaultDateFormat {
    pub date_format: String,
}

impl Filter for DefaultDateFormat {
    fn filter(
        &self,
        value: &tera::Value,
        _: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let date_str = value
            .as_str()
            .ok_or(tera::Error::msg("Missing date string"))?;
        let date = chrono::NaiveDateTime::from_str(date_str)
            .map_err(|e| tera::Error::msg(e.to_string()))?;
        let formatted_date = date.format(self.date_format.as_str()).to_string();

        to_value(formatted_date).map_err(tera::Error::from)
    }
}
