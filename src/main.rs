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
use std::error::Error;

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


type BoxResult<T> = Result<T, Box<Error>>;


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
fn get_connection() -> BoxResult<Connection> {
    Ok(sqlite::open("blog.db")?)
}


fn execute_query(query: String, 
                  cols: Vec<String>, 
                  inputs: &[Value]) 
                -> BoxResult<Vec<HashMap<String, Value>>>{

    //get the connection
    let conn = get_connection()?;
    //create the cursor
    let mut cursor = conn.prepare(query)?.cursor();
    //bind the values to the prepared statement
    cursor.bind(inputs)?;

    //initialize the returned array
    let mut res: Vec<HashMap<String, Value>> = Vec::new();

    //loop through the cursor
    while let Some(row) = cursor.next()? {
        //collect the row values
        let row_vals: Vec<Value> = row.to_vec();

        //zip the row column names with the values
        let row_map: HashMap<String,Value> = cols.clone()
                                                 .into_iter()
                                                 .zip(row_vals.into_iter())
                                                 .collect();
        //add the hashmap to the returned vector
        res.push(row_map);
    };
    Ok(res)
}

fn map_to_summary(map: &HashMap<String, Value>) -> Summary {
    let summr = Summary {
        id: map.get("id").unwrap_or(&Value::Integer(-1))
                         .as_integer()
                         .unwrap_or(-1),
        title: map.get("title").unwrap_or(&Value::String("Error: Title not found.".to_string()))
                                .as_string()
                                .unwrap_or("")
                                .to_string(),
        date: map.get("timeadd").unwrap_or(&Value::String("Error: Date not found.".to_string()))
                                 .as_string()
                                 .unwrap_or("")
                                 .to_string()[0..10]
                                 .to_string()
    };
    summr
}


fn map_to_post(map: &HashMap<String, Value>) -> Post {
    let post = Post {
        id: map.get("id").unwrap_or(&Value::Integer(-1))
                         .as_integer()
                         .unwrap_or(-1),
        title: map.get("title").unwrap_or(&Value::String("Error: Title not found.".to_string()))
                                .as_string()
                                .unwrap_or("")
                                .to_string(),
        content: map.get("content").unwrap_or(&Value::String("Error: Content not found.".to_string()))
                                    .as_string()
                                    .unwrap_or("")
                                    .to_string(),
        footer: map.get("footer").unwrap_or(&Value::String("Error: Footer not found.".to_string()))
                                  .as_string()
                                  .unwrap_or("")
                                  .to_string(),
        date: map.get("timeadd").unwrap_or(&Value::String("Error: Date not found.".to_string()))
                                 .as_string()
                                 .unwrap_or("")
                                 .to_string()[0..10]
                                 .to_string()
    };
    post
}


fn get_summary_latest(limit: i32) -> BoxResult<Vec<Summary>> {
    let query = SUMMARY_QUERY_BASE.to_string() +
                " order by id desc limit ?";
    let cols = SUMMARY_QUERY_COL.to_vec()
                                .iter()
                                .map(|x| x.to_string())
                                .collect();

    let inputs = &[Value::Integer(limit.into())];
    
    let query_result: Vec<Summary> = execute_query(query,cols,inputs)?
                                                .iter()
                                                .map(|x| map_to_summary(x))
                                                .collect();
    Ok(query_result)
}


//get the sidebar summaries
fn get_sidebar_summary() -> BoxResult<Vec<Summary>> {
    get_summary_latest(6)
}



fn get_posts_latest(limit: i32) -> BoxResult<Vec<Post>> {
    let query = POST_QUERY_BASE.to_string() +
                    " order by id desc limit ?";
    let cols = POST_QUERY_COL.to_vec()
                             .iter()
                             .map(|x| x.to_string())
                             .collect();

    let inputs = &[Value::Integer(limit.into())];

    let query_result: Vec<Post> = execute_query(query,cols,inputs)?
                                    .iter()
                                    .map(|x| map_to_post(x))
                                    .collect();
    Ok(query_result)
}


fn get_post_by_id(id: i32) -> BoxResult<Vec<Post>> {
    let query = POST_QUERY_BASE.to_string() +
                    " where id=?";
    let cols = POST_QUERY_COL.to_vec()
                             .iter()
                             .map(|x| x.to_string())
                             .collect();
    let inputs = &[Value::Integer(id.into())];
   
    let query_result: Vec<Post> = execute_query(query,cols,inputs)?
                                    .iter()
                                    .map(|x| map_to_post(x))
                                    .collect();
    Ok(query_result)
}

fn vector_unwrap<T, E>(op: Result<Vec<T>,E>) -> Vec<T> {
    match op {
        Ok(x) => x,
        _ => Vec::new()
    }
}


fn create_template<C: Serialize>(parent_content: C, 
                                 parent: &str, 
                                 wrapper_template_name: &str) -> Template {
    
    let sidebar_summaries = vector_unwrap(get_sidebar_summary());
        //we need to do some refactoring to remove this from here

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
    let posts = vector_unwrap(get_posts_latest(5));
    let content = GroupContent {items: posts};
    create_template(content, "blog", "index")
}

#[get("/<id>")]
fn get_post(id: i32) -> Template {
    let posts = vector_unwrap(get_post_by_id(id));
    let content = GroupContent {items: posts};
    create_template(content, "blog", "index")
}


#[get("/browse")]
fn browse_posts() -> Template {
    let summaries: Vec<Summary> = vector_unwrap(get_summary_latest(1000));
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

