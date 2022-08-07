# [Diesel ORM (Object–relational mapping)](https://en.wikipedia.org/wiki/Object%E2%80%93relational_mapping)

This is the framework that will be used to connect to the DB. Here you have a collection of examples that you can apply to your own project in case it's needed. To run the code that we'll have in the following sections you must run **cargo run** command.

## Migrations

Migrations are useful too keep track of all changes on the DB done from outside the (micro)-service. Modern ORM's have the migrations integrated into the framework. We will define the following migrations:

> blog/migrations/2022-07-07-0_create_posts/up.sql
```SQL
CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    slug VARCHAR NOT NULL,
    body TEXT NOT NULL
)
```

> blog/migrations/2022-07-07-0_create_posts/down.sql
```SQL
DROP TABLE posts;
```

With the following command you can execute the *up.sql* migrations to progressively modify the DB forward:
```
diesel migration run
```

This following command will do the opposite of the previous one which means that will execute the *down.sql* with the concept of re-doing what the previous command created.
```
diesel migration redo
```

## Models

If you executed the previous commands you may see that inside **blog/srrc/schemas.rs** you may have a macro created with posts name. That will be useful to do requests to the DB. Remebebr that you need to have a connection to the DB that will be stored into the **blog/.env** file with the nam DATABASE_URL.

Create the following models. They will allow you to interact with the databse from Rust native without having to know much about SQL:

> blog/src/models.rs
```rust
#[derive(Queryable, Debug)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub body: String
}

#[derive(Queryable, Debug)]
pub struct SimplifiedPost {
    pub title: String,
    pub body: String
}

use super::schema::posts;

#[derive(Insertable)]
#[table_name="posts"]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub slug: &'a str
}
```
*Checking the details you can see that there are some attributes defind ffor the data strcutres like `Queryable, Insertable and table_name="posts"`, these attributes allow you to interact with the DB using the native Rust models.

## Insert

Now that everything is configured lets insert some data to be able to play with it. If you follow the next code the most important takes is that it loads the **db_url** from the **.env** file, creates the DB connection and sends the connection to the diesel framework to insert the data:

> blog/src/main.rs
```rust
#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use diesel::associations::HasTable;
use dotenv::dotenv;
use std::env;

use diesel::prelude::*;
use diesel::pg::PgConnection;

fn main() {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL")
        .expect("ERROR:> DABASE URL NOT FUND IN .env FILE");

    let db_conn = PgConnection::establish(&db_url)
        .expect("ERROR:> Cannot connect to the databse");

    use self::schema::posts::dsl::*;
    use self::models::{Post, NewPost, SimplifiedPost};
    
    let new_post = NewPost {
        title: "First post",
        body: "Lorem ipsum",
        slug: "first-post"
    };

    diesel::insert_into(posts::table)
        .values(new_post)
        .get_result::<Post>(&db_conn)
        .expect("ERROR:> Failed to insert data into DB");
}
```
*Insert multiple rows that way you have a larger dataset that you can interact with.

## Query

The following examples asumes that everything is correctly imported into main.rs. To use this pieces of code place them in main.rs below the insert.

```rs
// SELECT * FROM posts
let posts_res = posts
    .load::<Post>(&db_conn)
    .expect("ERROR:> Posts cannot be retreived from DB");

// Display all posts into debug mode
for post in posts_res {
    println!("{:?}", post);
}
```

```rs
// SELECT * FROM posts LIMIT 1
let post_res = posts
    .limit(1)
    .load::<Post>(&db_conn)
    .expect("ERROR:> Posts cannot be retreived from DB");

for post in post_res {
    println!("{:?}", post);
}
```

```rs
/*
    SELECT * 
        FROM posts 
        ORDER BY id DESC
        LIMIT 1
*/
let post_with_order_res = posts
    .order(id.desc())
    .limit(1)
    .load::<Post>(&db_conn)
    .expect("ERROR:> Posts cannot be retreived from DB");

for post in post_with_order_res {
    println!("{:?}", post);
}
```

```rs
// SELECT * FROM posts WHERE slug = 'second-post'
let posts_with_where_res = posts
    .filter(slug.eq("second-post"))
    .load::<Post>(&db_conn)
    .expect("ERROR:> Posts cannot be retreived from DB");

for post in posts_with_where_res {
    println!("{:?}", post);
}
```

```rs
// SELECT title, body FROM posts
let simplified_posts_res = posts
    .select((title, body))
    .load::<SimplifiedPost>(&db_conn)
    .expect("ERROR:> Posts cannot be retreived from DB");

for post in simplified_posts_res {
    println!("{:?}", post);
}
```

*As you can observe from the previous examples is very similar to SQL without the necessity to write SQL code since the OEM auogenerates the queries.

## Update

The following example will update the title of a post by filtering by id. Due the nature of id being an incremental serial, a unique entry from the database can be queried back and displayed without having to loop over it like in the queries examples:

```rust
let post_update_res: Post = diesel::update(posts.filter(id.eq(2)))
        .set(title.eq("Second Post"))
        .get_result(&db_conn)
        .expect("ERROR:> Update error");

println!("Display updated post {:?}",post_update_res);

```

## Delete

This following statement will delete all posts where slug ends with "-post" which means that if you have had imported as by the pattern of "first-post", "second-post", "third-post"... will delete al data from the database.

```rust
diesel::delete(posts.filter(slug.like("%-post%")))
    .execute(&db_conn)
    .expect("ERROR:> Something went wrong trying to delete registers");
```

**[Commit 1b5394e contains the code written until this point](https://github.com/emidev98/blog-rust-fullstack/commit/1b5394e1f6546a1fbde35500015d6a9a4e2da10a)**

# [Actix Web Server](https://actix.rs/docs/) 

Since one of the tecniques to build backend software is by creating micro-service architectures based on REST APIs. We'll follow the same approach by using Actrix to define the following entry points:

- **GET:/** used to do a healthcheck to validate if the server is up or no.

- **GET:/posts** return all posts stored into the DB

- **POST:/posts/new** create a new post by sending a title and body.
> Request
```JSON
{
    "title": String,
    "body": String
}
```

> Response
```JSON
{
    "id": Number,
    "title": String,
    "body": String,
    "slug": String
}
```

## Server Models

One of the important things about the server is that we'll have the models that will be serializables which means that the server will receive data and will know how to transform that data to our expected model that furthermore we'll store that model into the DB.

As you can see here, we will add a implementation for the data structure Post to create the slug property and another method that will allow us to create posts on a very easy way by sending only the db connection with the NewPostHandler. Finally the last detail you can see that a Serialize and Deserialize attribute has been added to the structures that way will be transformed automatically to JSON in our example since it's a JSON Rest API by using serde.

> blog/src/models.rs
```Rust
use diesel::PgConnection;
use serde::{Serialize, Deserialize};
use super::schema::posts;
use diesel::prelude::*;

#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub body: String
}

impl Post {
    pub fn slugify(title: &str) -> String {
        title.replace(" ", "-").to_lowercase()
    }

    pub fn create_post<'a> (
        db_conn: &PgConnection, 
        post: &NewPostHandler
    ) -> Result<Post, diesel::result::Error> {
        let slug = Post::slugify(&post.title);
    
        let new_post = NewPost {
            title: &post.title,
            body: &post.body,
            slug: &slug
        };

        diesel::insert_into(posts::table).values(new_post).get_result::<Post>(db_conn)
    }
}

#[derive(Queryable, Debug, Serialize, Deserialize)]
pub struct NewPostHandler {
    pub title: String,
    pub body: String
}


#[derive(Queryable, Debug, Serialize, Deserialize)]
#[derive(Insertable)]
#[table_name="posts"]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub slug: &'a str
}
```

## Server Code

The main takes from server side are:
- instead of having an unique connection to the DB we create a pool of connections that way the server itself can manage and optimize the database connections instead of doing it manually per each request,
- each of the rutes architected previously are very easy defined by using attributes on each method with its own logic,
- the main entry point of out application is defined with **#[actix_web::main]** attribute adding the routes to our Actrix application by instantiating a new App.

To start the server at this point you will realize that the commannd **cargo run** will never stop unless you stop it since will be always listening for any request to the port 9900.

> blog/src/main.rs
```Rust
#[macro_use]
extern crate diesel;

pub mod models;
pub mod schema;

use self::schema::posts;
use self::schema::posts::dsl::*;
use self::models::{Post, NewPost, NewPostHandler};

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
```