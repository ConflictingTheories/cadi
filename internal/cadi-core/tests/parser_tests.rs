use cadi_core::parser::parse_file;
use cadi_core::ast::*;

#[test]
fn test_parse_simple_interface() {
    let input = r#"
    @interface VideoCodec {
        encode(frame: ImageData, quality: u8) -> Blob
    }
    "#;
    
    let doc = parse_file(input).expect("Failed to parse");
    assert_eq!(doc.interfaces.len(), 1);
    assert_eq!(doc.interfaces[0].name, "VideoCodec");
    assert_eq!(doc.interfaces[0].methods.len(), 1);
    assert_eq!(doc.interfaces[0].methods[0].name, "encode");
    assert_eq!(doc.interfaces[0].methods[0].params.len(), 2);
    assert_eq!(doc.interfaces[0].methods[0].return_type, "Blob");
}

#[test]
fn test_parse_with_contract() {
    let input = r#"
    @interface VideoCodec {
        @contract {
            codec: "h264"
            profile: "baseline"
            ensures: ["decode(encode(x)) == x"]
        }
    }
    "#;

    let doc = parse_file(input).expect("Failed to parse");
    assert_eq!(doc.interfaces.len(), 1);
    let contract = doc.interfaces[0].annotations.iter().find_map(|a| match a {
        Annotation::Contract(c) => Some(c),
        _ => None,
    }).expect("Contract not found");

    assert_eq!(contract.codec.as_deref(), Some("h264"));
    assert_eq!(contract.profile.as_deref(), Some("baseline"));
    assert_eq!(contract.ensures.len(), 1);
}

#[test]
fn test_parse_full_v2_spec_features() {
    let input = r#"
    @interface Advanced {
        @effects {
            concurrency: "thread_safe"
            io: ["network.send"]
        }
        @resources {
            memory: { peak: "1GB" }
        }
    }
    "#;

    let doc = parse_file(input).expect("Failed to parse");
    assert_eq!(doc.interfaces.len(), 1);
    
    let effects = doc.interfaces[0].annotations.iter().find_map(|a| match a {
        Annotation::Effects(e) => Some(e),
        _ => None,
    }).expect("Effects not found");
    assert_eq!(effects.concurrency.as_deref(), Some("thread_safe"));
    assert_eq!(effects.io.len(), 1);

    let resources = doc.interfaces[0].annotations.iter().find_map(|a| match a {
        Annotation::Resources(r) => Some(r),
        _ => None,
    }).expect("Resources not found");
    
    assert!(resources.memory.contains_key("peak"));
}

#[test]
fn test_parse_impl_and_constraints() {
    let input = r#"
    @impl VideoCodec.webgl1 {
        chunk_id: "chunk:sha256:abc123..."
        caps: ["webgl>=1.0"]
        
        @contract_impl {
            codec: "h264"
        }
    }
    
    @constraints {
        match_contract {
             // simplified rule for now
        }
    }
    "#;
    
    let doc = parse_file(input).expect("Failed to parse");
    assert_eq!(doc.implementations.len(), 1);
    assert_eq!(doc.implementations[0].name, "VideoCodec.webgl1");
    // caps is an attribute
    assert!(doc.implementations[0].attributes.contains_key("caps"));
    
    // Check constraints
    assert_eq!(doc.constraints.len(), 1);
}
