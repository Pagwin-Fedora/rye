extern crate reqwest;
extern crate lazy_static;
extern crate clap;
extern crate serde;
extern crate toml;
use lazy_static::lazy_static;
lazy_static!{
    static ref CONFIG:Config<'static> = {
        //convert to use toml and read from ~/.config/rye-cli/config.toml
        Config{remote_url:"http://localhost:9090/".to_string(), repo_template:"ssh://git@git.pagwin.xyz/git-server/repos/%%%"}
    };
    static ref HTTP:reqwest::blocking::Client = reqwest::blocking::Client::new();
}
fn main()->Result<(),Box<dyn std::error::Error>>{
    let mut line = String::new();
    std::io::stdin().read_line(&mut line)?;
    create_repo(line,None)?;
    Ok(())
}
fn create_repo(name:String,auth:Option<Auth>)->reqwest::Result<String>{
    let response = {
        let tmp = HTTP.post(CONFIG.remote_url.clone()+"create_repo")
            .body(name);
        match auth{
            Some(auth)=>{
                match auth{
                    Auth::Basic(username,password)=>{
                        tmp.basic_auth(username, password)
                    },
                    Auth::Bearer(token)=>{
                        tmp.bearer_auth(token)
                    }
                }
            }
            None=>{
                tmp 
            }
        }
    }.send()?;
    Ok(CONFIG.repo_template.replace("%%%", response.error_for_status()?.text()?.as_str()))
}

#[derive(serde::Serialize,serde::Deserialize)]
enum Auth{Basic(String,Option<String>),Bearer(String)}

#[derive(serde::Serialize,serde::Deserialize)]
struct Config<'a>{
    remote_url:String,
    // repo_template should have 3 %'s where the repo's name should go
    repo_template: &'a str,
    auth:Option<Auth>
}
