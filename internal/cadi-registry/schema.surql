-- 1. DEFINE TABLES (NODES)
DEFINE TABLE atom SCHEMAFULL;
DEFINE FIELD hash ON TABLE atom TYPE string;
DEFINE FIELD language ON TABLE atom TYPE string;
DEFINE FIELD content ON TABLE atom TYPE string; -- The raw code
DEFINE FIELD embedding ON TABLE atom TYPE array<float>; -- For Semantic Search

DEFINE TABLE concept SCHEMAFULL; -- "Abstract" ideas like "Sort Array"
DEFINE FIELD description ON TABLE concept TYPE string;

-- 2. DEFINE RELATIONS (EDGES)
-- Atom -> Depends on -> Interface
DEFINE TABLE depends_on SCHEMAFULL TYPE RELATION FROM atom TO atom; -- Simplified for Phase 1

-- Atom -> Implements -> Concept
DEFINE TABLE implements SCHEMAFULL TYPE RELATION FROM atom TO concept;

-- 3. DEFINE INDEXES
DEFINE INDEX idx_atom_hash ON atom COLUMNS hash UNIQUE;
DEFINE INDEX idx_embedding ON atom COLUMNS embedding TYPE HNSW DIMENSION 768 DIST COSINE;
