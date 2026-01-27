use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommandTree {
    pub version: u32,
    pub base_url: String,
    pub resources: Vec<Resource>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Resource {
    pub name: String,
    pub ops: Vec<Operation>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Operation {
    pub name: String,
    pub method: String,
    pub path: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub params: Vec<ParamDef>,
    pub request_body: Option<RequestBody>,
    pub security: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ParamDef {
    pub name: String,
    pub flag: String,
    pub location: String,
    pub required: bool,
    pub schema_type: Option<String>,
    pub description: Option<String>,
    pub is_array: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestBody {
    pub required: bool,
    pub content_types: Vec<String>,
    pub input_fields: Vec<InputField>,
    pub required_fields: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InputField {
    pub name: String,
    pub flag: String,
    pub required: bool,
    pub schema_type: Option<String>,
    pub description: Option<String>,
}

pub fn load_command_tree() -> CommandTree {
    let raw = include_str!("../schemas/command_tree.json");
    serde_json::from_str(raw).expect("invalid command_tree.json")
}
