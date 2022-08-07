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

    diesel::insert_into(posts::table())
        .values(new_post)
        .get_result::<Post>(&db_conn)
        .expect("ERROR:> Failed to insert data into DB");
    
    // Query 
    let posts_res = posts
        .load::<Post>(&db_conn)
        .expect("ERROR:> Posts cannot be retreived from DB");

    // Query with limit
    let post_res = posts
        .limit(1)
        .load::<Post>(&db_conn)
        .expect("ERROR:> Posts cannot be retreived from DB");

    // Query with limit and order
    let post_with_order_res = posts
        .order(id.desc())
        .limit(1)
        .load::<Post>(&db_conn)
        .expect("ERROR:> Posts cannot be retreived from DB");

    // Query by model
    let simplified_posts_res = posts
        .select((title, body))
        .load::<SimplifiedPost>(&db_conn)
        .expect("ERROR:> Posts cannot be retreived from DB");

    // Query with where
    let posts_with_where_res = posts
        .filter(slug.eq("second-post"))
        .load::<Post>(&db_conn)
        .expect("ERROR:> Posts cannot be retreived from DB");
        
    println!("Query");
    for post in posts_res {
        println!("{:?}", post);
    }

    println!("Query with limit");
    for post in post_res {
        println!("{:?}", post);
    }

    println!("Query with limit and order");
    for post in post_with_order_res {
        println!("{:?}", post);
    }

    println!("Query simplified posts");
    for post in simplified_posts_res {
        println!("{:?}", post);
    }

    println!("Query posts with where");
    for post in posts_with_where_res {
        println!("{:?}", post);
    }

    let post_update_res: Post = diesel::update(posts.filter(id.eq(2)))
        .set(title.eq("Second Post"))
        .get_result(&db_conn)
        .expect("ERROR:> Update error");


    println!("Display updated post {:?}",post_update_res);

    diesel::delete(posts.filter(slug.like("%-post%")))
        .execute(&db_conn)
        .expect("ERROR:> Something went wrong trying to delete registers");
}