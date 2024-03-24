#![allow(unused_variables)]
use geekorm::prelude::*;
use geekorm::{ForeignKey, GeekTable, PrimaryKey};

#[derive(Debug, GeekTable)]
pub struct User {
    pub id: PrimaryKey,
    pub name: String,
}

#[derive(Debug, GeekTable)]
pub struct Post {
    pub id: PrimaryKey,
    pub title: String,
    pub author_id: ForeignKey<User>,
}

fn main() {
    let user_posts = User::select()
        .columns(vec!["User.name", "Post.title"])
        .join(Post::table())
        .order_by("name", geekorm::QueryOrder::Asc)
        .build()
        .expect("Failed to build query");

    // This will print and test the query that will be executed
    println!("User posts query: {:?}", user_posts);
    assert_eq!(
        user_posts.query.as_str(),
        "SELECT User.name, Post.title FROM User JOIN Post ON User.id = Post.author_id ORDER BY name ASC");
}
