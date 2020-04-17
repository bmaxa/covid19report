use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::process::exit;
fn main(){
    let opts = options();
    let mut f = File::open(opts.file.as_str()).unwrap();
    let mut buf = String::new();
    let _ = f.read_to_string(&mut buf);
    let data:Value = serde_json::from_str(&buf).unwrap();
    display_stats(data,&opts);
}
#[derive(Clone)]
struct Options {
pub key: String,
pub file: String,
pub date: Option<String>,
pub sort: Sort,
pub columns: Vec<String>,
pub results: i32,
pub key_value: Option<String>,
pub type_: Type
}
#[derive(Clone,PartialEq)]
enum Sort {
    Asc,
    Desc
}
#[derive(Clone,PartialEq)]
enum Type {
    Numeric,
    String
}
fn options()->Options{
    let mut options = Options{ 
        key:"Confirmed".to_string(),
        file:"foo.json".to_string(),
        date: None,
        sort: Sort::Desc,
        columns: vec!["COUNTRY".to_string(),"CONFIRMED".to_string(),"DEATHS".to_string(),"RECOVERED".to_string(),"ACTIVE".to_string()],
        results: 5,
        key_value: None,
        type_: Type::Numeric
    };
    let args = std::env::args().collect::<Vec<String>>();
    fn print_usage(program: &str, opts:& getopts::Options) {

        let brief = format!("Usage: {} [options]", program);
        print!("{}", opts.usage(&brief));
    }

    let program = args[0].clone();



    let mut opts = getopts::Options::new();
    /*key: String,
    file: String,
    date: Option<String>,
    sort: Sort,
    columns: Vec<String>,
    results: i32,
    key_value: Option<String>,
    type_: Type*/

    opts.optopt("k", "key", "set sorting key", "Column name");
    opts.optopt("f", "file", "set input file", "filename");
    opts.optopt("d", "date", "set date", "mm-dd-yyyy");
    opts.optopt("s", "sort", "set sort", "asc or desc");
    opts.optopt("c", "columns", "set columns", "colname1,colname2,..");
    opts.optopt("r", "results", "set number of results", "number");
    opts.optopt("", "key_value", "set particular value of key", "key value");
    opts.optopt("t", "type", "set type of key column", "numeric or string ");

    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {

        Ok(m) => { m }

        Err(f) => { panic!(f.to_string()) }

    };

    if matches.opt_present("h") {

        print_usage(&program, &opts);
        exit(0);
    }

    if let Some(output) = matches.opt_str("k") {
        options.key = output;
    }
    if let Some(output) = matches.opt_str("f") {
        options.file = output;
    }
    let output = matches.opt_str("d");
    options.date = output;
    if let Some(output) = matches.opt_str("s") {
        options.sort= if output.to_lowercase() == "asc" {
            Sort::Asc
        } else {
            Sort::Desc
        }
    }
    if let Some(output) = matches.opt_str("c") {
        options.columns = output.split(',').map(|x|x.to_string()).collect();
        let mut found = false;
        for i in options.columns.iter() {
            if i.to_lowercase().contains(&options.key.to_lowercase()) {
                found = true;
                break;
            }
        }
        if !found { options.columns.push(options.key.clone()); }
    }
    if let Some(output) = matches.opt_str("r") {
        options.results = output.parse::<i32>().unwrap();
    }
    let output = matches.opt_str("key_value");
    options.key_value = output;
    let output = matches.opt_str("key_value"); 
    options.key_value = output;
    if let Some(output) = matches.opt_str("t") {
        options.type_ = if output.to_lowercase() == "numeric" {
            Type::Numeric
        } else {
            Type::String
        }
    }

    options


}

fn display_stats(data:Value,opts:&Options){
    let map = data.as_object().unwrap();
    let date_map = if let Some(ref date) = opts.date {
        find_date(map,date)
    } else {
        last_date(map)
    };
    if let None = date_map {
        println!("expected date {:?} not found",opts.date);
        return;
    }
    let mut stat_data = Vec::new();
    let mut key_data:Vec<(String,Vec<(usize,Value)>)> = Vec::new();

    let mut key_index = None;
    for (key,column) in  date_map.as_ref().unwrap(){
        if key.to_lowercase().contains(&opts.key.to_lowercase()) {
        key_index = column.as_array().unwrap().iter().enumerate().
                     fold(None,|v,(i,value)| 
                      if let Some(ref key_value) = opts.key_value {
                        if value.as_str().unwrap().to_lowercase() == key_value.to_lowercase() {
                                if v != None {
                                    v
                                } else {
                                    Some(i)
                                }
                        } else {
                            if v != None {
                                v
                            } else {
                                None
                            }
                        }

                      }else{
                            None
                         });
        }
    }
    for (key,column) in date_map.unwrap() {
        if contains(&opts.columns,&key) {
            let keys:Vec<Value> = column.as_array().unwrap().iter().enumerate().
                filter(|(i,_)|if let Some(key_index) = key_index {
                                    *i==key_index
                                  }else {
                                    true
                                  }).map(|(_,value)|value.clone()).collect(); 
            stat_data.push((key.clone(),keys.iter().enumerate().map(|(i,x)|(i,x.clone())).collect()));
            if key.to_lowercase().contains(&opts.key.to_lowercase()) {
                key_data.push((key,keys.iter().enumerate().map(|(i,x)|(i,x.clone())).collect()));
            }
        }
    }
    key_data[0].1.sort_by(|(_,x),(_,y)| if opts.type_ == Type::Numeric { 
                                            if opts.sort == Sort::Desc { y.as_str().unwrap().parse::<i32>().unwrap().partial_cmp(&x.as_str().unwrap().parse::<i32>().unwrap()).unwrap() }
                                            else { x.as_str().unwrap().parse::<i32>().unwrap().partial_cmp(&y.as_str().unwrap().parse::<i32>().unwrap()).unwrap() }
                                        } else {
                                            if opts.sort == Sort::Desc { y.as_str().unwrap().partial_cmp(x.as_str().unwrap()).unwrap() }
                                            else { x.as_str().unwrap().partial_cmp(y.as_str().unwrap()).unwrap() }
                                        });
    for value in opts.columns.iter() {
        print!("{:20}",value);
    }
    println!("");
    let mut counter = opts.results;
    for (key_index,_) in key_data[0].1.iter() {
        let mut row = Vec::new();
        for value in &opts.columns {
            if let Some(column) = find_column(&stat_data,value) {
                row.push(column[*key_index].1.as_str().unwrap());
            }
        }
        for value in row {
            print!("{:20}",value);
        }
        println!("");
        counter -= 1;
        if counter <= 0 { break }
    }
}

fn last_date(map:& serde_json::Map<String,Value>)->Option<serde_json::Map<String,Value>>{
    let mut max = (String::new(),chrono::NaiveDate::from_ymd(1971,1,1));
    for i in map.keys() {
        let d = chrono::NaiveDate::parse_from_str(i,"%m-%d-%Y").unwrap();
        if d > max.1 {
            max = (i.to_string(),d);
        }
    }
    Some(map.get(&max.0).unwrap().as_object().unwrap().clone())
}
fn find_date(map:& serde_json::Map<String,Value>, date:&String)->Option<serde_json::Map<String,Value>>{
    if let Some(date_map) = map.get(date) {
        Some(date_map.as_object().unwrap().clone())
    } else { None }
}

fn contains(cols:&Vec<String>,key:&String)->bool{
    for i in cols {
        if key.to_lowercase().contains(&i.to_lowercase()){
            return true;
        }
    }
    false
}
fn find_column<'l>(data: &'l Vec<(String,Vec<(usize,Value)>)>,value:&String)->Option<&'l Vec<(usize,Value)>> {
    for (i,v) in data {
        if i.to_lowercase().contains(&value.to_lowercase()) {
            return Some(v);
        }
    }
    None
}
