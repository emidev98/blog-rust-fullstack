#  Blog Rust Full Stack

This repository drives the developer thru the process of connecting Rust with a Database to add a REST API that allows to write and read data from the Database and in the end transform that REST API into a server side rendering web. Every step has its own independent utility but its progressive thru the process of creating a website with Rust.

1. How to connect to the Database using Rust.
2. How to create a rest API.
3. How to refact the rest API to add server side rendering.


## 1.[How to connect to the Database using Rust](https://diesel.rs/)

This is the framework that will be used to connect to the DB. Here you have a collection of examples that you can apply to your own project in case it's needed. To run the code that we'll have in the following sections you must run **cargo run** command.

### Migrations

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

### Models

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

### Insert

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

### Query

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

### Update

The following example will update the title of a post by filtering by id. Due the nature of id being an incremental serial, a unique entry from the database can be queried back and displayed without having to loop over it like in the queries examples:

```rust
let post_update_res: Post = diesel::update(posts.filter(id.eq(2)))
        .set(title.eq("Second Post"))
        .get_result(&db_conn)
        .expect("ERROR:> Update error");

println!("Display updated post {:?}",post_update_res);

```

### Delete

This following statement will delete all posts where slug ends with "-post" which means that if you have had imported as by the pattern of "first-post", "second-post", "third-post"... will delete al data from the database.

```rust
diesel::delete(posts.filter(slug.like("%-post%")))
    .execute(&db_conn)
    .expect("ERROR:> Something went wrong trying to delete registers");
```

**[Commit 1b5394e contains the code written until this point](https://github.com/emidev98/blog-rust-fullstack/commit/1b5394e1f6546a1fbde35500015d6a9a4e2da10a)**

## 2.[How to create a rest API](https://actix.rs/docs/) 

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

### Server Models

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

### Server Code

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

**[Commit d5d8685 contains the code written until this point](https://github.com/emidev98/blog-rust-fullstack/commit/d5d86850f39c9aa4f5e4ff746a97fbe397d9b085)**

## 3.[How to refact the rest API to add server side rendering](https://tera.netlify.app/)

Tera is the way to manage the templates in the server side. Which means that the GET requests will request an HTML template that will be generated in the server side and served to you for the browser just to render the data in the frontend. Until now I have focus my efforst on trying ot build a micro service. Then use a modern frontend framework to request the data to the backend but in the last moment I have decided to do server-side-rendering. 

To be honest this is not the best technic because you have to give up some of the strengths of both world since from a backend perspective is better to use the resources to comput data and from a frontend perspective is better to use the resources to compute the template and manage the different state changes to request some more data to the backend when needed. (mistakes aside, lets continue). I have changed the project path from **/blog** to **/** since all files will have the same scope of computation and it will not be divided by backend and frontend.

### Templates

We will store all templates into the **/templates** folder with each of the files having a very specific scope:
- index.html: is the one served to the users when they land on the website,
- post.html: have the content of each post without words limitation,
- new-post.html: allows the user to submit new posts.

> /templates/index.html
```html
<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta http-equiv="X-UA-Compatible" content="IE=edge">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Blog</title>
        <style>
            body {
                margin: 0;
                color: #dbdbdb;
                background-color: #1c1c1c;
            }

            h1 {
                margin: 0;
            }

            a {
                text-decoration: none;
                color: inherit; 
            }

            #Header {
                display: flex;
                align-items: center;
                justify-content: space-between;
                padding: 1em;
                background-color: #212121;
                border-bottom: 1px #dbdbdb solid;
            }

            #Header .HeaderLogo a{
                display: flex;
            }
            
            #Header .HeaderLogo img {
                filter: invert(0.9);
                width: 32px;
                height: 32px;
                margin-right: 1em;
            }

            #PostsWrapper {
                margin: 1em 2em 0 2em;
            }

            .PostContent {
                background-color: #262627;
                padding: 1em;
                border-radius: 0.25em;
                border: 1px gray solid;
                margin-bottom: 2em;

            }

            .PostHeader {
                display: flex;
                justify-content: space-between;
            }
        </style>
    </head>
    <body>
        <header id="Header">
            <div class="HeaderLogo">
                <a href="/">
                    <img src="https://upload.wikimedia.org/wikipedia/commons/d/d5/Rust_programming_language_black_logo.svg">
                    <h1>Blog</h1>
                </a>
            </div>

            <div class="HeaderActions">
                <a href="/posts/new">Create new post</a>
            </div>
        </header>

        <div id="PostsWrapper">
            {% for post in posts %}
                <div class="PostContent">
                    <div class="PostHeader">
                        <h2>{{post.title}}</h2>
                        <a href="/posts/{{post.slug}}">View full post</a>
                    </div>
                    <p>{{post.body | truncate(lenght=10)}}</p>
                </div>
            {% endfor %}
        </div>
    </body>
</html>
```


> /templates/post.html
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Blog | {{post.title}}</title>
    <style>
        body {
            margin: 0;
            color: #dbdbdb;
            background-color: #1c1c1c;
        }

        h1 {
            margin: 0;
        }

        a {
            text-decoration: none;
            color: inherit; 
        }

        #Header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 1em;
            background-color: #212121;
            border-bottom: 1px #dbdbdb solid;
        }

        #Header .HeaderLogo a{
            display: flex;
        }
        
        #Header .HeaderLogo img {
            filter: invert(0.9);
            width: 32px;
            height: 32px;
            margin-right: 1em;
        }

        #PostWrapper {
            margin: 1em 4em 0 4em;
        }

        .PostContent {
            background-color: #262627;
            padding: 1em;
            border-radius: 0.25em;
            border: 1px gray solid;
            margin-bottom: 2em;

        }

        .PostHeader {
            display: flex;
            justify-content: space-between;
        }
    </style>
</head>
    <body>
        <header id="Header">
            <div class="HeaderLogo">
                <a href="/">
                    <img src="https://upload.wikimedia.org/wikipedia/commons/d/d5/Rust_programming_language_black_logo.svg">
                    <h1>Blog | {{post.title}}</h1>
                </a>
            </div>
            <div class="HeaderActions">
                <a href="/">Back to posts list</a>
            </div>
        </header>

        
        <div id="PostWrapper">
            <div class="PostContent">
                <h2>{{post.title}}</h2>
                <p>{{post.body}}</p>
            </div>
        </div>
    </body>
</html>
```

At this point you can observe that the new-post contains native HTML/JS/CSS code to interact with the server from client side to submit the data that the user will input there are no validation checks since it is no necessary (we all know the pain to do all these checks in plain JS).
> /template/new-post.html
```html
<!DOCTYPE html>
<html lang="en">

<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Blog | New Post</title>
    <style>
        body {
            margin: 0;
            color: #dbdbdb;
            background-color: #1c1c1c;
        }

        h1 {
            margin: 0;
        }

        a {
            text-decoration: none;
            color: inherit;
        }

        #Header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 1em;
            background-color: #212121;
            border-bottom: 1px #dbdbdb solid;
        }

        #Header .HeaderLogo a{
            display: flex;
        }

        #Header .HeaderLogo img {
            filter: invert(0.9);
            width: 32px;
            height: 32px;
            margin-right: 1em;
        }

        #PostWrapper {
            display: flex;
            justify-content: center;
            margin: 1em 4em 0 4em;
            
        }

        .PostContent {
            display: flex;
            flex-direction: column;
            background-color: #262627;
            padding: 1em;
            border-radius: 0.25em;
            border: 1px gray solid;
            min-width: 280px;
            overflow: auto;
        }

        .PostContent textarea {
            margin-bottom: 1em;
        }

        .PostHeader {
            display: flex;
            justify-content: space-between;
        }
    </style>
</head>

<body>
    <header id="Header">
        <div class="HeaderLogo">
            <a href="/">
                <img src="https://upload.wikimedia.org/wikipedia/commons/d/d5/Rust_programming_language_black_logo.svg">
                <h1>Blog | New Post</h1>
            </a>
        </div>
        <div class="HeaderActions">
            <a href="/">Back to posts list</a>
        </div>
    </header>


    <div id="PostWrapper">
        <div class="PostContent">
            <h3>Title</h3>
            <input id="Title" type="text" />
            <h3>Post</h3>
            <textarea id="Body"></textarea>
            <button id="AddPost">Create Post</button>
        </div>
    </div>
</body>

<script>
    window.onload = () => {
        const onSendNewPost = () => {
            const title = document.getElementById("Title").value;
            const body = document.getElementById("Body").value;
            var http = new XMLHttpRequest();
            http.open('POST', "/posts/new", true);

            http.setRequestHeader('Content-type', 'application/json');

            http.onreadystatechange = function () {
                if (http.readyState == 4 && http.status == 200) {
                    window.location = '/posts/' + JSON.parse(http.response).slug;
                }
            }

            http.send(JSON.stringify({ body, title }));
        };

        document.getElementById("AddPost")
            .addEventListener("click", onSendNewPost);
    };
</script>

</html>
```

### Server rendering

As explained previously the decision of doing server-side-rendering will also force us to change the entry points which combined with Tera template engine will allow to render the data in the server side:

- **GET:/** send to the frontend the **index.html** with the list of posts rendered,
- **GET:/posts/{slug_id}** send to the frontend the **post.html** with the full expanded details of each post,
- **GET:/posts/new** send to the frontend the **new-post.html** with the inputs and logic from the frontend to persist a new post in the databse,
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

The main takes from the decision of implementing server-side-rendering are:
- **app_data** function from **actrix_web::App** allow to inject new modules very easy to transform the data in an elegant way.
- if an API is well designed shuld be easy to modify the **GET** entry points in order to do server side rendering.
- the last important point here as personal opinion I think server-side rendering shouldn't happen since it can take computation time for other relevant tasks and since clients are that powerful now in days feels like is not really needed to have server-side-rendering.

```rust
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

#[get("/posts/{slug_id}")]
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

#[get("/posts/new")]
async fn new_post_view(template_manager: web::Data<tera::Tera>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(template_manager.render("new-post.html", &tera::Context::new()).unwrap())
}

#[post("/posts/new")]
async fn new_post_create(pool: web::Data<DbPool>, data: web::Json<NewPostHandler>) -> impl Responder {
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
            .service(new_post_view)
            .service(new_post_create)
            .service(post_view)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(tera))

    })
    .bind(("0.0.0.0", 9900))?.run().await
}
```