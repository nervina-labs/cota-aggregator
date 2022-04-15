use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct IssuerInfo {
    pub version:     String,
    pub name:        String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar:      Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct ClassInfo {
    pub version:        String,
    pub name:           String,
    pub image:          String,
    pub cota_id:        String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol:         Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description:    Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio:          Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video:          Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model:          Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub characteristic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties:     Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct InfoData<T> {
    #[serde(skip_serializing)]
    pub target: String,
    #[serde(rename = "type")]
    pub type_:  String,
    pub data:   T,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Metadata<T> {
    #[serde(skip_serializing)]
    pub id:       String,
    #[serde(skip_serializing)]
    pub ver:      String,
    pub metadata: InfoData<T>,
}
