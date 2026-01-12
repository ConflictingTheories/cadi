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

        // TODO: Validate implementations and constraints
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
                _ => {} // Implement others as needed
            }
        }
    }

    fn contract(&mut self, c: &ContractDef, context: &str) {
        if c.codec.is_none() {
            self.push_error("Contract missing required field: codec".to_string(), context);
        }
        // Example logic: if codec is h264, check constraints
    }

    fn effects(&mut self, e: &EffectsDef, context: &str) {
        // Example: check if concurrency is valid enum
        if let Some(c) = &e.concurrency {
            let valid = ["thread_safe", "single_threaded", "immutable"];
            if !valid.contains(&c.as_str()) {
                 self.push_error(format!("Invalid concurrency value: {}", c), context);
            }
        }
    }

    fn permissions(&mut self, p: &PermissionsDef, context: &str) {
         if p.allow.is_empty() && p.deny.is_empty() && p.sandbox.is_empty() {
             // Maybe warning? or just ignored.
         }
    }

    fn resources(&mut self, r: &ResourcesDef, context: &str) {
        // Check memory string format?
        for (k, v) in &r.memory {
             if k == "peak" && v.is_empty() {
                 self.push_error("Resource memory peak cannot be empty".to_string(), context);
             }
        }
    }
}
