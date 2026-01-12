use crate::ast::*;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub path: String,
}

pub struct Validator {
    errors: Vec<ValidationError>,
}

impl Validator {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn validate(mut self, doc: &CadlDocument) -> Result<(), Vec<ValidationError>> {
        self.doc(doc);
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors)
        }
    }

    fn push_error(&mut self, message: String, path: &str) {
        self.errors.push(ValidationError {
            message,
            path: path.to_string(),
        });
    }

    fn doc(&mut self, doc: &CadlDocument) {
        let mut interface_names = HashSet::new();
        for iface in &doc.interfaces {
            if !interface_names.insert(&iface.name) {
                self.push_error(format!("Duplicate interface name: {}", iface.name), "root");
            }
            self.interface(iface);
        }

        for (i, imp) in doc.implementations.iter().enumerate() {
            self.implementation(imp, i);
        }

        for (i, con) in doc.constraints.iter().enumerate() {
            self.constraint(con, i);
        }
    }

    fn interface(&mut self, iface: &InterfaceDef) {
        let mut method_names = HashSet::new();
        for method in &iface.methods {
            if !method_names.insert(&method.name) {
                self.push_error(format!("Duplicate method name: {}", method.name), &iface.name);
            }
        }

        for ann in &iface.annotations {
            match ann {
                Annotation::Contract(c) => self.contract(c, &iface.name),
                Annotation::Effects(e) => self.effects(e, &iface.name),
                Annotation::Permissions(p) => self.permissions(p, &iface.name),
                Annotation::Resources(r) => self.resources(r, &iface.name),
                Annotation::DataFormat(df) => self.data_format(df, &iface.name),
                Annotation::Protocol(proto) => self.protocol(proto, &iface.name),
                Annotation::Numerical(num) => self.numerical(num, &iface.name),
                Annotation::Observability(obs) => self.observability(obs, &iface.name),
                Annotation::Abi(abi) => self.abi(abi, &iface.name),
                Annotation::Unknown(name, _) => {
                    // Unknown annotations at the interface level might be fine, but we can warn/error if desired.
                    tracing::debug!("Unknown annotation: {} on interface {}", name, iface.name);
                }
            }
        }
    }

    fn implementation(&mut self, imp: &ImplDef, index: usize) {
        if imp.name.is_empty() {
             self.push_error(format!("Implementation at index {} has no name", index), "implementations");
        }
        // Check if it follows InterfaceName.ImplName format
        if !imp.name.contains('.') {
            self.push_error(format!("Implementation name '{}' should be in 'Interface.Implementation' format", imp.name), &imp.name);
        }
    }

    fn constraint(&mut self, con: &ConstraintDef, index: usize) {
        if con.rules.is_empty() && con.other.is_empty() {
            self.push_error(format!("Constraint at index {} is empty", index), "constraints");
        }
    }

    fn contract(&mut self, c: &ContractDef, context: &str) {
        if c.codec.is_none() && c.other.is_empty() {
            self.push_error("Contract missing descriptive fields (codec, etc.)".to_string(), context);
        }
    }

    fn effects(&mut self, e: &EffectsDef, context: &str) {
        if let Some(c) = &e.concurrency {
            let valid = ["thread_safe", "single_threaded", "immutable", "reentrant"];
            if !valid.contains(&c.as_str()) {
                 self.push_error(format!("Invalid concurrency value: {}", c), context);
            }
        }
    }

    fn permissions(&mut self, p: &PermissionsDef, context: &str) {
         if p.allow.is_empty() && p.deny.is_empty() && p.sandbox.is_empty() {
              self.push_error("Permissions annotation is empty".to_string(), context);
         }
    }

    fn resources(&mut self, r: &ResourcesDef, context: &str) {
        for (k, v) in &r.memory {
             if k == "peak" && v.is_empty() {
                 self.push_error("Resource memory peak cannot be empty".to_string(), context);
             }
        }
    }

    fn data_format(&mut self, _df: &DataFormatDef, _context: &str) {
        // Placeholder for deeper data format validation
    }

    fn protocol(&mut self, proto: &ProtocolDef, context: &str) {
        if !proto.states.is_empty() && proto.initial.is_none() {
            self.push_error("Protocol with states must have an initial state".to_string(), context);
        }
        if let Some(initial) = &proto.initial {
            if !proto.states.contains(initial) {
                self.push_error(format!("Initial state '{}' not found in states list", initial), context);
            }
        }
    }

    fn numerical(&mut self, _num: &NumericalDef, _context: &str) {
        // Placeholder for numerical precision validation
    }

    fn observability(&mut self, _obs: &ObservabilityDef, _context: &str) {
        // Placeholder
    }

    fn abi(&mut self, abi: &AbiDef, context: &str) {
        if let Some(encoding) = &abi.string_encoding {
            let valid = ["utf8", "utf16", "ascii", "cesu8"];
            if !valid.contains(&encoding.as_str()) {
                self.push_error(format!("Invalid string encoding: {}", encoding), context);
            }
        }
    }
}

