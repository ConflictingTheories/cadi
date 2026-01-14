/// MCP Tools - Token-Optimized for "NO READ" Pattern
/// 
/// The key innovation: MCP tools return INTERFACES not SOURCE CODE.
/// This is what enables LLMs to use CADI without reading all the code.

use serde_json::json;
use cadi_core::interface::ComponentInterface;

/// Response from cadi_search
/// Returns: Component interfaces (NOT source code)
pub fn search_response(results: Vec<ComponentInterface>) -> serde_json::Value {
    json!({
        "count": results.len(),
        "results": results.iter().map(|interface| {
            interface.to_mcp_response()
        }).collect::<Vec<_>>(),
        "note": "Use cadi_get_interface to see full details. Use cadi_build to compile."
    })
}

/// Response from cadi_get_interface
/// Returns: Full interface metadata (still NOT source code)
pub fn get_interface_response(interface: ComponentInterface) -> serde_json::Value {
    let mut response = interface.to_mcp_response();
    
    // Add more details not in summary
    if let serde_json::Value::Object(ref mut obj) = response {
        obj.insert("behavior".to_string(), json!(interface.behavior));
        obj.insert("preconditions".to_string(), 
            json!(interface.preconditions));
        obj.insert("postconditions".to_string(), 
            json!(interface.postconditions));
        obj.insert("constraints".to_string(), 
            json!(interface.constraints));
        
        // Side effects (LLM needs to know if this does I/O!)
        obj.insert("side_effects".to_string(), 
            json!(interface.side_effects.iter().map(|s| s.to_string()).collect::<Vec<_>>()));
        
        // All composition points
        obj.insert("composition_points".to_string(),
            json!(interface.composition_points.iter().map(|p| json!({
                "name": p.name,
                "expected_type": p.expected_type.name,
                "required": p.required,
                "description": p.description,
            })).collect::<Vec<_>>()));
        
        // Full examples
        obj.insert("examples".to_string(), json!(interface.usage_examples));
    }
    
    response
}

/// Response when LLM asks "How do I compose these?"
/// Returns: What fits together
pub fn composition_advice(
    component_a: &ComponentInterface,
    potential_matches: Vec<(ComponentInterface, f64)>,
) -> serde_json::Value {
    json!({
        "component_a": {
            "id": component_a.id,
            "name": component_a.summary,
        },
        "potential_fits": potential_matches.into_iter().map(|(comp, score)| {
            json!({
                "id": comp.id,
                "name": comp.summary,
                "confidence": score,
                "composition_points": comp.composition_points.iter().map(|p| json!({
                    "name": p.name,
                    "can_accept": comp.id,
                })).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
        "next_step": "Use cadi_build with CBS specification to assemble"
    })
}

/// Response when source code IS needed (for actual build)
/// Only then do we return source
pub fn get_source_response(
    interface: &ComponentInterface,
    source_code: String,
) -> serde_json::Value {
    json!({
        "id": interface.id,
        "source": source_code,
        "summary": interface.summary,
        "warning": "This is the full source - only shown when needed for build/execution"
    })
}

/// When an LLM asks "Do these components match?"
pub fn compatibility_check(
    component_a: &ComponentInterface,
    component_b: &ComponentInterface,
) -> serde_json::Value {
    // Check if B's inputs match A's outputs, or vice versa
    let a_output_type = &component_a.output.name;
    let b_input_types: Vec<&String> = component_b.inputs.iter().map(|p| &p.type_sig.name).collect();

    let is_compatible = b_input_types.contains(&a_output_type);

    json!({
        "component_a": {
            "id": component_a.id,
            "outputs": component_a.output.name,
        },
        "component_b": {
            "id": component_b.id,
            "inputs": component_b.inputs.iter()
                .map(|p| json!({"name": p.name, "type": p.type_sig.name}))
                .collect::<Vec<_>>(),
        },
        "compatible": is_compatible,
        "reason": if is_compatible {
            "Output type of A matches input type of B".to_string()
        } else {
            format!("{} doesn't match any input to {}", a_output_type, component_b.id)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cadi_core::interface::ComponentInterface;

    #[test]
    fn test_search_response_is_small() {
        let interface = ComponentInterface::minimal(
            "cadi://fn/test".to_string(),
            "Test".to_string(),
            "fn test()".to_string(),
        );

        let response = search_response(vec![interface]);
        let json_str = serde_json::to_string(&response).unwrap();

        // Should be tiny - definitely not full code
        assert!(json_str.len() < 500);
    }

    #[test]
    fn test_interface_response_includes_metadata() {
        let mut interface = ComponentInterface::minimal(
            "cadi://fn/test".to_string(),
            "Test".to_string(),
            "fn test()".to_string(),
        );
        interface.behavior = "This does something useful".to_string();

        let response = get_interface_response(interface);
        
        assert!(response["behavior"].is_string());
        assert_eq!(response["behavior"].as_str().unwrap(), "This does something useful");
    }
}
