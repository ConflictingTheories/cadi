use cadi_core::{graph::GraphStore, graph::GraphNode, hash};
use cadi_core::graph::EdgeType;
use std::fs;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = env::args().nth(1).expect("storage path arg");
    let a_path = env::args().nth(2).expect("a.rs path arg");
    let b_path = env::args().nth(3).expect("b.rs path arg");
    let a = fs::read_to_string(a_path)?.into_bytes();
    let b = fs::read_to_string(b_path)?.into_bytes();
    let id_a = hash::chunk_id_from_content(&a);
    let id_b = hash::chunk_id_from_content(&b);
    let node_a = GraphNode::new(id_a.clone(), hash::sha256_bytes(&a)).with_language("rust").with_defines(vec!["helper".to_string()]);
    let mut node_b = GraphNode::new(id_b.clone(), hash::sha256_bytes(&b)).with_language("rust");
    node_b = node_b.with_references(vec!["helper".to_string()]);

    let gs = GraphStore::open(format!("{}/graph-db", storage))?;
    gs.insert_node(&node_a)?;
    gs.store_content(&id_a, &a)?;
    gs.insert_node(&node_b)?;
    gs.store_content(&id_b, &b)?;
    gs.add_dependency(&id_b, &id_a, EdgeType::Imports)?;
    println!("inserted {} and {}", id_a, id_b);
    Ok(())
}
