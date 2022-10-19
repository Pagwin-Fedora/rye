#![feature(proc_macro_hygiene, decl_macro, try_blocks, try_trait_v2)]
#[macro_use] extern crate rocket;
extern crate git2;
extern crate dirs;
extern crate lazy_static;
use std::{io::{Read, Write}, ops::{FromResidual, Try}};

use lazy_static::lazy_static;
use rocket::Data;

lazy_static!{
    static ref CONFIG:Config = retrieve_config();
}

fn main() {
    //making sure lazy_static code runs immediately before rocket starts spawning worker threads
    let _ = CONFIG.repos;
    rocket::ignite().mount("/",routes![ping, create_repo, change_description, read_description]).launch();
}


#[derive(serde::Serialize,serde::Deserialize)]
struct Config{
    repos:std::path::PathBuf,
}

/// function that retrieves the config
fn retrieve_config<'a>() -> Config{
    let path = match std::env::var(std::env!("CARGO_PKG_NAME").to_string().to_uppercase()+"_CONFIG"){
        Ok(v)=>v.into(),
        Err(_)=>{
            let mut tmp = dirs::config_dir().expect("No config directory");
            tmp.push(std::option_env!("CARGO_PKG_NAME").unwrap());
            tmp.push("config.toml");
            tmp
        }
    };
    let mut buf = String::new();
    std::fs::File::open(path).expect("file doesn't exist create config.toml").read_to_string(&mut buf).expect("IO error when reading file");
    toml::from_str(&buf).unwrap_or_else(|e|{panic!("Toml error:\n{}",e.to_string())})
}

/// Corresponds to the root endpoint and is useful to verify the service is up
#[get("/")]
fn ping()->String{
    format!("Pong the repo creation endpoint is up!")
}
struct EasyDisplay(Result<String,String>);
impl From<EasyDisplay> for Result<String,String>{
    fn from(v: EasyDisplay) -> Self {
        v.0
    }
}
impl From<Result<String,String>> for EasyDisplay{
    fn from(v: Result<String,String>) -> Self {
        EasyDisplay(v)
    }
}
impl <'a> rocket::response::Responder<'a> for EasyDisplay{
    fn respond_to(self, _request: &rocket::Request) -> rocket::response::Result<'a> {
        let mut builder = rocket::response::ResponseBuilder::new(rocket::response::Response::new());
        match self.into(){
            Result::Ok(v)=>{
                builder.status(rocket::http::Status::Ok);
                builder.sized_body(std::io::Cursor::new(v));
            },
            Result::Err(v)=>{
                builder.status(rocket::http::Status::InternalServerError);
                builder.sized_body(std::io::Cursor::new(v));
            }
        }
        rocket::response::Result::Ok(builder.finalize())
    }
}
impl Try for EasyDisplay{
    type Output = String;
    type Residual = String;
    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self.0 {
            Ok(v)=>std::ops::ControlFlow::Continue(v),
            Err(v)=>std::ops::ControlFlow::Break(v)
        }
    }
    fn from_output(output: Self::Output) -> Self {
        EasyDisplay(Ok(output))
    }
}
impl FromResidual for EasyDisplay{
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        EasyDisplay(Err(residual))
    }
}
/// Corresponds to the endpoint /create_repo, this endpoint takes a string for it's body which it
/// will use as the name of the repo it will initiate
#[post("/create_repo", data = "<repo_name>")]
fn create_repo(repo_name:Data)->EasyDisplay{
    //thank you try block for letting me write the code I want
    let tmp: Result<String,Box<dyn std::error::Error>> = try{
        let repo_name = {
            let mut tmp = String::new();
            repo_name.open().read_to_string(&mut tmp).unwrap();
            tmp
        };
        let mut builder = std::fs::DirBuilder::new();
        let mut pth = CONFIG.repos.clone();
        builder.recursive(true);
        builder.create(&pth).unwrap();

        builder.recursive(false);
        pth.push(repo_name.as_str());
        builder.create(&pth).unwrap();
        git2::Repository::init_bare(pth).unwrap();
        repo_name.into()
    };
    tmp.map_err(|e|e.to_string()).into()
}

/// The endpoint to change the description of a repo
#[post("/<repo_name>/description", data = "<description>")]
fn change_description(repo_name:String,description:Data)-> EasyDisplay{
    //building the path to the description
    let mut pth = CONFIG.repos.clone();
    pth.push(&repo_name);
    pth.push("description");

    let tmp: Result<(),std::io::Error> = try{
        let description = {
            let mut tmp = String::new();
            description.open().read_to_string(&mut tmp).unwrap();
            tmp
        };
        let mut file = std::fs::File::create(pth).unwrap();
        write!(file,"{}",description).unwrap();
        ().into()
    };
    tmp.map(|_|repo_name).map_err(|e|e.to_string()).into()
}

/// The endpoint to get the current description of a repo
#[get("/<repo_name>/description")]
fn read_description(repo_name:String)-> EasyDisplay{
    let mut pth = CONFIG.repos.clone();
    pth.push(&repo_name);
    pth.push("description");
    let res:Result<String, Box<dyn std::error::Error>> = try{
        let mut buf = String::new();
        let mut file = std::fs::File::open(pth).unwrap();
        file.read_to_string(&mut buf).unwrap();
        buf.into()
    };
    res.map_err(|e|e.to_string()).into()
}
