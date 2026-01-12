use std::path::PathBuf;
use cadi_core::atomizer::AtomizerConfig;
use cadi_core::project_analyzer::ProjectAnalyzer;
use cadi_core::rehydration::RehydrationEngine;
use cadi_core::graph::GraphStore;
use cadi_core::graph::node::GraphNode;
use cadi_core::graph::EdgeType;

#[test]
fn import_and_view_todo_timer_web() {
    // Import the example project and create a view
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/todo-timer-web");

    let analyzer = ProjectAnalyzer::default();
    let result = analyzer.import_project(&root).expect("import failed");

    // Create an in-memory graph and insert nodes and content
    let graph = GraphStore::in_memory().expect("graph open failed");
    for chunk in &result.chunks {
        // Build graph node
        let mut node = GraphNode::new(&chunk.chunk_id, &chunk.content_hash)
            .with_alias(chunk.name.clone())
            .with_language(chunk.language.clone())
            .with_granularity(format!("{:?}", chunk.granularity))
            .with_size(chunk.size);

        // Insert node
        graph.insert_node(&node).expect("insert node failed");

        // Store content by reading the source file and slicing lines from sources
        if let Some(src) = chunk.sources.first() {
            if let Ok(content) = std::fs::read_to_string(&src.file) {
                let lines: Vec<&str> = content.lines().collect();
                let start = src.start_line.unwrap_or(1).saturating_sub(1);
                let end = src.end_line.unwrap_or(lines.len()).min(lines.len());
                let chunk_content = lines[start..end].join("\n");
                graph.store_content(&chunk.chunk_id, chunk_content.as_bytes()).expect("store content failed");
            }
        }
    }

    // Create a view for the imported atoms
    let engine = RehydrationEngine::new(graph);
    let atom_ids: Vec<String> = result.chunks.iter().map(|c| c.chunk_id.clone()).collect();

    let rt = tokio::runtime::Runtime::new().expect("tokio rt");
    let view = rt.block_on(engine.create_view(atom_ids, Default::default())).expect("create view failed");

    assert!(view.source.contains("Todo Timer") || view.source.contains("todo-app") || view.source.contains("Todo"));
}
