use ed25519_dalek::SecretKey;
use ed25519_dalek::ExpandedSecretKey;
use ed25519_dalek::PublicKey;
use ed25519_dalek::Verifier;
use base64::engine::general_purpose;
use base64::Engine as _;

use serde_json::json;

#[test]
fn test_verify_attestation_signature_roundtrip() {
    // generate a keypair
    // Deterministic key from fixed seed (for test purposes)
    let seed = [42u8; 32];
    let secret = SecretKey::from_bytes(&seed).unwrap();
    let public = PublicKey::from(&secret);

    let receipt_without_sig = json!({
        "attestation": {"type": "test"},
        "data": "example",
        "signer": {"identity": {"key": {"public_key": general_purpose::STANDARD.encode(public.to_bytes())}}}
    });
    let signed_bytes = serde_json::to_vec(&receipt_without_sig).unwrap();

    let expanded = ExpandedSecretKey::from(&secret);
    let sig = expanded.sign(&signed_bytes, &public);
    let sig_b64 = general_purpose::STANDARD.encode(sig.to_bytes());
    let _pk_b64 = general_purpose::STANDARD.encode(public.to_bytes());
    let mut receipt = receipt_without_sig.clone();
    if let Some(att) = receipt.get_mut("attestation") {
        if att.is_object() {
            att.as_object_mut().unwrap().insert("signature".to_string(), json!({"algorithm": "ed25519", "value": sig_b64}));
        }
    }

    // Verify by removing signature and serializing (same bytes we signed)
    let mut verify_copy = receipt.clone();
    if let Some(att) = verify_copy.get_mut("attestation") {
        if att.is_object() {
            att.as_object_mut().unwrap().remove("signature");
        }
    }
    let verify_bytes = serde_json::to_vec(&verify_copy).unwrap();

    let sig_val = receipt.pointer("/attestation/signature/value").and_then(|v| v.as_str()).unwrap();
    let pub_b64 = receipt.pointer("/signer/identity/key/public_key").and_then(|v| v.as_str()).unwrap();

    let sig_bytes = general_purpose::STANDARD.decode(sig_val).unwrap();
    let pk_bytes = general_purpose::STANDARD.decode(pub_b64).unwrap();

    let pk = ed25519_dalek::PublicKey::from_bytes(&pk_bytes).unwrap();
    let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes).unwrap();

    assert!(pk.verify(&verify_bytes, &sig).is_ok());
}
