use cadi_core::parser::parse_file;
use cadi_core::validator::Validator;

#[test]
fn test_validate_valid_doc() {
    let input = r#"
    @interface VideoCodec {
        @contract {
            codec: "h264"
        }
    }
    "#;
    let doc = parse_file(input).expect("Parse failed");
    let validator = Validator::new();
    assert!(validator.validate(&doc).is_ok());
}

#[test]
fn test_validate_duplicate_interface() {
    let input = r#"
    @interface A {}
    @interface A {}
    "#;
    let doc = parse_file(input).expect("Parse failed");
    let validator = Validator::new();
    let res = validator.validate(&doc);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert_eq!(errs[0].message, "Duplicate interface name: A");
}

#[test]
fn test_validate_duplicate_method() {
    let input = r#"
    @interface A {
        foo() -> void
        foo() -> int
    }
    "#;
    let doc = parse_file(input).expect("Parse failed");
    let validator = Validator::new();
    let res = validator.validate(&doc);
    assert!(res.is_err());
    let errs = res.unwrap_err();
    assert_eq!(errs[0].message, "Duplicate method name: foo");
}

#[test]
fn test_validate_missing_contract_field() {
    let input = r#"
    @interface A {
        @contract {
            // Missing codec
        }
    }
    "#;
    let doc = parse_file(input).expect("Parse failed");
    let validator = Validator::new();
    let res = validator.validate(&doc);
    assert!(res.is_err());
    assert!(res.unwrap_err()[0].message.contains("required field: codec"));
}

#[test]
fn test_validate_invalid_effects_concurrency() {
    let input = r#"
    @interface A {
        @effects {
            concurrency: "unsafe_mode"
        }
    }
    "#;
    let doc = parse_file(input).expect("Parse failed");
    let validator = Validator::new();
    let res = validator.validate(&doc);
    assert!(res.is_err());
    assert!(res.unwrap_err()[0].message.contains("Invalid concurrency value"));
}
