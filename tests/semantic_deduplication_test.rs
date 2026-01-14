use cadi_core::normalizer::SemanticNormalizer;
use cadi_core::deduplication::DeduplicationEngine;

#[test]
fn test_identical_semantics_same_hash() {
    let code_a = r#"
        function addNumbers(x, y) {
            return x + y;
        }
    "#;

    let code_b = r#"
        function   add  (  a  ,  b  )   {
            return    a    +    b  ;
        }
    "#;

    let normalizer = SemanticNormalizer::new("typescript").unwrap();
    let result_a = normalizer.normalize(code_a).unwrap();
    let result_b = normalizer.normalize(code_b).unwrap();

    assert_eq!(result_a.hash, result_b.hash);
}

#[test]
fn test_deduplication_engine() {
    let mut engine = DeduplicationEngine::new();

    let hash = "semantic:abc123";

    let (is_new_1, equiv_1) = engine.register_chunk("chunk1", hash);
    assert!(is_new_1);
    assert!(equiv_1.is_empty());

    let (is_new_2, equiv_2) = engine.register_chunk("chunk2", hash);
    assert!(!is_new_2);
    assert_eq!(equiv_2, vec!["chunk1"]);

    let found = engine.find_equivalents(hash);
    assert_eq!(found, vec!["chunk1", "chunk2"]);
}

#[tokio::test]
async fn test_semantic_similarity_threshold() {
    let code_a = "function add(x, y) { return x + y; }";
    let code_b = "function add(x, y) { return x + y + 1; }";

    let (is_identical, similarity) = cadi_core::deduplication::DeduplicationEngine::check_similarity(
        code_a,
        code_b,
        "typescript",
    )
    .await
    .unwrap();

    assert!(!is_identical);
    assert!(similarity > 0.8); // Very similar but not identical
}
