use cadi_core::atomizer::{AtomExtractor, AtomizerConfig};
use cadi_core::graph::{GraphStore, GraphNode, EdgeType};
use cadi_core::rehydration::RehydrationEngine;
use cadi_core::hash;

#[tokio::test]
async fn integration_atomizer_virtual_view() {
    // Prepare simple Rust source snippets
    let src_a = r#"pub fn helper() -> i32 { 42 }
"#;
    let src_b = r#"pub fn use_helper() -> i32 { helper() }
"#;

    // Extract atoms
    let extractor = AtomExtractor::new("rust", AtomizerConfig::default());
    let atoms_a = extractor.extract(src_a).expect("Failed to extract atoms A");
    let atoms_b = extractor.extract(src_b).expect("Failed to extract atoms B");

    // Pick first function atom from each
    let atom_a = atoms_a.into_iter().find(|a| a.kind.is_executable()).expect("No function in A");
    let atom_b = atoms_b.into_iter().find(|a| a.kind.is_executable()).expect("No function in B");

    // Initialize an in-memory GraphStore
    let graph = GraphStore::in_memory().expect("Failed to create graph store");

    // Helper to insert extracted atom as a GraphNode and store content
    let insert_atom = |graph: &GraphStore, atom: &cadi_core::atomizer::ExtractedAtom, filename: &str| {
        let content = atom.source.as_bytes();
        let chunk_id = hash::chunk_id_from_content(content);
        let content_hash = hash::sha256_bytes(content);

        let mut node = GraphNode::new(chunk_id.clone(), content_hash.clone())
            .with_language("rust")
            .with_size(content.len())
            .with_defines(atom.defines.clone())
            .with_references(atom.references.clone())
            .with_source(filename.to_string(), atom.start_line, atom.end_line);

        // Insert node & content
        graph.insert_node(&node).expect("insert_node failed");
        graph.store_content(&chunk_id, content).expect("store_content failed");

        chunk_id
    };

    let id_a = insert_atom(&graph, &atom_a, "a.rs");
    let id_b = insert_atom(&graph, &atom_b, "b.rs");

    // Wire dependencies based on references
    // For each reference in each node, try to resolve symbol to a defining chunk
    let maybe_node_b = graph.get_node(&id_b).expect("get_node b failed").unwrap();
    for sym in maybe_node_b.symbols_referenced.iter() {
        if let Ok(Some(target_chunk)) = graph.find_symbol(sym) {
            // add dependency: b depends on target
            graph.add_dependency(&id_b, &target_chunk, EdgeType::Imports).expect("add_dependency failed");
        }
    }

    // Now create a rehydration engine and assemble a view for chunk B with expansion
    let engine = RehydrationEngine::new(graph);

    let view = engine.create_view(vec![id_b.clone()], cadi_core::rehydration::config::ViewConfig::default().with_expansion(1)).await.expect("create_view failed");

    // Expect the view to include both B and A (A as ghost import)
    assert!(view.atoms.contains(&id_b), "View should contain the original atom B");
    assert!(view.atoms.contains(&id_a), "View should contain atom A via ghost import");
    assert!(view.ghost_atoms.contains(&id_a), "Atom A should be listed as a ghost atom");

    // The assembled source should include the helper function name
    assert!(view.source.contains("helper"), "View source should contain helper symbol");

    // Also verify view_for_symbol can locate the defining atom
    let view_sym = engine.view_for_symbol("helper", cadi_core::rehydration::config::ViewConfig::default()).await.expect("view_for_symbol failed");
    assert!(view_sym.source.contains("helper"), "Symbol view should include helper");
}
