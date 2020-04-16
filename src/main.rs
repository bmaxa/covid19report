#![feature(try_blocks)]
extern crate serde_json;
extern crate reqwest;
extern crate base64;
use serde_json::Value;
use futures::Future;
use std::fs::File;

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
        let response = get_response(url);
        let body = response.await.text().await?;
        let parsed:Value = serde_json::from_str(body.as_str()).unwrap();
        let mut rc = Vec::new();
        for names in parsed.as_array().unwrap() {
            rc.push(names["name"].clone());
        }
        Ok(rc)
    }
}

fn parse_files(args:Vec<Value>)->impl Future<Output = std::result::Result<Value,reqwest::Error>> {
    async {
        let mut map = serde_json::Map::new();
        for name in args {
            let url ="https://api.github.com/repos/cssegisanddata/covid-19/contents/csse_covid_19_data/csse_covid_19_daily_reports".to_string() + "/" + name.as_str().unwrap();
            let response = get_response(url);
            let body = response.await.text().await?;
            match serde_json::from_str(body.as_str()) {
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
                Err(_) => ()
            };
        }
        Ok(Value::Null)
    }
}
fn vec2string(v:Vec<u8>)->String{
    std::str::from_utf8(&v).unwrap().to_string()
}

async fn get_response(url:String)->reqwest::Response {
   let client = reqwest::Client::builder()
            .user_agent("rust")
            .build().unwrap();
    loop {
        let response =client.get(&url).send().await.unwrap(); 
        let rest = response.headers()["X-RateLimit-Remaining"].to_str().unwrap().parse::<i32>().unwrap();
        println!("rest: {}",rest);
        if rest != 0 { return response; }
        let millis = std::time::Duration::from_secs(1);
        std::thread::sleep(millis);
    }
}
