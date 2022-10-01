extern crate reqwest;
extern crate lazy_static;
extern crate clap;
extern crate serde;
extern crate toml;
use lazy_static::lazy_static;
lazy_static!{
    static ref CONFIG:Config<'static> = {
        Config{remote_url:"".to_string(), repo_template:"ssh://git@git.pagwin.xyz/git-server/repos/{1}"}
    };
    static ref HTTP:reqwest::blocking::Client = reqwest::blocking::Client::new();
}
fn main() {
    
}
fn create_repo(name:String,auth:Option<Auth>)->reqwest::Result<()>{
    let response = {
        let tmp = HTTP.post(&CONFIG.remote_url)
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
    response.error_for_status_ref().map(|_|{})
}

enum Auth{Basic(String,Option<String>),Bearer(String)}
struct Config<'a>{
    remote_url:String,
    repo_template: &'a str
}
