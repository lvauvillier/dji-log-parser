use serde::Serialize;

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct App {
    // App tip
    pub tip: String,
    // App warning
    pub warn: String,
}
