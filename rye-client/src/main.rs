extern crate reqwest;
extern crate lazy_static;
extern crate clap;
extern crate serde;
extern crate toml;
extern crate dirs;
extern crate regex;
use std::{path::Path, io::Read};

use clap::{Command, Arg, ValueHint};
use lazy_static::lazy_static;
use regex::Regex;
lazy_static!{
    static ref CONFIG:Option<Config> = Config::from_file(default_config()).ok();
    static ref HTTP:reqwest::blocking::Client = reqwest::blocking::Client::new();
    static ref BASIC_USER:Regex = Regex::new("username:.*,").expect("internal regex error on auth, please try again");
    static ref BASIC_PASS:Regex = Regex::new("password:.*").expect("internal regex error on auth, please try again");
    static ref TOKEN:Regex = Regex::new("token:.*").expect("internal regex error on auth please try again");
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
    let matches = cli.get_matches();
    let config = match matches.get_one::<std::path::PathBuf>("config"){
        Some(path)=>{
            Config::from_file(path)?
        },
        None=>{
            CONFIG.clone().unwrap()
        }
    };
    let sub = match matches.subcommand_name(){
        Some(s)=>s,
        None=>{
            cmd().print_help()?;
            std::process::exit(1);
        }
    };
    let sub_matches = matches.subcommand_matches(sub).unwrap();
    match sub.into() {
        SubCommands::CreateRepo=>{
            let repo_name = match sub_matches.get_one::<String>("repo_name"){
                Some(name)=>name,
                None=>{
                    eprintln!("Didn't provide a name for the repository being created");
                    std::process::exit(1);
                }
            };
            println!("{}",create_repo(repo_name.clone(), &config)?);
        },
        SubCommands::GetDescription=>{
            let repo_name = match sub_matches.get_one::<String>("repo_name"){
                Some(name)=>name,
                None=>{
                    eprintln!("Didn't provide a name for the repository you want the description of");
                    std::process::exit(1);
                }
            };
            println!("{}",get_description(repo_name.clone(), &config)?);
        },
        SubCommands::SetDescription=>{
            let repo_name = match sub_matches.get_one::<String>("repo_name"){
                Some(name)=>name,
                None=>{
                    eprintln!("Didn't provide a name for the repository you're setting the description of");
                    std::process::exit(1);
                }
            };
            let new_description = match sub_matches.get_one::<String>("new_description"){
                Some(name)=>name,
                None=>{
                    eprintln!("Didn't provide a new description");
                    std::process::exit(1);
                }
            };
            set_description(repo_name.clone(), new_description.clone(), &config)?;
        }
    }

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
            .arg(Arg::new("repo_name").value_hint(ValueHint::Other).index(1)))
        .subcommand(Command::new("set_description")
            .arg(Arg::new("repo_name").value_hint(ValueHint::Other).index(1))
            .arg(Arg::new("new_description").value_hint(ValueHint::Other).index(2)));
    program

}
fn create_repo(name:String,Config {auth, remote_url, repo_template,..}:&Config)->reqwest::Result<String>{
    let response = {
        let tmp = HTTP.post(remote_url.clone()+"create_repo")
            .body(name);
        handle_auth(tmp, auth.clone())
    }.send()?;
    Ok(repo_template.replace("%%%", response.error_for_status()?.text()?.as_str()))
}
fn get_description(repo_name:String,Config{auth, remote_url,..}:&Config)->reqwest::Result<String>{
    let response = {
        let tmp = HTTP.get(remote_url.clone()+repo_name.as_ref()+"/description");
        handle_auth(tmp, auth.clone())
    }.send()?;
    Ok(response.error_for_status()?.text()?)
}
fn set_description(repo_name:String, new_description:String,Config{auth,remote_url,..}:&Config)->reqwest::Result<()>{
    let response = {
        let tmp = HTTP.post(remote_url.clone()+repo_name.as_ref()+"/description")
            .body(new_description);
        handle_auth(tmp, auth.clone())
    }.send()?;
    response.error_for_status().map(|_|{})
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


#[derive(serde::Serialize,serde::Deserialize,Clone)]
enum Auth{Basic(String,Option<String>),Bearer(String)}
impl From<String> for Auth{
    // TODO: refactor this mess
    fn from(val: String) -> Self {
        match TOKEN.captures(val.as_str()){
            Some(captures)=>{
                if let Some(Some(val)) = captures.iter().next(){
                    Self::Bearer(val.as_str().split(":").nth(1).unwrap().to_owned())
                }
                else{
                    panic!("Invalid Auth given");
                }
            },
            None=>{
                let vals = val.split(",").collect::<Vec<_>>();
                let username = BASIC_USER.captures(vals[0])
                    .expect("Invalid Auth given")
                    .iter().next().unwrap().unwrap()
                    .as_str().split(":").nth(1).unwrap().to_owned();
                let password = BASIC_USER.captures(vals[0])
                    .map(|capture|capture
                    .iter().next().flatten().map(|val|val
                    .as_str().split(":").nth(1).unwrap().to_owned())).flatten();
                Self::Basic(username, password)
            }
        }
    }
}
enum SubCommands{CreateRepo,GetDescription,SetDescription}
impl From<&str> for SubCommands{
    fn from(val: &str) -> Self {
        match val{
            "create_repo"=>{
                Self::CreateRepo
            },
            "get_description"=>{
                Self::GetDescription
            },
            "set_description"=>{
                Self::SetDescription
            },
            _=>panic!("Unexpected internal situation")
        }
        
    }
}
#[derive(serde::Serialize,serde::Deserialize,Clone)]
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
