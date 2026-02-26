use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

use backend::domain::DomainError;
use backend::repositories::postgres::PostgresAdminRepository;
use backend::repositories::AdminRepository;

/// Creates a test database container and returns a connected pool
async fn setup_test_db() -> (PgPool, testcontainers::ContainerAsync<Postgres>) {
    let container = Postgres::default()
        .start()
        .await
        .expect("Failed to start postgres container");

    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let connection_string = format!(
        "postgres://postgres:postgres@127.0.0.1:{}/postgres",
        port
    );

    let pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    (pool, container)
}

#[tokio::test]
async fn test_create_admin() {
    let (pool, _container) = setup_test_db().await;
    let repo = PostgresAdminRepository::new(pool);

    let admin = repo.create("testuser", "hashed_password_123").await.unwrap();

    assert_eq!(admin.username, "testuser");
    assert_eq!(admin.password_hash, "hashed_password_123");
    assert!(admin.id > 0);
}

#[tokio::test]
async fn test_find_by_username() {
    let (pool, _container) = setup_test_db().await;
    let repo = PostgresAdminRepository::new(pool);

    // Create first
    repo.create("findme", "password").await.unwrap();

    // Then find
    let found = repo.find_by_username("findme").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().username, "findme");

    // Not found case
    let not_found = repo.find_by_username("nonexistent").await.unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_find_by_id() {
    let (pool, _container) = setup_test_db().await;
    let repo = PostgresAdminRepository::new(pool);

    let created = repo.create("byiduser", "password").await.unwrap();

    let found = repo.find_by_id(created.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, created.id);

    // Not found
    let not_found = repo.find_by_id(99999).await.unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_create_duplicate_username_fails() {
    let (pool, _container) = setup_test_db().await;
    let repo = PostgresAdminRepository::new(pool);

    repo.create("duplicate", "pass1").await.unwrap();
    
    let result = repo.create("duplicate", "pass2").await;
    
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, backend::domain::DomainError::AlreadyExists(_)));
}

#[tokio::test]
async fn test_change_password_success() {
    let (pool, _container) = setup_test_db().await;
    let repo = PostgresAdminRepository::new(pool);

    let admin = repo.create("passuser", "old_hash").await.unwrap();

    // Change password
    repo.change_password(admin.id, "new_hash").await.unwrap();

    // Verify it changed
    let updated = repo.find_by_id(admin.id).await.unwrap().unwrap();
    assert_eq!(updated.password_hash, "new_hash");
}

#[tokio::test]
async fn test_change_password_not_found() {
    let (pool, _container) = setup_test_db().await;
    let repo = PostgresAdminRepository::new(pool);

    let result = repo.change_password(99999, "new_hash").await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DomainError::AdminNotFound(_)));
}

#[tokio::test]
async fn test_exists_any_empty() {
    let (pool, _container) = setup_test_db().await;
    let repo = PostgresAdminRepository::new(pool);

    let exists = repo.exists_any().await.unwrap();
    assert!(!exists);
}

#[tokio::test]
async fn test_exists_any_with_admin() {
    let (pool, _container) = setup_test_db().await;
    let repo = PostgresAdminRepository::new(pool);

    repo.create("someone", "hash").await.unwrap();

    let exists = repo.exists_any().await.unwrap();
    assert!(exists);
}
