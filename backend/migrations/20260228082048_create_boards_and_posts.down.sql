DROP INDEX IF EXISTS idx_posts_thread;
DROP INDEX IF EXISTS idx_posts_replies;
DROP TRIGGER IF EXISTS trg_assign_post_number;
DROP FUNCTION IF EXISTS assign_post_number;
DROP TABLE IF EXISTS posts;
DROP TABLE IF EXISTS boards;
