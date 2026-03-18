use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, warn};

mod config;
mod domain;
mod repositories;

// mod services;
// mod api;

use config::Config;
use repositories::postgres::{PostgresAdminRepository, PostgresBoardRepository, PostgresPostRepository};
use repositories::disk::DiskImageRepository;
use repositories::AdminRepository;

#[tokio::main]
async fn main() {
    // tracing for logs
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    // load config
    let config = Config::load().unwrap_or_else(|e| {
        eprintln!("failed to load config: {}", e);
        std::process::exit(1);
    });

    info!("starting imageboard backend");

    // db pool
    let pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await
        .unwrap_or_else(|e| {
            eprintln!("failed to connect to database: {}", e);
            std::process::exit(1);
        });

    info!("connected to database");

    // run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap_or_else(|e| {
            eprintln!("failed to run migrations: {}", e);
            std::process::exit(1);
        });

    info!("migrations applied");

    // repositories
    let admin_repo = Arc::new(PostgresAdminRepository::new(pool.clone()));
    let board_repo = Arc::new(PostgresBoardRepository::new(pool.clone()));
    let post_repo  = Arc::new(PostgresPostRepository::new(pool.clone()));

    let image_repo = Arc::new(
        DiskImageRepository::new(
            config.storage.images_path.clone(),
            config.storage.thumbnails_path.clone(),
        )
        .await
        .unwrap_or_else(|e| {
            eprintln!("failed to initialize image storage: {}", e);
            std::process::exit(1);
        }),
    );

    info!(
        images_path = %config.storage.images_path.display(),
        thumbnails_path = %config.storage.thumbnails_path.display(),
        "image storage ready"
    );

    // seed default admin if none exists
    seed_admin(&*admin_repo).await;

    // let services = Services::new(admin_repo, board_repo, post_repo, image_repo, &config);
    // let app = api::router::build_router(services, config.clone());
    
    // let listener = tokio::net::TcpListener::bind(config.server.addr())
    //     .await
    //     .expect("failed to bind");

    // info!(addr = %config.server.addr(), "listening");
    // axum::serve(listener, app).await.expect("server error");

    info!("repositories initialized");
}

async fn seed_admin(admin_repo: &dyn AdminRepository) {
    match admin_repo.exists_any().await {
        Ok(true) => {
            info!("admin account already exists, skipping seed");
        }
        Ok(false) => {
            let hash = bcrypt::hash("admin", bcrypt::DEFAULT_COST)
                .expect("failed to hash default password");

            match admin_repo.create("admin", &hash).await {
                Ok(admin) => {
                    warn!(
                        id = admin.id,
                        "created default admin account with password 'admin' - change this immediately"
                    );
                }
                Err(e) => {
                    eprintln!("failed to seed admin: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("failed to check for existing admins: {}", e);
            std::process::exit(1);
        }
    }
}
