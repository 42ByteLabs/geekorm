#![allow(unused_variables, unused_imports)]

use geekorm::prelude::*;

#[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Users {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKeyInteger,
    pub name: String,
}

#[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Posts {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKeyInteger,
    #[geekorm(unique)]
    pub title: String,
    #[geekorm(foreign_key = "Users.id")]
    pub author: ForeignKey<i32, Users>,
}

fn main() {
    let user = Users::new("GeekMasher");
    println!("User: {:?}", user);

    let post1 = Posts::new("How I started programming in Rust", user.id);
    println!("Post1: {:?}", post1);
    let post2 = Posts::new("Why I love Rust", user.id);
    println!("Post2: {:?}", post2);

    // Select all posts by a user
    let posts_by_user = Posts::query_select()
        .join(Users::table())
        .where_eq("Users.name", "GeekMasher")
        .build()
        .expect("Failed to build query");
    println!("Posts by user query: {:?}", posts_by_user.query);
    assert_eq!(
        posts_by_user.query.as_str(),
        "SELECT Posts.id, Posts.title, Posts.author FROM Posts INNER JOIN Users ON Users.id = Posts.author WHERE Users.name = ?;"
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
