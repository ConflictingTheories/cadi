use cadi_builder::BuildPlanner;
use cadi_core::deduplication::DeduplicationEngine;
use std::sync::{Arc, Mutex};

#[tokio::test]
async fn test_check_for_equivalents_detects_existing() {
    let engine = Arc::new(Mutex::new(DeduplicationEngine::new()));
    let planner = BuildPlanner::new(engine.clone());

    // Prepare a canonical hash by normalizing code
    let code = "function add(x, y) { return x + y; }";
    let normalizer = cadi_core::SemanticNormalizer::new("typescript").unwrap();
    let norm = normalizer.normalize(code).unwrap();

    // Register an existing chunk with that hash
    {
        let mut e = engine.lock().unwrap();
        e.register_chunk("chunk1", &norm.hash);
    }

    let equivalents = planner.check_for_equivalents(code, "typescript", "test").await.unwrap();
    assert_eq!(equivalents.len(), 1);
    assert_eq!(equivalents[0].id, "chunk1");
}
