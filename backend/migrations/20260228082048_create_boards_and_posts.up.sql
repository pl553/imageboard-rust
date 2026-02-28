CREATE TABLE boards (
    id BIGSERIAL PRIMARY KEY,
    slug VARCHAR(10) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    post_counter BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE posts (
    board_id BIGINT NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    post_number BIGINT NOT NULL,
    parent_number BIGINT,
    name VARCHAR(100) NOT NULL DEFAULT 'Anonymous',
    text TEXT NOT NULL,
    image_filename VARCHAR(255),
    image_thumbnail VARCHAR(255),
    image_original_name VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (board_id, post_number),
    
    FOREIGN KEY (board_id, parent_number) 
        REFERENCES posts(board_id, post_number) 
        ON DELETE CASCADE
);

-- Trigger function to auto-assign post_number
CREATE OR REPLACE FUNCTION assign_post_number()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE boards 
    SET post_counter = post_counter + 1 
    WHERE id = NEW.board_id 
    RETURNING post_counter INTO NEW.post_number;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_assign_post_number
    BEFORE INSERT ON posts
    FOR EACH ROW
    EXECUTE FUNCTION assign_post_number();

CREATE INDEX idx_posts_threads ON posts(board_id, created_at DESC) 
    WHERE parent_number IS NULL;

CREATE INDEX idx_posts_replies ON posts(board_id, parent_number, created_at ASC) 
    WHERE parent_number IS NOT NULL;
