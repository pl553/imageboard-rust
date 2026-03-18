use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

use backend::domain::{CreateBoard, DomainError};
use backend::repositories::postgres::PostgresBoardRepository;
use backend::repositories::BoardRepository;

async fn setup_test_db() -> (PgPool, testcontainers::ContainerAsync<Postgres>) {
    let container = Postgres::default()
        .start()
        .await
        .expect("failed to start postgres container");

    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

    let pool = PgPool::connect(&url)
        .await
        .expect("failed to connect");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations");

    (pool, container)
}

fn make_board(slug: &str) -> CreateBoard {
    CreateBoard {
        slug: slug.to_string(),
        name: format!("/{}/", slug),
        description: None,
    }
}

fn make_board_with_desc(slug: &str, desc: &str) -> CreateBoard {
    CreateBoard {
        slug: slug.to_string(),
        name: format!("/{}/", slug),
        description: Some(desc.to_string()),
    }
}

#[tokio::test]
async fn test_create_board() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool);

    let board = repo.create(make_board("g")).await.unwrap();

    assert_eq!(board.slug, "g");
    assert!(board.id > 0);
    assert_eq!(board.thread_count, 0);
}

#[tokio::test]
async fn test_create_board_with_description() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool);

    let board = repo
        .create(make_board_with_desc("tv", "television & film"))
        .await
        .unwrap();

    assert_eq!(board.description.unwrap(), "television & film");
}

#[tokio::test]
async fn test_create_duplicate_slug_fails() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool);

    repo.create(make_board("v")).await.unwrap();
    let err = repo.create(make_board("v")).await.unwrap_err();

    assert!(matches!(err, DomainError::AlreadyExists(_)));
}

#[tokio::test]
async fn test_find_all_empty() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool);

    let boards = repo.find_all().await.unwrap();
    assert!(boards.is_empty());
}

#[tokio::test]
async fn test_find_all_returns_all_boards() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool);

    repo.create(make_board("a")).await.unwrap();
    repo.create(make_board("b")).await.unwrap();
    repo.create(make_board("c")).await.unwrap();

    let boards = repo.find_all().await.unwrap();
    assert_eq!(boards.len(), 3);
}

#[tokio::test]
async fn test_find_by_slug_found() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool);

    repo.create(make_board("an")).await.unwrap();
    let board = repo.find_by_slug("an").await.unwrap();

    assert!(board.is_some());
    assert_eq!(board.unwrap().slug, "an");
}

#[tokio::test]
async fn test_find_by_slug_not_found() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool);

    let board = repo.find_by_slug("zzz").await.unwrap();
    assert!(board.is_none());
}

#[tokio::test]
async fn test_find_by_id_found() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool);

    let created = repo.create(make_board("sp")).await.unwrap();
    let found = repo.find_by_id(created.id).await.unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().id, created.id);
}

#[tokio::test]
async fn test_find_by_id_not_found() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool);

    let found = repo.find_by_id(99999).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_delete_by_slug_existing() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool);

    repo.create(make_board("del")).await.unwrap();
    let deleted = repo.delete_by_slug("del").await.unwrap();

    assert!(deleted);
    assert!(repo.find_by_slug("del").await.unwrap().is_none());
}

#[tokio::test]
async fn test_delete_by_slug_nonexistent() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool);

    let deleted = repo.delete_by_slug("nope").await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_exists_true() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool);

    repo.create(make_board("ex")).await.unwrap();
    assert!(repo.exists("ex").await.unwrap());
}

#[tokio::test]
async fn test_exists_false() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool);

    assert!(!repo.exists("nope").await.unwrap());
}

#[tokio::test]
async fn test_thread_count_reflects_posts() {
    let (pool, _c) = setup_test_db().await;
    let repo = PostgresBoardRepository::new(pool.clone());

    let board = repo.create(make_board("f")).await.unwrap();
    assert_eq!(board.thread_count, 0);

    // insert a thread (OP post) directly - parent_number NULL means it's a thread
    sqlx::query(
        "INSERT INTO posts (board_id, name, text) VALUES ($1, 'Anon', 'hello')",
    )
    .bind(board.id)
    .execute(&pool)
    .await
    .unwrap();

    let updated = repo.find_by_slug("f").await.unwrap().unwrap();
    assert_eq!(updated.thread_count, 1);
}
