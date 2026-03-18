use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

use backend::domain::{DomainError, PostId};
use backend::repositories::postgres::{PostgresBoardRepository, PostgresPostRepository};
use backend::repositories::{BoardRepository, PostRepository};

async fn setup_test_db() -> (PgPool, testcontainers::ContainerAsync<Postgres>) {
    let container = Postgres::default()
        .start()
        .await
        .expect("failed to start postgres container");

    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

    let pool = PgPool::connect(&url).await.expect("failed to connect");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    (pool, container)
}

async fn setup_repos(
    pool: PgPool,
) -> (PostgresBoardRepository, PostgresPostRepository) {
    (
        PostgresBoardRepository::new(pool.clone()),
        PostgresPostRepository::new(pool),
    )
}

async fn create_test_board(board_repo: &PostgresBoardRepository, slug: &str) -> i64 {
    board_repo
        .create(backend::domain::CreateBoard {
            slug: slug.to_string(),
            name: slug.to_string(),
            description: None,
        })
        .await
        .unwrap()
        .id
}

// ---- create_thread ----

#[tokio::test]
async fn test_create_thread() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "g").await;

    let post = post_repo
        .create_thread(board_id, "Anon", "hello world", None)
        .await
        .unwrap();

    assert_eq!(post.board_id, board_id);
    assert_eq!(post.name, "Anon");
    assert_eq!(post.text, "hello world");
    assert!(post.parent_number.is_none());
    assert!(post.post_number > 0);
    assert!(post.image.is_none());
}

#[tokio::test]
async fn test_create_thread_post_number_increments() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "g").await;

    let p1 = post_repo.create_thread(board_id, "Anon", "first", None).await.unwrap();
    let p2 = post_repo.create_thread(board_id, "Anon", "second", None).await.unwrap();

    assert!(p2.post_number > p1.post_number);
}

// ---- create_reply ----

#[tokio::test]
async fn test_create_reply() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "v").await;

    let op = post_repo.create_thread(board_id, "Anon", "op post", None).await.unwrap();

    let reply = post_repo
        .create_reply(board_id, op.post_number, "Anon", "nice thread", None)
        .await
        .unwrap();

    assert_eq!(reply.parent_number, Some(op.post_number));
    assert_eq!(reply.board_id, board_id);
}

#[tokio::test]
async fn test_create_reply_to_nonexistent_thread_fails() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "v").await;

    let err = post_repo
        .create_reply(board_id, 99999, "Anon", "reply", None)
        .await
        .unwrap_err();

    assert!(matches!(err, DomainError::PostNotFound(_)));
}

// ---- find_by_id ----

#[tokio::test]
async fn test_find_by_id_found() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "an").await;

    let created = post_repo.create_thread(board_id, "Anon", "find me", None).await.unwrap();
    let id = PostId { board_id, post_number: created.post_number };

    let found = post_repo.find_by_id(id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().text, "find me");
}

#[tokio::test]
async fn test_find_by_id_not_found() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "an").await;

    let id = PostId { board_id, post_number: 99999 };
    let found = post_repo.find_by_id(id).await.unwrap();
    assert!(found.is_none());
}

// ---- delete ----

#[tokio::test]
async fn test_delete_post() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "b").await;

    let post = post_repo.create_thread(board_id, "Anon", "delete me", None).await.unwrap();
    let id = PostId { board_id, post_number: post.post_number };

    post_repo.delete(id).await.unwrap();

    assert!(post_repo.find_by_id(id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_delete_thread_cascades_to_replies() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "b").await;

    let op = post_repo.create_thread(board_id, "Anon", "op", None).await.unwrap();
    let reply = post_repo
        .create_reply(board_id, op.post_number, "Anon", "reply", None)
        .await
        .unwrap();

    let op_id = PostId { board_id, post_number: op.post_number };
    let reply_id = PostId { board_id, post_number: reply.post_number };

    post_repo.delete(op_id).await.unwrap();

    // reply should be gone too via cascade
    assert!(post_repo.find_by_id(reply_id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_post_errors() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "b").await;

    let id = PostId { board_id, post_number: 99999 };
    let err = post_repo.delete(id).await.unwrap_err();
    assert!(matches!(err, DomainError::PostNotFound(_)));
}

// ---- exists / is_thread ----

#[tokio::test]
async fn test_exists() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "f").await;

    let post = post_repo.create_thread(board_id, "Anon", "hi", None).await.unwrap();
    let id = PostId { board_id, post_number: post.post_number };

    assert!(post_repo.exists(id).await.unwrap());
    assert!(!post_repo.exists(PostId { board_id, post_number: 99999 }).await.unwrap());
}

#[tokio::test]
async fn test_is_thread() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "f").await;

    let op = post_repo.create_thread(board_id, "Anon", "op", None).await.unwrap();
    let reply = post_repo
        .create_reply(board_id, op.post_number, "Anon", "reply", None)
        .await
        .unwrap();

    let op_id = PostId { board_id, post_number: op.post_number };
    let reply_id = PostId { board_id, post_number: reply.post_number };

    assert!(post_repo.is_thread(op_id).await.unwrap());
    assert!(!post_repo.is_thread(reply_id).await.unwrap());
}

// ---- count_threads ----

#[tokio::test]
async fn test_count_threads() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "s").await;

    assert_eq!(post_repo.count_threads(board_id).await.unwrap(), 0);

    post_repo.create_thread(board_id, "Anon", "t1", None).await.unwrap();
    post_repo.create_thread(board_id, "Anon", "t2", None).await.unwrap();

    assert_eq!(post_repo.count_threads(board_id).await.unwrap(), 2);
}

// ---- delete_by_board ----

#[tokio::test]
async fn test_delete_by_board() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "del").await;

    post_repo.create_thread(board_id, "Anon", "t1", None).await.unwrap();
    post_repo.create_thread(board_id, "Anon", "t2", None).await.unwrap();

    post_repo.delete_by_board(board_id).await.unwrap();

    assert_eq!(post_repo.count_threads(board_id).await.unwrap(), 0);
}

// ---- find_thread_detail ----

#[tokio::test]
async fn test_find_thread_detail() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "tv").await;

    let op = post_repo.create_thread(board_id, "Anon", "op", None).await.unwrap();
    post_repo.create_reply(board_id, op.post_number, "Anon", "reply 1", None).await.unwrap();
    post_repo.create_reply(board_id, op.post_number, "Anon", "reply 2", None).await.unwrap();

    let thread_id = PostId { board_id, post_number: op.post_number };
    let detail = post_repo.find_thread_detail(thread_id).await.unwrap().unwrap();

    assert_eq!(detail.op.post_number, op.post_number);
    assert_eq!(detail.replies.len(), 2);
    assert_eq!(detail.replies[0].text, "reply 1");
    assert_eq!(detail.replies[1].text, "reply 2");
}

#[tokio::test]
async fn test_find_thread_detail_not_found() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "tv").await;

    let id = PostId { board_id, post_number: 99999 };
    let result = post_repo.find_thread_detail(id).await.unwrap();
    assert!(result.is_none());
}

// ---- find_thread_previews ----

#[tokio::test]
async fn test_find_thread_previews_basic() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "p").await;

    let op = post_repo.create_thread(board_id, "Anon", "thread 1", None).await.unwrap();
    post_repo.create_reply(board_id, op.post_number, "Anon", "reply 1", None).await.unwrap();
    post_repo.create_reply(board_id, op.post_number, "Anon", "reply 2", None).await.unwrap();
    post_repo.create_reply(board_id, op.post_number, "Anon", "reply 3", None).await.unwrap();

    let result = post_repo.find_thread_previews(board_id, 1, 10, 3).await.unwrap();

    assert_eq!(result.items.len(), 1);
    let preview = &result.items[0];
    assert_eq!(preview.op.post_number, op.post_number);
    assert_eq!(preview.reply_count, 3);
    assert_eq!(preview.last_replies.len(), 3);
    assert_eq!(preview.omitted_count, 0);
}

#[tokio::test]
async fn test_find_thread_previews_omitted_count() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "p").await;

    let op = post_repo.create_thread(board_id, "Anon", "thread 1", None).await.unwrap();
    for i in 0..5 {
        post_repo
            .create_reply(board_id, op.post_number, "Anon", &format!("reply {}", i), None)
            .await
            .unwrap();
    }

    // only preview 3 of the 5 replies
    let result = post_repo.find_thread_previews(board_id, 1, 10, 3).await.unwrap();
    let preview = &result.items[0];

    assert_eq!(preview.reply_count, 5);
    assert_eq!(preview.last_replies.len(), 3);
    assert_eq!(preview.omitted_count, 2);
}

#[tokio::test]
async fn test_find_thread_previews_pagination() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "p").await;

    for i in 0..5 {
        post_repo
            .create_thread(board_id, "Anon", &format!("thread {}", i), None)
            .await
            .unwrap();
    }

    let page1 = post_repo.find_thread_previews(board_id, 1, 3, 0).await.unwrap();
    let page2 = post_repo.find_thread_previews(board_id, 2, 3, 0).await.unwrap();

    assert_eq!(page1.items.len(), 3);
    assert_eq!(page2.items.len(), 2);
    assert_eq!(page1.total_items, 5);
    assert_eq!(page1.total_pages, 2);
}

#[tokio::test]
async fn test_find_thread_previews_last_replies_are_most_recent() {
    let (pool, _c) = setup_test_db().await;
    let (board_repo, post_repo) = setup_repos(pool).await;
    let board_id = create_test_board(&board_repo, "p").await;

    let op = post_repo.create_thread(board_id, "Anon", "op", None).await.unwrap();
    post_repo.create_reply(board_id, op.post_number, "Anon", "old reply", None).await.unwrap();
    post_repo.create_reply(board_id, op.post_number, "Anon", "newer reply", None).await.unwrap();
    post_repo.create_reply(board_id, op.post_number, "Anon", "newest reply", None).await.unwrap();

    // preview only 2
    let result = post_repo.find_thread_previews(board_id, 1, 10, 2).await.unwrap();
    let replies = &result.items[0].last_replies;

    assert_eq!(replies.len(), 2);
    // should be the 2 most recent, in chronological order
    assert_eq!(replies[0].text, "newer reply");
    assert_eq!(replies[1].text, "newest reply");
}
