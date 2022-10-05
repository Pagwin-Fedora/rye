extern crate reqwest;
extern crate lazy_static;
extern crate clap;
extern crate serde;
extern crate toml;
extern crate dirs;
use std::{path::Path, io::Read};

use clap::{Command, Arg, ValueHint};
use lazy_static::lazy_static;
lazy_static!{
    static ref CONFIG:Option<Config> = Config::from_file(default_config()).ok();
    static ref HTTP:reqwest::blocking::Client = reqwest::blocking::Client::new();
}
fn default_config()->impl AsRef<Path>{
    let mut buf = std::path::PathBuf::new();
    buf.push(dirs::config_dir().expect("no config dir on system"));
    buf.push("config.toml");
    buf
}
fn main()->Result<(),Box<dyn std::error::Error>>{
    //let mut line = String::new();
    //std::io::stdin().read_line(&mut line)?;
    //println!("{}",create_repo(line,None)?);
    let mut cli = cmd();
    cli.build();
    cli.get_matches().get_one("auth")
    
    Ok(())
}
/// function that's used to construct the clap command object we use for cli stuff
fn cmd()->Command{
    let auth_arg = Arg::new("auth")
        .short('a')
        .long("auth")
        .global(true)
        .value_hint(ValueHint::Other);
    let config_arg = Arg::new("config")
        .short('c')
        .long("config")
        .value_hint(ValueHint::FilePath);
    let program = Command::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .arg(auth_arg)
        .arg(config_arg)
        .subcommand(Command::new("create_repo")
            .arg(Arg::new("repo_name").value_hint(ValueHint::Other).index(1)))
        .subcommand(Command::new("get_description")
            .arg(Arg::new("repo_name").value_hint(ValueHint::Other).index(1)));
    program

}
fn create_repo(name:String,auth:Option<Auth>)->reqwest::Result<String>{
    let response = {
        let tmp = HTTP.post(CONFIG.remote_url.clone()+"create_repo")
            .body(name);
        handle_auth(tmp, auth)
    }.send()?;
    Ok(CONFIG.repo_template.replace("%%%", response.error_for_status()?.text()?.as_str()))
}
fn get_description(repo_name:String,auth:Option<Auth>)->reqwest::Result<String>{
    let response = {
        let tmp = HTTP.get(CONFIG.remote_url.clone()+repo_name.as_ref()+"/description");
        handle_auth(tmp, auth)
    }.send()?;
    Ok(response.error_for_status()?.text()?)
}
fn set_description(repo_name:String, new_description:String,auth:Option<Auth>)->reqwest::Result<()>{
    let response = {
        let tmp = HTTP.post(CONFIG.remote_url.clone()+repo_name.as_ref()+"/description")
            .body(new_description);
        handle_auth(tmp, auth)
    }.send()?;
    response.error_for_status().map(|_|{})
}
struct MyResult<T,E>(Result<T,E>);
impl<T:From<U>,U,E> From<Result<U,E>> for MyResult<T,E>{
  fn from(res:Result<U,E>)->Self{
    MyResult(res.map(T::from))
  }
}
fn handle_auth(req:reqwest::blocking::RequestBuilder,auth:Option<Auth>)->reqwest::blocking::RequestBuilder{
    match auth{
        Some(auth)=>{
            match auth{
                Auth::Basic(username,password)=>{
                    req.basic_auth(username, password)
                },
                Auth::Bearer(token)=>{
                    req.bearer_auth(token)
                }
            }
        }
        None=>{
            req
        }
    }
}


#[derive(serde::Serialize,serde::Deserialize)]
enum Auth{Basic(String,Option<String>),Bearer(String)}

#[derive(serde::Serialize,serde::Deserialize)]
struct Config{
    remote_url:String,
    // repo_template should have 3 %'s where the repo's name should go
    repo_template: String,
    auth:Option<Auth>
}
impl Config{
    fn from_file<P:AsRef<Path>>(loc:P)->Result<Config,Box<dyn std::error::Error>>{
        let mut buf = Vec::new();
        let mut file = std::fs::File::open(loc)?;
        file.read_to_end(&mut buf)?;
        toml::from_slice(buf.as_slice()).map_err(Box::from)
    }
}
