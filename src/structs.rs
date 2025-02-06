use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct ApiResponse {
    pub(crate) result: ContestResult,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ContestResult {
    pub(crate) contest: Contest,
    pub(crate) rows: Vec<Row>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Contest {
    pub(crate) id: i32,
    pub(crate) name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Row {
    pub(crate) rank: i32,
    #[serde(deserialize_with = "deserialize_f64_as_i32")]
    pub(crate) points: i32,
    #[serde(default)]
    pub(crate) score: f32,
    pub(crate) party: Party,
}
fn deserialize_f64_as_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = f64::deserialize(deserializer)?; // Deserialize as f64
    if value.fract() == 0.0 {
        Ok(value as i32) // Convert safely to i32
    } else {
        Err(de::Error::custom(format!(
            "Expected integer but found {}",
            value
        )))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Party {
    pub(crate) members: Vec<Member>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Member {
    pub(crate) handle: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Battle {
    pub(crate) group_code: String,
    pub(crate) contest_id: String,
}
