use actix_web::{web::{self, Data}, App, HttpResponse, HttpServer, Responder};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{SqlitePool, FromRow};

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

struct AppState {
    db: SqlitePool,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let connection_pool: SqlitePool = SqlitePool::connect("sqlite://sqlite.db")
        .await
        .expect("Couldn't create sqlitepool");

    let app_state = Data::new(AppState { db: connection_pool });

    HttpServer::new(move || {
        App::new()
            .route("/hey", web::get().to(manual_hello))
            .route("/register", web::post().to(register))
            .route("/tmp", web::post().to(create_user_tmp))
            // .app_data(db_data.clone())
            .app_data(app_state.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn create_user_tmp(
    app_state: Data<AppState>,
    ) -> impl Responder {
    let utc_now = Utc::now();
    println!("{}", utc_now);
    sqlx::query!(
        r#"INSERT INTO users
        (username, password, created_at)
        VALUES ($1, $2, $3)"#,
        "someusername",
        "somepassword",
        utc_now,
        )
        .execute(&app_state.db)
        .await
        .expect("Failed to create tmpuser");

    return HttpResponse::Ok().json("");
}

async fn register(
    registration_request: web::Json<RegistrationRequest>, 
    app_state: Data<AppState>,
    ) -> impl Responder {

    let username = registration_request.username.clone();
    let query_as_result = sqlx::query_as!(UserModel,
        r#"SELECT 
        id,
        username,
        password,
        created_at,
        updated_at
        FROM Users 
        WHERE username = $1"#, 
        username)
        .fetch_all(&app_state.db)
        .await;

    if query_as_result.is_err() {
        println!("{:?}", query_as_result.err());
        let message = "Something exploded";
        return HttpResponse::InternalServerError().json(json!({"status": "error", "message": message}));
    }
    
    let users = query_as_result.unwrap();

    println!("query_as_result: {:?}", users);
    return HttpResponse::Ok().json(users);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub username: String,
    pub password: String,
}

#[derive(FromRow, Debug, Deserialize, Serialize)]
pub struct UserModel {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}
