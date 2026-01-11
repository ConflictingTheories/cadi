-- =============================================================================
-- CADI PostgreSQL Initialization
-- =============================================================================
-- Schema for persistent metadata storage (optional)

-- Extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- -----------------------------------------------------------------------------
-- Chunks Table
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS chunks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    chunk_id VARCHAR(255) NOT NULL UNIQUE,
    chunk_type VARCHAR(50) NOT NULL,
    size_bytes BIGINT NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    content_type VARCHAR(100) DEFAULT 'application/octet-stream',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    
    CONSTRAINT valid_chunk_type CHECK (chunk_type IN ('source', 'ir', 'blob', 'container'))
);

CREATE INDEX idx_chunks_chunk_id ON chunks(chunk_id);
CREATE INDEX idx_chunks_type ON chunks(chunk_type);
CREATE INDEX idx_chunks_created ON chunks(created_at);
CREATE INDEX idx_chunks_metadata ON chunks USING gin(metadata);

-- -----------------------------------------------------------------------------
-- Signatures Table
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS signatures (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    chunk_id VARCHAR(255) NOT NULL REFERENCES chunks(chunk_id) ON DELETE CASCADE,
    signer_id VARCHAR(255) NOT NULL,
    signature_algorithm VARCHAR(50) NOT NULL,
    signature BYTEA NOT NULL,
    public_key TEXT NOT NULL,
    signed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    UNIQUE(chunk_id, signer_id)
);

CREATE INDEX idx_signatures_chunk ON signatures(chunk_id);
CREATE INDEX idx_signatures_signer ON signatures(signer_id);

-- -----------------------------------------------------------------------------
-- Build Receipts Table
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS build_receipts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    chunk_id VARCHAR(255) NOT NULL REFERENCES chunks(chunk_id) ON DELETE CASCADE,
    builder_id VARCHAR(255) NOT NULL,
    build_environment JSONB NOT NULL,
    input_chunks JSONB NOT NULL,
    build_command TEXT,
    build_duration_ms INTEGER,
    built_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    signature BYTEA
);

CREATE INDEX idx_receipts_chunk ON build_receipts(chunk_id);
CREATE INDEX idx_receipts_builder ON build_receipts(builder_id);

-- -----------------------------------------------------------------------------
-- Trusted Signers Table
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS trusted_signers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    signer_pattern VARCHAR(255) NOT NULL,
    trust_level VARCHAR(50) NOT NULL DEFAULT 'verify',
    description TEXT,
    public_key TEXT,
    added_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    added_by VARCHAR(255),
    
    CONSTRAINT valid_trust_level CHECK (trust_level IN ('full', 'verify', 'minimal'))
);

CREATE UNIQUE INDEX idx_signers_pattern ON trusted_signers(signer_pattern);

-- -----------------------------------------------------------------------------
-- Access Tokens Table
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS access_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    token_hash VARCHAR(64) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    scopes JSONB DEFAULT '["read"]',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    last_used_at TIMESTAMP WITH TIME ZONE,
    revoked_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_tokens_hash ON access_tokens(token_hash);

-- -----------------------------------------------------------------------------
-- Audit Log Table
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    actor_id VARCHAR(255),
    details JSONB DEFAULT '{}',
    ip_address INET,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_audit_action ON audit_log(action);
CREATE INDEX idx_audit_resource ON audit_log(resource_type, resource_id);
CREATE INDEX idx_audit_actor ON audit_log(actor_id);
CREATE INDEX idx_audit_created ON audit_log(created_at);

-- -----------------------------------------------------------------------------
-- Functions
-- -----------------------------------------------------------------------------

-- Update timestamp function
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply to chunks table
CREATE TRIGGER chunks_updated_at
    BEFORE UPDATE ON chunks
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

-- -----------------------------------------------------------------------------
-- Initial Data
-- -----------------------------------------------------------------------------

-- Default trusted signers
INSERT INTO trusted_signers (signer_pattern, trust_level, description) VALUES
    ('github.com/cadi-project/*', 'full', 'Official CADI project'),
    ('local', 'full', 'Local development')
ON CONFLICT DO NOTHING;

-- Done
SELECT 'CADI database initialized successfully' AS status;
