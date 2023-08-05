use actix_web::{web::{self, Data}, App, HttpResponse, HttpServer, Responder};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{SqlitePool, FromRow};

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let connection_pool: SqlitePool = SqlitePool::connect("sqlite://sqlite.db")
        .await
        .expect("Couldn't create sqlitepool");
    let db_data = Data::new(connection_pool.clone());

    HttpServer::new(move || {
        App::new()
            .route("/hey", web::get().to(manual_hello))
            .route("/register", web::post().to(register))
            .route("/tmp", web::post().to(create_user_tmp))
            .app_data(db_data.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn create_user_tmp(
    db: web::Data<SqlitePool>,
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
        .execute(db.get_ref())
        .await
        .expect("Failed to create tmpuser");

    return HttpResponse::Ok().json("");
}

async fn register(
    registration_request: web::Json<RegistrationRequest>, 
    db: web::Data<SqlitePool>,
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
        .fetch_all(db.get_ref())
        .await;
    let query_result = sqlx::query!(
        r#"SELECT 
        id,
        username,
        password,
        created_at,
        updated_at
        FROM Users 
        WHERE username = $1"#, 
        username)
        .fetch_all(db.get_ref())
        .await;

    if query_result.is_err() {
        println!("{:?}", query_result.err());
        let message = "Something exploded";
        return HttpResponse::InternalServerError().json(json!({"status": "error", "message": message}));
    }

    if query_as_result.is_err() {
        println!("{:?}", query_result.err());
        let message = "Something exploded";
        return HttpResponse::InternalServerError().json(json!({"status": "error", "message": message}));
    }
    
    let users: Vec<UserModel> = query_result
        .unwrap()
        .iter()
        // .map(|r| UserModel::try_from(r.clone()).ok())
        .map(|r| UserModel { 
            id: r.id,
            username: r.username.clone(),
            password: r.password.clone(),
            created_at: r.created_at,
            // created_at: r.created_at as DateTime<Utc>,
            updated_at: r.updated_at,
        })
        .collect::<Vec<_>>();
    println!("query_result: {:?}", users);
    println!("query_as_result: {:?}", query_as_result.unwrap());
    return HttpResponse::Ok().json("");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationRequest {
    pub username: String,
    pub password: String,
}


// impl FromRow<'_, SqliteRow> for UserModel {
//     fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
//         Ok(UserModel { 
//             id: row.try_get("id")? as u32,
//             username: (row.try_get("username")? as String).clone(),
//             password: (row.try_get("password")? as String).clone(),
//             created_at: (row.try_get("created_at")? as NaiveDateTime).and_utc(),
//             // created_at: row.created_at as DateTime<Utc>,
//             updated_at: match (row.try_get("updated_at")? as Option<NaiveDateTime>) { Some(t) => Some(t.and_utc()), _ => None },
//         })
//     }
// }
#[derive(FromRow, Debug, Deserialize, Serialize)]
pub struct UserModel {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

// impl FromRow<Row> for UserModel {
//     fn from_row(row: &'r R) -> Result<Self, sqlx::Error> {
//         todo!()
//     }
// }

// impl TryFrom<&Record> for UserModel {
//     type Error = String;
//
//     fn try_from(value: &Record) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }
// impl TryFrom<Row> for UserModel {
//     type Error = String;
//
//     fn try_from(value: Row) -> Result<Self, Self::Error> {
//         todo!()
//     }
// }
