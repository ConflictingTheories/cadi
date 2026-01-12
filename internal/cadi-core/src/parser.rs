use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use std::collections::HashMap;
use crate::ast::*;

#[derive(Parser)]
#[grammar = "src/grammar.pest"] 
// Note: path is relative to Cargo.toml of the crate usually
pub struct CadlParser;

pub fn parse_file(input: &str) -> Result<CadlDocument, String> {
    let mut pairs = CadlParser::parse(Rule::file, input)
        .map_err(|e| format!("{}", e))?;

    let mut doc = CadlDocument {
        interfaces: Vec::new(),
        implementations: Vec::new(),
        constraints: Vec::new(),
        top_level_annotations: Vec::new(),
    };

    for pair in pairs.next().unwrap().into_inner() {
        match pair.as_rule() {
            Rule::interface_def => {
                doc.interfaces.push(parse_interface(pair));
            }
            Rule::impl_def => {
                doc.implementations.push(parse_impl(pair));
            }
            Rule::annotation => {
                let ann = parse_annotation(pair);
                // Check if it's strictly a constraint block (special case in AST usually, 
                // but here we might treat @constraints as just another annotation or promote it)
                // For the spec, @constraints { ... } is a block.
                if let Annotation::Unknown(name, map) = &ann {
                    if name == "constraints" {
                        doc.constraints.push(ConstraintDef {
                            name: None,
                            rules: Vec::new(), // TODO: parse internals better
                            other: map.clone(),
                        });
                        continue;
                    }
                }
                doc.top_level_annotations.push(ann);
            }
            Rule::EOI => (),
            _ => (),
        }
    }

    Ok(doc)
}

fn parse_interface(pair: Pair<Rule>) -> InterfaceDef {
    let mut inner = pair.into_inner();
    // input string: @interface Name { ... }
    // first token is "interface" keyword (implicit), second is identifier
    // Wait, grammar: "@interface" ~ identifier ~ "{" ~ interface_content ~ "}"
    
    let name = inner.next().unwrap().as_str().to_string();
    let content = inner.next().unwrap();

    let mut methods = Vec::new();
    let mut annotations = Vec::new();

    for item in content.into_inner() {
        match item.as_rule() {
            Rule::method_def => {
                methods.push(parse_method(item));
            }
            Rule::annotation => {
                annotations.push(parse_annotation(item));
            }
            _ => {}
        }
    }

    InterfaceDef {
        name,
        methods,
        annotations,
    }
}

fn parse_method(pair: Pair<Rule>) -> MethodDef {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    
    // Check if params exist
    let next = inner.next().unwrap();
    let (params, return_type) = if next.as_rule() == Rule::params {
        let params = parse_params(next);
        (params, inner.next().unwrap().as_str().trim().to_string())
    } else {
        (Vec::new(), next.as_str().trim().to_string())
    };

    MethodDef {
        name,
        params,
        return_type,
    }
}

fn parse_params(pair: Pair<Rule>) -> Vec<ParamDef> {
    let mut params = Vec::new();
    for p in pair.into_inner() {
        let mut inner = p.into_inner();
        let name = inner.next().unwrap().as_str().to_string();
        let type_name = inner.next().unwrap().as_str().to_string();
        params.push(ParamDef { name, type_name });
    }
    params
}

fn parse_annotation(pair: Pair<Rule>) -> Annotation {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let content = inner.next().unwrap(); // object or block_body

    let map = parse_object_like(content);

    match name.as_str() {
        "contract" => Annotation::Contract(from_map_contract(map)),
        "effects" => Annotation::Effects(from_map_effects(map)),
        "abi" => Annotation::Abi(from_map_abi(map)),
        "data_format" => Annotation::DataFormat(from_map_data_format(map)),
        "resources" => Annotation::Resources(from_map_resources(map)),
        "protocol" => Annotation::Protocol(from_map_protocol(map)),
        "numerical" => Annotation::Numerical(from_map_numerical(map)),
        "observability" => Annotation::Observability(from_map_observability(map)),
        "permissions" => Annotation::Permissions(from_map_permissions(map)),
        _ => Annotation::Unknown(name, map),
    }
}

fn parse_object_like(pair: Pair<Rule>) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    for item in pair.into_inner() {
        match item.as_rule() {
            Rule::pair => {
                let mut inner = item.into_inner();
                let key = inner.next().unwrap().as_str().to_string();
                let val = parse_value(inner.next().unwrap());
                map.insert(key, val);
            }
            Rule::named_block => {
                let mut inner = item.into_inner();
                let key = inner.next().unwrap().as_str().to_string();
                let content = inner.next().unwrap();
                let val = Value::Object(parse_object_like(content));
                map.insert(key, val);
            }
            // TODO: handle nested annotations in blocks if necessary
            _ => {}
        }
    }
    map
}

fn parse_value(pair: Pair<Rule>) -> Value {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::string => {
             // Remove quotes
             let s = inner.as_str();
             Value::String(s[1..s.len()-1].to_string())
        },
        Rule::number => {
            let n = inner.as_str().parse::<f64>().unwrap_or(0.0);
            Value::Number(n)
        },
        Rule::boolean => {
            let b = inner.as_str() == "true";
            Value::Bool(b)
        },
        Rule::identifier => Value::String(inner.as_str().to_string()),
        Rule::array => {
             let vals = inner.into_inner().map(parse_value).collect();
             Value::Array(vals)
        },
        Rule::object => {
             Value::Object(parse_object_like(inner))
        },
        Rule::null_val => Value::String("null".to_string()),
        _ => Value::String(inner.as_str().to_string()),
    }
}

fn parse_impl(pair: Pair<Rule>) -> ImplDef {
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let content = inner.next().unwrap();

    let mut attributes = HashMap::new();
    let mut annotations = Vec::new();

    for item in content.into_inner() {
        match item.as_rule() {
            Rule::pair => {
                let mut p_inner = item.into_inner();
                let key = p_inner.next().unwrap().as_str().to_string();
                let val = parse_value(p_inner.next().unwrap());
                attributes.insert(key, val);
            }
            Rule::annotation => {
                annotations.push(parse_annotation(item));
            }
            _ => {}
        }
    }

    ImplDef {
        name,
        attributes,
        annotations,
    }
}

// Helpers to convert HashMap<String, Value> to specific structs
fn from_map_contract(mut map: HashMap<String, Value>) -> ContractDef {
    ContractDef {
        codec: extract_string(&mut map, "codec"),
        profile: extract_string(&mut map, "profile"),
        container: extract_string(&mut map, "container"),
        ensures: extract_string_list(&mut map, "ensures"),
        complexity: extract_string_map(&mut map, "complexity"),
        other: map,
    }
}

fn from_map_effects(mut map: HashMap<String, Value>) -> EffectsDef {
    EffectsDef {
        concurrency: extract_string(&mut map, "concurrency"),
        io: extract_string_list(&mut map, "io"),
        memory: extract_string(&mut map, "memory"),
        mutates: extract_string_list(&mut map, "mutates"),
        reads: extract_string_list(&mut map, "reads"),
        blocking: extract_string(&mut map, "blocking"),
        async_exec: extract_string(&mut map, "async"),
        other: map,
    }
}

// Minimal stubs for others to allow compilation for now
fn from_map_abi(mut map: HashMap<String, Value>) -> AbiDef {
    AbiDef {
        string_encoding: extract_string(&mut map, "string_encoding"),
        memory_ownership: extract_string_map(&mut map, "memory_ownership"),
        error_model: extract_string(&mut map, "error_model"),
        other: map,
    }
}

fn from_map_data_format(mut map: HashMap<String, Value>) -> DataFormatDef {
    DataFormatDef {
        input: extract_data_format_spec_map(&mut map, "input"),
        output: extract_data_format_spec_map(&mut map, "output"),
    }
}

fn extract_data_format_spec_map(map: &mut HashMap<String, Value>, key: &str) -> HashMap<String, DataFormatSpec> {
    match map.remove(key) {
        Some(Value::Object(obj)) => obj.into_iter().map(|(k, v)| {
            let spec = match v {
                Value::Object(mut inner_map) => DataFormatSpec {
                    format: extract_string(&mut inner_map, "format"),
                    schema: extract_string(&mut inner_map, "schema"),
                    value_range: extract_number_list(&mut inner_map, "value_range"),
                    other: inner_map,
                },
                _ => DataFormatSpec {
                    format: None,
                    schema: None,
                    value_range: None,
                    other: HashMap::new(),
                }
            };
            (k, spec)
        }).collect(),
        _ => HashMap::new(),
    }
}

fn from_map_resources(mut map: HashMap<String, Value>) -> ResourcesDef {
    ResourcesDef {
        requires: extract_string_list(&mut map, "requires"),
        memory: extract_string_map(&mut map, "memory"),
        cpu_time: extract_string(&mut map, "cpu_time"),
        gpu: extract_string_map(&mut map, "gpu"),
        other: map,
    }
}

fn from_map_protocol(mut map: HashMap<String, Value>) -> ProtocolDef {
    ProtocolDef {
        states: extract_string_list(&mut map, "states"),
        initial: extract_string(&mut map, "initial"),
        transitions: extract_string_list(&mut map, "transitions"),
        other: map,
    }
}

fn from_map_numerical(mut map: HashMap<String, Value>) -> NumericalDef {
    NumericalDef {
        precision: extract_string(&mut map, "precision"),
        error_bounds: extract_string_map(&mut map, "error_bounds"),
        other: map,
    }
}

fn from_map_observability(mut map: HashMap<String, Value>) -> ObservabilityDef {
    // Just putting raw values for now
    ObservabilityDef {
        logging: extract_map(&mut map, "logging"),
        metrics: extract_map(&mut map, "metrics"),
        tracing: extract_map(&mut map, "tracing"),
    }
}

fn from_map_permissions(mut map: HashMap<String, Value>) -> PermissionsDef {
    PermissionsDef {
        allow: extract_string_list(&mut map, "allow"),
        deny: extract_string_list(&mut map, "deny"),
        sandbox: extract_string_map(&mut map, "sandbox"),
    }
}

// Extraction helpers
fn extract_string(map: &mut HashMap<String, Value>, key: &str) -> Option<String> {
    match map.remove(key) {
        Some(Value::String(s)) => Some(s),
        Some(v) => Some(format!("{:?}", v)), // Fallback
        None => None,
    }
}

fn extract_string_list(map: &mut HashMap<String, Value>, key: &str) -> Vec<String> {
    match map.remove(key) {
        Some(Value::Array(arr)) => arr.into_iter().filter_map(|v| match v {
            Value::String(s) => Some(s),
            _ => None,
        }).collect(),
        _ => Vec::new(),
    }
}

fn extract_string_map(map: &mut HashMap<String, Value>, key: &str) -> HashMap<String, String> {
    match map.remove(key) {
        Some(Value::Object(obj)) => obj.into_iter().map(|(k, v)| (k, match v {
            Value::String(s) => s,
            _ => format!("{:?}", v),
        })).collect(),
        _ => HashMap::new(),
    }
}

fn extract_map(map: &mut HashMap<String, Value>, key: &str) -> HashMap<String, Value> {
    match map.remove(key) {
        Some(Value::Object(obj)) => obj,
        _ => HashMap::new(),
    }
}

fn extract_number_list(map: &mut HashMap<String, Value>, key: &str) -> Option<Vec<f64>> {
    match map.remove(key) {
        Some(Value::Array(arr)) => Some(arr.into_iter().filter_map(|v| match v {
            Value::Number(n) => Some(n),
            _ => None,
        }).collect()),
        _ => None,
    }
}
