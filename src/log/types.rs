#[derive(
    Debug,
    Clone,
    strum::Display,
    strum::EnumString,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}
