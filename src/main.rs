extern crate serde_json;
extern crate reqwest;
extern crate base64;
use serde_json::Value;
use futures::Future;
use std::fs::File;
use std::process::exit;
use std::io::prelude::*;


fn main() {
    let future = get_list(); // Nothing is printed
    let mut r = tokio::runtime::Runtime::new().unwrap();
    let res = r.block_on(future);
    let future = parse_files(res.unwrap());
    let _ = r.block_on(future);
}

fn get_list()->impl Future<Output = std::result::Result<Vec<Value>,reqwest::Error>> {
    async {
        let url  = "https://api.github.com/repos/cssegisanddata/covid-19/contents/csse_covid_19_data/csse_covid_19_daily_reports".to_string();
        let body = get_response(url);
        let body = body.await;
        let parsed:Result<Value,_> = serde_json::from_str(&body);
        let parsed = match parsed {
            Err(error) => { 
                println!("probably authorization error: {}",error);
                exit(0);
        },
            Ok(parsed) => parsed
        };
        let mut rc = Vec::new();
        if !parsed.is_array() {
                println!("probably authorization error");
                exit(0);
        }
        let mut flag = false;
        for names in parsed.as_array().unwrap() {
            if !flag {
                if let Some(date) = options().date {
                    if let Some(_) = names["name"].as_str().unwrap().find(&date){
                        rc.push(names["name"].clone());
                        flag = true;
                        println!("found date {} {}",date,names["name"]);
                    }
                } else {
                    rc.push(names["name"].clone());
                }
            }
            else {
                rc.push(names["name"].clone());
            }
        }
        Ok(rc)
    }
}

fn parse_files(args:Vec<Value>)->impl Future<Output = std::result::Result<Value,reqwest::Error>> {
    async {
        let mut map = serde_json::Map::new();
        for name in args {
            let url ="https://api.github.com/repos/cssegisanddata/covid-19/contents/csse_covid_19_data/csse_covid_19_daily_reports".to_string() + "/" + name.as_str().unwrap();
            let body = get_response(url);
            let body = body.await;
            match serde_json::from_str(&body) {
                Ok(body) => {
                    let a:Value = body;
                    let d = a["name"].as_str().unwrap().split('.').collect::<Vec<_>>()[0];
                    match chrono::NaiveDate::parse_from_str(d,"%m-%d-%Y"){
                        Ok(dt)=> 
                        { 
                            println!("{:?}",dt); 
                            let cnt = a["content"].as_str();
                            let cnt = vec2string(cnt.unwrap().chars().filter(|x| *x!='\n' && *x != '\r').map(|x|x as u8).collect::<Vec<_>>()); 
                            println!("{}",cnt);
                            let s =vec2string(base64::decode(cnt).unwrap());
                            let mut rdr = csv::ReaderBuilder::new().from_reader(s.as_bytes());
                            let mut v = serde_json::Map::new();
                            let mut vv = Vec::new();
                            for row in rdr.headers().unwrap().iter() {
                                println!("{}",row);
                                v.insert(row.to_string(),serde_json::Value::Array(Vec::new()));
                                vv.push(row.to_string());

                            }
                            for row in rdr.records() {
                                for (field,name) in row.unwrap().iter().zip(vv.iter()) {
                                    println!("{}, {}",name,field);
                                    v.get_mut(name).unwrap().as_array_mut().unwrap().push(serde_json::Value::String(field.to_string()));
                                }
                            }
                            map.insert(d.to_string(),serde_json::Value::Object(v));
                            let f = File::create("foo.json");
                            let _ = f.unwrap().write_all(serde_json::to_string_pretty(&serde_json::Value::Object(map.clone())).unwrap().as_bytes());
                        },
                        Err(e) => { println!("err: {:?} {}",e,d); }
                    }
                },
                Err(error) => {
                    println!("Probably auth error {}",error);
                    exit(0);
                }
            };
        }
        Ok(Value::Null)
    }
}
fn vec2string(v:Vec<u8>)->String{
    std::str::from_utf8(&v).unwrap().to_string()
}

async fn get_response(url:String)->String {
    loop {
        let client = reqwest::Client::builder()
            .user_agent("rust")
            .build().unwrap();
        let request =client.get(&url);
        let request = build_request(request);
        let response = request.send().await.unwrap();
        let rest = response.headers()["X-RateLimit-Remaining"].to_str().unwrap().parse::<i32>().unwrap();
        let date = chrono::NaiveDateTime::from_timestamp(response.headers()["X-RateLimit-Reset"].to_str().unwrap().parse::<i64>().unwrap(),0);
        let body = response.text().await.unwrap();
        println!("rest: {} available: {}",rest,date);
        if rest != 0 { return body; }
        println!("{}",body);
        let millis = std::time::Duration::from_secs(60);
        std::thread::sleep(millis);
    }
}
fn build_request(request: reqwest::RequestBuilder)->reqwest::RequestBuilder {
    let request = if let Some (username) = options().username { 
        request.basic_auth(username,if let Some(password) = options().password {
            Some(password) 
        } else { 
            None }) 
    } else 
    { 
        request 
    };
    request
}
struct Options {
    username: Option<String>,
    password: Option<String>,
    date: Option<String>
}
fn options()->Options {
    let mut options = Options{ 
        password:None,
        username: None,
        date:None
    };
    let args = std::env::args().collect::<Vec<String>>();
    fn print_usage(program: &str, opts:& getopts::Options) {

        let brief = format!("Usage: {} [options]", program);
        print!("{}", opts.usage(&brief));
    }

    let program = args[0].clone();
    let mut opts = getopts::Options::new();

    opts.optopt("p", "password", "set password", "password");
    opts.optopt("u", "username", "set username", "username");
    opts.optopt("d", "date", "set date", "m-d-y");
    opts.optflag("h", "help", "this help");
    let matches = match opts.parse(&args[1..]) {

        Ok(m) => { m }

        Err(f) => { panic!(f.to_string()) }

    };
    if matches.opt_present("h") {

        print_usage(&program, &opts);
        exit(0);
    }
    if let Some(output) = matches.opt_str("u") {
        options.username = Some(output);
    }
    if let Some(output) = matches.opt_str("p") {
        options.password= Some(output);
    }
    if let Some(output) = matches.opt_str("d") {
        options.date= Some(output);
    }
    options
}
