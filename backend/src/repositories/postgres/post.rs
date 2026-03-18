use async_trait::async_trait;
use sqlx::PgPool;
use chrono::{DateTime, Utc};

use crate::domain::{DomainError, DomainResult, ImageInfo, Paginated, Post, PostId, ThreadDetail, ThreadPreview};
use crate::repositories::PostRepository;

pub struct PostgresPostRepository {
    pool: PgPool,
}

impl PostgresPostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// raw row from the posts table
#[derive(sqlx::FromRow)]
struct PostRow {
    board_id: i64,
    post_number: i64,
    parent_number: Option<i64>,
    name: String,
    text: String,
    image_filename: Option<String>,
    image_thumbnail: Option<String>,
    image_original_name: Option<String>,
    image_size_bytes: Option<i64>,
    image_width: Option<i32>,
    image_height: Option<i32>,
    image_mime_type: Option<String>,
    created_at: DateTime<Utc>,
}

impl From<PostRow> for Post {
    fn from(row: PostRow) -> Self {
        let image = match (
            row.image_filename,
            row.image_thumbnail,
            row.image_original_name,
            row.image_mime_type,
        ) {
            (Some(filename), Some(thumbnail_filename), Some(original_name), Some(mime_type)) => {
                Some(ImageInfo {
                    filename,
                    thumbnail_filename,
                    original_name,
                    size_bytes: row.image_size_bytes.unwrap_or(0),
                    width: row.image_width.unwrap_or(0),
                    height: row.image_height.unwrap_or(0),
                    mime_type,
                })
            }
            _ => None,
        };

        Post {
            board_id: row.board_id,
            post_number: row.post_number,
            parent_number: row.parent_number,
            name: row.name,
            text: row.text,
            image,
            created_at: row.created_at,
        }
    }
}

// used by find_thread_previews - includes reply_count and row_num for filtering
#[derive(sqlx::FromRow)]
struct ThreadPreviewRow {
    board_id: i64,
    post_number: i64,
    parent_number: Option<i64>,
    name: String,
    text: String,
    image_filename: Option<String>,
    image_thumbnail: Option<String>,
    image_original_name: Option<String>,
    image_size_bytes: Option<i64>,
    image_width: Option<i32>,
    image_height: Option<i32>,
    image_mime_type: Option<String>,
    created_at: DateTime<Utc>,
    reply_count: i64,
    // row number within the thread's replies (1 = most recent reply)
    // null for the OP row itself
    reply_row_num: Option<i64>,
    bumped_at: DateTime<Utc>,
}

#[async_trait]
impl PostRepository for PostgresPostRepository {
    async fn find_by_id(&self, id: PostId) -> DomainResult<Option<Post>> {
        let row = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT
                board_id, post_number, parent_number, name, text,
                image_filename, image_thumbnail, image_original_name,
                image_size_bytes, image_width, image_height, image_mime_type,
                created_at
            FROM posts
            WHERE board_id = $1 AND post_number = $2
            "#,
        )
        .bind(id.board_id)
        .bind(id.post_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.map(Post::from))
    }

    async fn create_thread(
        &self,
        board_id: i64,
        name: &str,
        text: &str,
        image: Option<&ImageInfo>,
    ) -> DomainResult<Post> {
        let row = sqlx::query_as::<_, PostRow>(
            r#"
            INSERT INTO posts (
                board_id, name, text,
                image_filename, image_thumbnail, image_original_name,
                image_size_bytes, image_width, image_height, image_mime_type
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING
                board_id, post_number, parent_number, name, text,
                image_filename, image_thumbnail, image_original_name,
                image_size_bytes, image_width, image_height, image_mime_type,
                created_at
            "#,
        )
        .bind(board_id)
        .bind(name)
        .bind(text)
        .bind(image.map(|i| &i.filename))
        .bind(image.map(|i| &i.thumbnail_filename))
        .bind(image.map(|i| &i.original_name))
        .bind(image.map(|i| i.size_bytes))
        .bind(image.map(|i| i.width))
        .bind(image.map(|i| i.height))
        .bind(image.map(|i| &i.mime_type))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(row.into())
    }

    async fn create_reply(
        &self,
        board_id: i64,
        parent_number: i64,
        name: &str,
        text: &str,
        image: Option<&ImageInfo>,
    ) -> DomainResult<Post> {
        let row = sqlx::query_as::<_, PostRow>(
            r#"
            INSERT INTO posts (
                board_id, parent_number, name, text,
                image_filename, image_thumbnail, image_original_name,
                image_size_bytes, image_width, image_height, image_mime_type
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING
                board_id, post_number, parent_number, name, text,
                image_filename, image_thumbnail, image_original_name,
                image_size_bytes, image_width, image_height, image_mime_type,
                created_at
            "#,
        )
        .bind(board_id)
        .bind(parent_number)
        .bind(name)
        .bind(text)
        .bind(image.map(|i| &i.filename))
        .bind(image.map(|i| &i.thumbnail_filename))
        .bind(image.map(|i| &i.original_name))
        .bind(image.map(|i| i.size_bytes))
        .bind(image.map(|i| i.width))
        .bind(image.map(|i| i.height))
        .bind(image.map(|i| &i.mime_type))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match &e {
            sqlx::Error::Database(db_err) if db_err.is_foreign_key_violation() => {
                DomainError::PostNotFound(parent_number)
            }
            _ => DomainError::Database(e.to_string()),
        })?;

        Ok(row.into())
    }

    async fn delete(&self, id: PostId) -> DomainResult<()> {
        let result = sqlx::query(
            "DELETE FROM posts WHERE board_id = $1 AND post_number = $2",
        )
        .bind(id.board_id)
        .bind(id.post_number)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(DomainError::PostNotFound(id.post_number));
        }

        Ok(())
    }

    async fn find_thread_previews(
        &self,
        board_id: i64,
        page: u32,
        limit: u32,
        preview_replies: u32,
    ) -> DomainResult<Paginated<ThreadPreview>> {
        let offset = (page.saturating_sub(1) * limit) as i64;
        let limit_i64 = limit as i64;
        let preview_replies_i64 = preview_replies as i64;

        // this query:
        // 1. finds all OPs for the board ordered by most recent bump (latest reply time)
        // 2. for each OP, also pulls the last N replies using a window function
        // 3. returns them all in one go, we reassemble into ThreadPreview
        let rows = sqlx::query_as::<_, ThreadPreviewRow>(
            r#"
            WITH thread_bumps AS (
                SELECT
                    parent_number,
                    MAX(created_at) AS bumped_at,
                    COUNT(*) AS reply_count
                FROM posts
                WHERE board_id = $1 AND parent_number IS NOT NULL
                GROUP BY parent_number
            ),
            ops AS (
                SELECT
                    p.board_id,
                    p.post_number,
                    p.parent_number,
                    p.name,
                    p.text,
                    p.image_filename,
                    p.image_thumbnail,
                    p.image_original_name,
                    p.image_size_bytes,
                    p.image_width,
                    p.image_height,
                    p.image_mime_type,
                    p.created_at,
                    COALESCE(tb.reply_count, 0) AS reply_count,
                    COALESCE(tb.bumped_at, p.created_at) AS bumped_at,
                    NULL::bigint AS reply_row_num
                FROM posts p
                LEFT JOIN thread_bumps tb ON tb.parent_number = p.post_number
                WHERE p.board_id = $1 AND p.parent_number IS NULL
                ORDER BY bumped_at DESC
                LIMIT $2 OFFSET $3
            ),
            replies AS (
                SELECT
                    p.board_id,
                    p.post_number,
                    p.parent_number,
                    p.name,
                    p.text,
                    p.image_filename,
                    p.image_thumbnail,
                    p.image_original_name,
                    p.image_size_bytes,
                    p.image_width,
                    p.image_height,
                    p.image_mime_type,
                    p.created_at,
                    COALESCE(tb.reply_count, 0) AS reply_count,
                    COALESCE(tb.bumped_at, ops_p.created_at) AS bumped_at,
                    ROW_NUMBER() OVER (
                        PARTITION BY p.parent_number
                        ORDER BY p.created_at DESC
                    ) AS reply_row_num
                FROM posts p
                JOIN ops ON ops.post_number = p.parent_number
                JOIN posts ops_p ON ops_p.board_id = p.board_id AND ops_p.post_number = p.parent_number
                LEFT JOIN thread_bumps tb ON tb.parent_number = p.parent_number
                WHERE p.board_id = $1 AND p.parent_number IS NOT NULL
            )
            SELECT * FROM ops
            UNION ALL
            SELECT * FROM replies WHERE reply_row_num <= $4
            ORDER BY bumped_at DESC, parent_number NULLS FIRST, created_at ASC
            "#,
        )
        .bind(board_id)
        .bind(limit_i64)
        .bind(offset)
        .bind(preview_replies_i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        let total = self.count_threads(board_id).await?;

        let previews = assemble_thread_previews(rows);

        Ok(Paginated::new(previews, page, limit, total))
    }

    async fn find_thread_detail(&self, thread: PostId) -> DomainResult<Option<ThreadDetail>> {
        let rows = sqlx::query_as::<_, PostRow>(
            r#"
            SELECT
                board_id, post_number, parent_number, name, text,
                image_filename, image_thumbnail, image_original_name,
                image_size_bytes, image_width, image_height, image_mime_type,
                created_at
            FROM posts
            WHERE board_id = $1 AND (post_number = $2 OR parent_number = $2)
            ORDER BY created_at ASC
            "#,
        )
        .bind(thread.board_id)
        .bind(thread.post_number)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        if rows.is_empty() {
            return Ok(None);
        }

        let mut posts: Vec<Post> = rows.into_iter().map(Post::from).collect();

        // first row should be the OP
        let op_index = posts.iter().position(|p| p.parent_number.is_none());

        match op_index {
            None => Ok(None),
            Some(i) => {
                let op = posts.remove(i);
                Ok(Some(ThreadDetail { op, replies: posts }))
            }
        }
    }

    async fn count_threads(&self, board_id: i64) -> DomainResult<u64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM posts WHERE board_id = $1 AND parent_number IS NULL",
        )
        .bind(board_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(count as u64)
    }

    async fn delete_by_board(&self, board_id: i64) -> DomainResult<()> {
        sqlx::query("DELETE FROM posts WHERE board_id = $1")
            .bind(board_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(())
    }

    async fn exists(&self, id: PostId) -> DomainResult<bool> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM posts WHERE board_id = $1 AND post_number = $2)",
        )
        .bind(id.board_id)
        .bind(id.post_number)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(exists)
    }

    async fn is_thread(&self, id: PostId) -> DomainResult<bool> {
        let is_thread = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM posts
                WHERE board_id = $1 AND post_number = $2 AND parent_number IS NULL
            )
            "#,
        )
        .bind(id.board_id)
        .bind(id.post_number)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Database(e.to_string()))?;

        Ok(is_thread)
    }
}

fn assemble_thread_previews(rows: Vec<ThreadPreviewRow>) -> Vec<ThreadPreview> {
    use std::collections::BTreeMap;

    // BTreeMap preserves insertion order which we rely on since the query orders by bumped_at
    let mut threads: BTreeMap<i64, (ThreadPreviewRow, Vec<Post>, i64, DateTime<Utc>)> =
        BTreeMap::new();

    // track op order since BTreeMap sorts by key (post_number) not bump time
    let mut op_order: Vec<i64> = Vec::new();

    for row in rows {
        if row.parent_number.is_none() {
            // this is an OP row
            let op_number = row.post_number;
            let reply_count = row.reply_count;
            let bumped_at = row.bumped_at;
            op_order.push(op_number);
            threads.insert(op_number, (row, Vec::new(), reply_count, bumped_at));
        } else {
            // this is a preview reply - attach to its OP
            let parent = row.parent_number.unwrap();
            if let Some((_, replies, _, _)) = threads.get_mut(&parent) {
                // rows come back newest-first from window func, re-sort to asc after collecting
                let post = post_from_preview_row(row);
                replies.push(post);
            }
        }
    }

    op_order
        .into_iter()
        .filter_map(|op_number| {
            threads.remove(&op_number).map(|(op_row, mut replies, reply_count, bumped_at)| {
                let omitted_count = (reply_count - replies.len() as i64).max(0);
                let op = post_from_preview_row(op_row);

                ThreadPreview {
                    op,
                    reply_count,
                    omitted_count,
                    last_replies: replies,
                    bumped_at,
                }
            })
        })
        .collect()
}

fn post_from_preview_row(row: ThreadPreviewRow) -> Post {
    let image = match (
        row.image_filename,
        row.image_thumbnail,
        row.image_original_name,
        row.image_mime_type,
    ) {
        (Some(filename), Some(thumbnail_filename), Some(original_name), Some(mime_type)) => {
            Some(ImageInfo {
                filename,
                thumbnail_filename,
                original_name,
                size_bytes: row.image_size_bytes.unwrap_or(0),
                width: row.image_width.unwrap_or(0),
                height: row.image_height.unwrap_or(0),
                mime_type,
            })
        }
        _ => None,
    };

    Post {
        board_id: row.board_id,
        post_number: row.post_number,
        parent_number: row.parent_number,
        name: row.name,
        text: row.text,
        image,
        created_at: row.created_at,
    }
}
