use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CadlDocument {
    pub interfaces: Vec<InterfaceDef>,
    pub implementations: Vec<ImplDef>,
    pub constraints: Vec<ConstraintDef>,
    pub top_level_annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InterfaceDef {
    pub name: String,
    pub methods: Vec<MethodDef>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImplDef {
    pub name: String, // e.g. VideoCodec.webgl1
    pub attributes: HashMap<String, Value>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstraintDef {
    pub name: Option<String>,
    pub rules: Vec<String>, // simplified for now
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MethodDef {
    pub name: String,
    pub params: Vec<ParamDef>,
    pub return_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParamDef {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Annotation {
    Contract(ContractDef),
    Effects(EffectsDef),
    Abi(AbiDef),
    DataFormat(DataFormatDef),
    Resources(ResourcesDef),
    Protocol(ProtocolDef),
    Numerical(NumericalDef),
    Observability(ObservabilityDef),
    Permissions(PermissionsDef),
    Unknown(String, HashMap<String, Value>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

// @contract
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContractDef {
    pub codec: Option<String>,
    pub profile: Option<String>,
    pub container: Option<String>,
    pub ensures: Vec<String>,
    pub complexity: HashMap<String, String>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

// @effects
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EffectsDef {
    pub concurrency: Option<String>,
    pub io: Vec<String>,
    pub memory: Option<String>,
    pub mutates: Vec<String>,
    pub reads: Vec<String>,
    pub blocking: Option<String>,
    pub async_exec: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

// @abi
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AbiDef {
    pub string_encoding: Option<String>,
    pub memory_ownership: HashMap<String, String>,
    pub error_model: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

// @data_format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataFormatDef {
    pub input: HashMap<String, DataFormatSpec>,
    pub output: HashMap<String, DataFormatSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DataFormatSpec {
    pub format: Option<String>,
    pub schema: Option<String>,
    pub value_range: Option<Vec<f64>>, // Simple numeric range for now
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

// @resources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourcesDef {
    pub requires: Vec<String>,
    pub memory: HashMap<String, String>,
    pub cpu_time: Option<String>,
    pub gpu: HashMap<String, String>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

// @protocol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtocolDef {
    pub states: Vec<String>,
    pub initial: Option<String>,
    pub transitions: Vec<String>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

// @numerical
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NumericalDef {
    pub precision: Option<String>,
    pub error_bounds: HashMap<String, String>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

// @observability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObservabilityDef {
    pub logging: HashMap<String, Value>,
    pub metrics: HashMap<String, Value>,
    pub tracing: HashMap<String, Value>,
}

// @permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PermissionsDef {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub sandbox: HashMap<String, String>,
}
