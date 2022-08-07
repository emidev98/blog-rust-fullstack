#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use self::schema::posts::dsl::*;
use self::models::{Post, NewPostHandler};

use dotenv::dotenv;
use std::env;
use tera::Tera;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use diesel::r2d2::Pool;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[get("/")]
async fn index(template_manager: web::Data<tera::Tera>, pool: web::Data<DbPool>) -> impl Responder {
    let db_conn = pool
        .get()
        .expect("ERROR:> Cannot connect to the database");

    match web::block(move || {posts.load::<Post>(&db_conn)}).await {
        Ok(data) => {
            let data = data.unwrap();
            let mut ctx = tera::Context::new();
            ctx.insert("posts", &data);
            
            HttpResponse::Ok()
                .content_type("text/html")
                .body(template_manager.render("index.html", &ctx).unwrap())
        },
        Err(_err) => HttpResponse::Ok().body("Error")
    }
}

#[get("/post/{slug_id}")]
async fn post_view(
    template_manager: web::Data<tera::Tera>,
    pool: web::Data<DbPool>,
    slug_id: web::Path<String>
) -> impl Responder {
    let db_conn = pool
        .get()
        .expect("ERROR:> Cannot connect to the database");
    let url_slug = slug_id.into_inner();

    match web::block(move || {posts.filter(slug.eq(url_slug)).load::<Post>(&db_conn)}).await {
        Ok(data) => {
            let data = data.unwrap();

            if data.len() == 0 {
                return HttpResponse::NotFound().finish();
            }
            let mut ctx = tera::Context::new();
            ctx.insert("post", &data[0]);
            
            HttpResponse::Ok()
                .content_type("text/html")
                .body(template_manager.render("post.html", &ctx).unwrap())
        },
        Err(_err) => HttpResponse::Ok().body("Error")
    }
}

#[post("/posts/new")]
async fn new_post(pool: web::Data<DbPool>, data: web::Json<NewPostHandler>) -> impl Responder {
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
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();

        App::new()
            .service(index)
            .service(new_post)
            .service(post_view)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(tera))

    })
    .bind(("0.0.0.0", 9900))?.run().await
}