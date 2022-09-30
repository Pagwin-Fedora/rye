#![feature(proc_macro_hygiene, decl_macro, try_blocks, try_trait_v2)]
#[macro_use] extern crate rocket;
extern crate git2;
extern crate dirs;
extern crate lazy_static;
use std::{io::{Read, Write}};

use lazy_static::lazy_static;
use rocket::Data;

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
fn create_repo(repo_name:Data)->String{
    //thank you try block for letting me write the code I want
    let tmp: Result<String,Box<dyn std::error::Error>> = try{
        let repo_name = {
            let mut tmp = String::new();
            repo_name.open().read_to_string(&mut tmp)?;
            tmp
        };
        let mut builder = std::fs::DirBuilder::new();
        let mut pth = CONFIG.repos.clone();
        builder.recursive(true);
        builder.create(&pth).map(|_|{()})?;

        builder.recursive(false);
        pth.push(repo_name.as_str());
        builder.create(&pth).map(|_|{()})?;
        git2::Repository::init_bare(pth)?;
        repo_name.into()
    };

    match tmp{
        Ok(name)=>name,
        Err(e)=>e.to_string()
    }
}

/// The endpoint to change the description of a repo
#[post("/<repo_name>/description", data = "<description>")]
fn change_description(repo_name:String,description:Data)-> String{
    //building the path to the description
    let mut pth = CONFIG.repos.clone();
    pth.push(&repo_name);
    pth.push("description");

    let tmp: Result<(),std::io::Error> = try{
        let description = {
            let mut tmp = String::new();
            description.open().read_to_string(&mut tmp)?;
            tmp
        };
        let mut file = std::fs::File::create(pth)?;
        write!(file,"{}",description)?;
        ().into()
    };
    match tmp{
        Ok(_)=>repo_name,
        Err(e)=>e.to_string()
    }
}

/// The endpoint to get the current description of a repo
#[get("/<repo_name>/description")]
fn read_description(repo_name:String)-> String{
    let mut pth = CONFIG.repos.clone();
    pth.push(&repo_name);
    pth.push("description");
    let res:Result<String, Box<dyn std::error::Error>> = try{
        let mut buf = String::new();
        let mut file = std::fs::File::open(pth)?;
        file.read_to_string(&mut buf)?;
        buf.into()
    };
    match res{
        Ok(desc)=>desc,
        Err(e)=>e.to_string()
    }
}
