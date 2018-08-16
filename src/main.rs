#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
extern crate sqlite;
extern crate serde;

#[macro_use]
extern crate serde_derive;

use serde::Serialize;
use std::collections::HashMap;
use rocket_contrib::Template;
use sqlite::{Connection, Value};
use rocket::response::NamedFile;
use std::path::{Path,PathBuf};

#[derive(Serialize)]
struct Post {
    id: i64,
    title: String,
    content: String,
    footer: String,
    date: String,
}

#[derive(Serialize)]
struct Summary {
    id: i64,
    title: String,
    date: String
}

#[derive(Serialize)]
struct GroupContent<C: Serialize> {
    items: Vec<C>
}

#[derive(Serialize)]
struct PageContent<C: Serialize, T: Serialize> {
    summaries: Vec<T>,
    content: C,
    parent: String
}


const POST_QUERY_BASE: &str = "select 
                                id, 
                                content, 
                                title, 
                                footer, 
                                timeadd 
                                from posts";

const POST_QUERY_COL: &[&str] = &["id", 
                                  "content", 
                                  "title", 
                                  "footer", 
                                  "timeadd"];

const SUMMARY_QUERY_BASE: &str = "select 
                                    id, 
                                    title, 
                                    timeadd 
                                    from posts";

const SUMMARY_QUERY_COL: &[&str] = &["id",
                                     "title",
                                     "timeadd"];


//create a database connection
fn get_connection() -> Connection {
    sqlite::open("blog.db").unwrap()
}


fn execute_query(query: String, 
                  cols: Vec<String>, 
                  inputs: &[Value]) -> Vec<HashMap<String, Value>>{
    //get the connection
    let conn = get_connection();
    //create the cursor
    let mut cursor = conn.prepare(query).unwrap().cursor();
    //bind the values to the prepared statement
    cursor.bind(inputs).unwrap();

    //initialize the returned array
    let mut res: Vec<HashMap<String, Value>> = Vec::new();

    //loop through the cursor
    while let Some(row) = cursor.next().unwrap() {
        //collect the row values
        let row_vals: Vec<Value> = row.to_vec();

        //zip the row column names with the values
        let row_map: HashMap<String,Value> = cols.clone()
                                                 .into_iter()
                                                 .zip(row_vals.into_iter())
                                                 .collect();
        //add the hashmap to the returned vector
        res.push(row_map);
    }
    res
}


fn map_to_summary(map: &HashMap<String, Value>) -> Summary {
    Summary {
        id: map.get("id").unwrap()
                         .as_integer()
                         .unwrap(),
        title: map.get("title").unwrap()
                               .as_string()
                               .unwrap()
                               .to_string(),
        date: map.get("timeadd").unwrap()
                                .as_string()
                                .unwrap()
                                .to_string()[0..10]
                                .to_string()
    }
}


fn map_to_post(map: &HashMap<String, Value>) -> Post {
    Post {
        id: map.get("id").unwrap()
                         .as_integer()
                         .unwrap(),
        title: map.get("title").unwrap()
                               .as_string()
                               .unwrap()
                               .to_string(),
        content: map.get("content").unwrap()
                                   .as_string()
                                   .unwrap()
                                   .to_string(),
        footer: map.get("footer").unwrap()
                                 .as_string()
                                 .unwrap()
                                 .to_string(),
        date: map.get("timeadd").unwrap()
                                .as_string()
                                .unwrap()
                                .to_string()[0..10]
                                .to_string()
    }
}


fn get_summary_latest(limit: i32) -> Vec<Summary> {
    let query = SUMMARY_QUERY_BASE.to_string() +
                " order by id desc limit ?";
    let cols = SUMMARY_QUERY_COL.to_vec()
                                .iter()
                                .map(|x| x.to_string())
                                .collect();
    let inputs = &[Value::Integer(limit.into())];
    let query_result: Vec<Summary> = execute_query(query,cols,inputs)
                                                .iter()
                                                .map(|x| map_to_summary(x))
                                                .collect();
    query_result
}


//get the sidebar summaries
fn get_sidebar_summary() -> Vec<Summary> {
    get_summary_latest(6)
}



fn get_posts_latest(limit: i32) -> Vec<Post> {
    let query = POST_QUERY_BASE.to_string() +
                    " order by id desc limit ?";
    let cols = POST_QUERY_COL.to_vec()
                             .iter()
                             .map(|x| x.to_string())
                             .collect();
    let inputs = &[Value::Integer(limit.into())];
    let query_result: Vec<Post> = execute_query(query,cols,inputs)
                                    .iter()
                                    .map(|x| map_to_post(x))
                                    .collect();
    query_result
}


fn get_post_by_id(id: i32) -> Vec<Post> {
    let query = POST_QUERY_BASE.to_string() +
                    " where id=?";
    let cols = POST_QUERY_COL.to_vec()
                             .iter()
                             .map(|x| x.to_string())
                             .collect();
    let inputs = &[Value::Integer(id.into())];
    let query_result: Vec<Post> = execute_query(query,cols,inputs)
                                    .iter()
                                    .map(|x| map_to_post(x))
                                    .collect();
    query_result
}



fn create_template<C: Serialize>(parent_content: C, 
                                 parent: &str, 
                                 wrapper_template_name: &str) -> Template {
    
    let sidebar_summaries = get_sidebar_summary();
    let content = PageContent {content: parent_content, 
                               summaries: sidebar_summaries, 
                               parent: parent.to_string()};
    
    let full = Template::render(wrapper_template_name.to_string(), &content);
    full
}


//endpoints
//==========================


#[get("/res/<file..>")]
fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("resource/").join(file)).ok()
}


#[get("/")]
fn index() -> Template {
    let posts = get_posts_latest(5);
    let content = GroupContent {items: posts};
    create_template(content, "blog", "index")
}

#[get("/<id>")]
fn get_post(id: i32) -> Template {
    let posts = get_post_by_id(id);
    let content = GroupContent {items: posts};
    create_template(content, "blog", "index")
}


#[get("/browse")]
fn browse_posts() -> Template {
    let summaries: Vec<Summary> = get_summary_latest(1000);
    let content = GroupContent {items: summaries};
    create_template(content, "browse", "index")
}   


fn main() {
    println!("Hello, world!");
    rocket::ignite()
        .mount("/", routes![index, 
                            files, 
                            get_post, 
                            browse_posts])
        .attach(Template::fairing())
        .launch();
}

