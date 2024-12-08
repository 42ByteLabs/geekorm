//! # Example - Parsing Enums
//!
//! This example demonstrates how to parse enums from a database using GeekORM.

use geekorm::prelude::*;
use std::str::FromStr;

/// Enum representing the type of a project
#[derive(Data, Debug, Clone, Default, PartialEq)]
pub enum ProjectType {
    #[default]
    Library,
    #[geekorm(aliases = "App,Application")]
    Application,
    Framework,
    Tool,
    #[geekorm(key = "Programming Language")]
    Language,
}

/// Enum representing the type of a project (alternative implementation)
#[derive(Data, Debug, Clone, Default, PartialEq)]
#[geekorm(from_str = "lowercase", to_string = "lowercase")]
pub enum ProjectTypeAlt {
    #[default]
    Library,
    #[geekorm(aliases = "app,application")]
    Application,
    Framework,
    #[geekorm(aliases = "tool,toolkit")]
    Tool,
    #[geekorm(key = "Programming Language")]
    Language,
}

fn main() {
    // By default, GeekORM will automatically implement the `FromStr` trait for the enum
    let project_type = ProjectType::from_str("Library").unwrap();
    println!("Using FromStr directly: {:?}", project_type);
    assert_eq!(project_type, ProjectType::Library);

    // GeekORM will also implement the `Display` trait for the enum and use the variant name/key
    let project_type = ProjectType::Language.to_string();
    println!("Using Display: {}", project_type);
    assert_eq!(project_type, "Programming Language");

    // GeekORM will also implement the `FromStr` trait for the aliases
    let project_type = ProjectType::from_str("App").unwrap();
    println!("Using an Alias: {:?}", project_type);
    assert_eq!(project_type, ProjectType::Application);

    // GeekORM will also implement the `From` trait for various types
    // `&str`
    let project_type: ProjectType = "Framework".into();
    println!("Automatic conversion from str: {:?}", project_type);
    assert_eq!(project_type, ProjectType::Framework);

    // `String`
    let project_name: String = "Tool".to_string();
    let project_type = ProjectType::from(project_name.to_string());
    println!("From a String type: {:?}", project_type);
    assert_eq!(project_type, ProjectType::Tool);

    // `&String`
    let project_type = ProjectType::from(&project_name);
    println!("From a reference to a String: {:?}", project_type);
    assert_eq!(project_type, ProjectType::Tool);

    // The above examples will only do exact matches.
    // You can also parsing the data using a `from_str = "lowercase"`
    // attribute to parse the input in lowercase.
    let project_type = ProjectTypeAlt::from("TOOL");
    println!("From &str (lowercase): {:?}", project_type);
    assert_eq!(project_type, ProjectTypeAlt::Tool);

    // And can also convert the enum to a string using a `to_string = "lowercase"`
    // attribute to convert the output to lowercase.
    println!(
        "To string / display (lowercase): {}",
        ProjectTypeAlt::Framework
    );
    assert_eq!(ProjectTypeAlt::Framework.to_string(), "framework");
}
