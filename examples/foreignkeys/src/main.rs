#![allow(unused_variables, unused_imports)]
use geekorm::prelude::*;
use geekorm::{ForeignKey, GeekTable, PrimaryKeyInteger};

#[derive(Debug, Clone, Default, GeekTable)]
pub struct Users {
    pub id: PrimaryKeyInteger,
    pub name: String,
}

#[derive(Debug, Clone, Default, GeekTable)]
pub struct Posts {
    pub id: PrimaryKeyInteger,
    pub title: String,
    #[geekorm(foreign_key = "Users.id")]
    pub author: ForeignKey<i32, Users>,
}

fn main() {
    let user = Users::new(String::from("GeekMasher"));
    println!("User: {:?}", user);

    let post1 = Posts::new(String::from("How I started programming in Rust"), user.id);
    println!("Post1: {:?}", post1);
    let post2 = Posts::new(String::from("Why I love Rust"), user.id);
    println!("Post2: {:?}", post2);

    // Select all posts by a user
    let posts_by_user = Posts::select()
        .columns(vec!["Posts.title", "Users.name"])
        .join(Users::table())
        .build()
        .expect("Failed to build query");

    println!("Posts by user query: {:?}", posts_by_user.query);
    assert_eq!(
        posts_by_user.query.as_str(),
        "SELECT (Posts.title, Users.name) FROM Posts INNER JOIN Users ON Users.id = Posts.author_id;"
    );

    // // Select all users and their posts
    // let user_posts = Users::select()
    //     .columns(vec!["Users.name", "Posts.title"])
    //     .join(Posts::table())
    //     .order_by("name", geekorm::QueryOrder::Asc)
    //     .build()
    //     .expect("Failed to build query");
    //
    // println!("User posts query: {:?}", user_posts);
    // assert_eq!(
    //     user_posts.query.as_str(),
    //     "SELECT (Users.name, Posts.title) FROM Users INNER JOIN Posts ON Users.id = Posts.author_id ORDER BY name ASC;"
    // );
}
