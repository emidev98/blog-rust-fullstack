#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use self::schema::posts::dsl::*;
use self::models::{Post, NewPostHandler};

use dotenv::dotenv;
use std::env;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use diesel::r2d2::Pool;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[get("/")]
async fn healthcheck() -> impl Responder {
    HttpResponse::Ok().body("healthcheck")
}

#[get("/posts")]
async fn get_posts(pool: web::Data<DbPool>) -> impl Responder {
    let db_conn = pool
        .get()
        .expect("ERROR:> Cannot connect to the database");

    match web::block(move || {posts.load::<Post>(&db_conn)}).await {
        Ok(data) => HttpResponse::Ok().json(data.unwrap()),
        Err(_err) => HttpResponse::Ok().body("Error")
    }
}

#[post("/posts/new")]
async fn post_posts(pool: web::Data<DbPool>, data: web::Json<NewPostHandler>) -> impl Responder {
    let db_conn = pool
        .get()
        .expect("ERROR:> Cannot connect to the database");

    match web::block(move || {Post::create_post(&db_conn, &data)}).await {
        Ok(data) => HttpResponse::Ok().json(data.unwrap()),
        Err(_err) => HttpResponse::Ok().body("Error")
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL")
        .expect("ERROR:> DABASE URL NOT FUND IN .env FILE");
    
    let connection = ConnectionManager::<PgConnection>::new(db_url);

    let pool = Pool::builder()
        .build(connection)
        .expect("ERROR:> Creating DB connection");
    
    HttpServer::new(move || {
        App::new()
            .service(healthcheck)
            .service(get_posts)
            .service(post_posts)
            .app_data(web::Data::new(pool.clone()))
    })
    .bind(("0.0.0.0", 9900))?.run().await
}