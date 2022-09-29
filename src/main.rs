#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
extern crate git2;
extern crate dirs;
extern crate lazy_static;
use std::io::{Read, Write};

use lazy_static::lazy_static;

lazy_static!{
    static ref CONFIG:Config = retrieve_config();
}

fn main() {
    //making sure lazy_static code runs immediately before rocket starts spawning worker threads
    let _ = CONFIG.repos;
    rocket::ignite().mount("/",routes![ping,create_repo,change_description,read_description]).launch();
}


#[derive(serde::Serialize,serde::Deserialize)]
struct Config{
    repos:std::path::PathBuf,
}

/// function that retrieves the config
fn retrieve_config<'a>() -> Config{
    let mut path = dirs::config_dir().expect("No config directory");
    path.push(std::option_env!("CARGO_PKG_NAME").unwrap());
    path.push("config.toml");
    let mut buf = String::new();
    std::fs::File::open(path).expect("file doesn't exist create config.toml").read_to_string(&mut buf).expect("IO error when reading file");
    toml::from_str(&buf).unwrap_or_else(|e|{panic!("Toml error:\n{}",e.to_string())})
}

/// Corresponds to the root endpoint and is useful to verify the service is up
#[get("/")]
fn ping()->String{
    format!("Pong the repo creation endpoint is up!")
}

/// Corresponds to the endpoint /create_repo, this endpoint takes a string for it's body which it
/// will use as the name of the repo it will initiate
#[post("/create_repo", data = "<repo_name>")]
fn create_repo(repo_name:String)->Result<String,Box<dyn std::error::Error>>{
    let mut builder = std::fs::DirBuilder::new();
    let mut pth = CONFIG.repos.clone();
    
    // just making sure the directory for our repos exists
    builder.recursive(true);
    builder.create(&pth)?;

    builder.recursive(false);
    pth.push(repo_name.as_str());
    builder.create(&pth)?;
    git2::Repository::init_bare(pth)?;
    Ok(repo_name)
}

/// The endpoint to change the description of a repo
#[post("/<repo_name>/description", data = "<description>")]
fn change_description(repo_name:String,description:String)-> Result<String, Box<dyn std::error::Error>>{
    let mut pth = CONFIG.repos.clone();
    pth.push(&repo_name);
    pth.push("description");
    let mut file = std::fs::File::create(pth)?;
    write!(file,"{}",description)?;
    Ok(repo_name)
}

/// The endpoint to get the current description of a repo
#[get("/<repo_name>/description")]
fn read_description(repo_name:String)-> Result<String, Box<dyn std::error::Error>>{
    let mut pth = CONFIG.repos.clone();
    pth.push(&repo_name);
    pth.push("description");
    let mut file = std::fs::File::open(pth)?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    Ok(buf)
}
