-- Guild configuration table
CREATE TABLE IF NOT EXISTS guild_configs (
    guild_id BIGINT PRIMARY KEY,
    audit_channel_id BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Panels table
CREATE TABLE IF NOT EXISTS panels (
    id UUID PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    style VARCHAR(20) NOT NULL DEFAULT 'button',
    color INTEGER NOT NULL DEFAULT 5793266,
    channel_id BIGINT,
    message_id BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT chk_panels_style CHECK (style IN ('button', 'select_menu'))
);

-- Panel roles table
CREATE TABLE IF NOT EXISTS panel_roles (
    id UUID PRIMARY KEY,
    panel_id UUID NOT NULL REFERENCES panels(id) ON DELETE CASCADE,
    role_id BIGINT NOT NULL,
    label VARCHAR(80) NOT NULL,
    emoji VARCHAR(100),
    description VARCHAR(100),
    position INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_panels_guild_id ON panels(guild_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_panels_guild_name ON panels(guild_id, name);
CREATE INDEX IF NOT EXISTS idx_panels_message_id ON panels(message_id) WHERE message_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_panel_roles_panel_id ON panel_roles(panel_id);
CREATE INDEX IF NOT EXISTS idx_panel_roles_role_id ON panel_roles(role_id);

-- Updated at trigger function
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply trigger to tables with updated_at
DROP TRIGGER IF EXISTS guild_configs_updated_at ON guild_configs;
CREATE TRIGGER guild_configs_updated_at
    BEFORE UPDATE ON guild_configs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();

DROP TRIGGER IF EXISTS panels_updated_at ON panels;
CREATE TRIGGER panels_updated_at
    BEFORE UPDATE ON panels
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();
