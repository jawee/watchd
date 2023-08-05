use actix_web::{web::{self, Data}, App, HttpResponse, HttpServer, Responder};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{SqlitePool, FromRow, error::ErrorKind, Database};

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
            .route("/register", web::post().to(register))
            .route("/users", web::get().to(get_users))
            // .app_data(db_data.clone())
            .app_data(app_state.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn get_users(
    app_state: Data<AppState>,
    ) -> impl Responder {

    let query_as_result = sqlx::query_as!(UserModel,
        r#"SELECT 
        id,
        username,
        password,
        created_at,
        updated_at
        FROM Users 
        "#
        )
        .fetch_all(&app_state.db)
        .await;

    if query_as_result.is_err() {
        let error = query_as_result.err().unwrap();
        println!("{:?}", error);
        let message = "Something exploded";
        return HttpResponse::InternalServerError().json(json!({"status": "error", "message": message}));
    }
    
    let users = query_as_result.unwrap();

    println!("query_as_result: {:?}", users);
    return HttpResponse::Ok().json(users);
}

async fn register(
    registration_request: web::Json<RegistrationRequest>, 
    app_state: Data<AppState>,
    ) -> impl Responder {

    let utc_now = Utc::now();
    let query = sqlx::query!(
        r#"INSERT INTO users
        (username, password, created_at)
        VALUES (?, ?, ?)"#,
        registration_request.username,
        registration_request.password,
        utc_now,
        )
        .execute(&app_state.db)
        .await
        .map_err(|e|
                 {
                     println!("{:?}", e);
                     match e {
                         sqlx::Error::Database(dbe) => { 
                             let res = match dbe.is_unique_violation() {
                                 true => "Unique constraint",
                                 _ => "Unknown constraint"
                             };
                             println!("{}", res);
                             res
                         },
                         _ => {
                             println!("something bullshit");
                             "Unknown"
                         }
                     }
                 });
    if query.is_err() {
        //TODO: implement error handling
        let err = query.err();
        println!("{:?}", err);
        return HttpResponse::InternalServerError().json(json!({"status": "error", "message": err} ));
    }

    return HttpResponse::Ok().json("success");
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
