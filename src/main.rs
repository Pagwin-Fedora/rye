#![feature(proc_macro_hygiene, decl_macro)]

extern crate git2;
#[macro_use] extern crate rocket;
fn main() {
    rocket::ignite().mount("/",routes![ping,create_repo]).launch();
}
#[get("/")]
fn ping()->String{
    format!("Pong the repo creation endpoint is up!")
}
#[post("/create_repo/<repo_name>")]
fn create_repo(repo_name:String)-> String{
    repo_name
}
