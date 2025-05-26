use std::net::{Ipv4Addr, SocketAddr};

use anyhow::Result;
use axum::{Router, extract::State, http::StatusCode, routing::get};
use tokio::net::TcpListener;

use sqlx::{PgPool, postgres::PgConnectOptions};

// a) データベースの接続設定を表す構造体
struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

// b) アプリケーション用のデータベース設定構造体から、
// PostgreSQLの接続用の構造体へ変換する。
impl From<DatabaseConfig> for PgConnectOptions {
    fn from(config: DatabaseConfig) -> Self {
        PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .username(&config.username)
            .password(&config.password)
            .database(&config.database)
    }
}

// c) PostgreSQL専用の接続プールを作成する関数
fn connect_database_with(cfg: DatabaseConfig) -> PgPool {
    PgPool::connect_lazy_with(cfg.into())
}

// データベースのヘルスチェックを行うハンドラ
async fn health_check_db(State(db): State<PgPool>) -> StatusCode {
    let connection_result = sqlx::query("SELECT 1").fetch_one(&db).await;
    match connection_result {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// // ハンドラの定義
// // /hello パスにアクセスしたときに呼び出されるハンドラ
// async fn hello_world() -> &'static str {
//     "Hello, World! Unko!"
// }

// ヘルスチェック用のハンドラ
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

// ヘルスチェックのテスト
#[tokio::test]
async fn test_health_check() {
    // ヘルスチェックのハンドラを呼び出して、ステータスコードが200 OKであることを確認
    let status_code = health_check().await;
    assert_eq!(status_code, StatusCode::OK);
}

#[sqlx::test]
async fn health_check_db_works(pool: sqlx::PgPool) {
    // データベースのヘルスチェックを行い、ステータスコードが200 OKであることを確認
    let status_code = health_check_db(State(pool)).await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::main]
async fn main() -> Result<()> {
    // データベース接続設定を定義する。
    let database_cfg = DatabaseConfig {
        host: "localhost".into(),
        port: 5432,
        username: "app".into(),
        password: "passwd".into(),
        database: "app".into(),
    };

    // データベース接続プールを作成する。
    let conn_pool = connect_database_with(database_cfg);

    // ルーターの作成
    // let app = Router::new().route("/hello", get(hello_world));
    let app = Router::new()
        .route("/health", get(health_check))
        // ルータにデータベースチェック用のハンドラを登録
        .route("/health/db", get(health_check_db))
        // ルータの State にデータベース接続プールを追加し、各ハンドラで使えるようにする。
        .with_state(conn_pool);

    // ソケットアドレスの作成
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);

    // TCPリスナーの作成
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on {}", addr);

    // サーバーの起動
    Ok(axum::serve(listener, app).await?)
    // // println!("Server started");
}
