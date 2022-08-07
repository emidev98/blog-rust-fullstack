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


#[derive(Insertable)]
#[table_name="posts"]
pub struct NewPost<'a> {
    pub title: &'a str,
    pub body: &'a str,
    pub slug: &'a str
}