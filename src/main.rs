#[macro_use]
extern crate rocket;
use rocket::{serde::{Serialize, Deserialize, json::Json}, Response};

use rusqlite::{Connection, Result};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct RequestBody{
    url_to_shorten: String,
    custom_link: Option<String>
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ResponseBody{
    shortened_url: Option<String>,
    error: Option<String>
}

#[post("/shorten",data = "<request_body>")]
fn shorten_url_handler(request_body: Json<RequestBody>) -> Json<ResponseBody> {

    let mut db = match Connection::open("urls.db"){
        Ok(c) => c,
        Err(e) => {
            println!("1");
            eprintln!("{}", e.to_string());
            return Json(ResponseBody {shortened_url: None, error: Some(e.to_string())})
        }
    };

    match db.execute("CREATE TABLE IF NOT EXISTS urls (
        id TEXT(7) PRIMARY KEY,
        fullUrl TEXT(512) NOT NULL,
        time INTEGER NOT NULL
    )",()) {
        Ok(result) =>{
            println!("{}", result)
        },
        Err(err)=>{
            println!("2");
            eprintln!("{}",err.to_string());
            return Json(ResponseBody {shortened_url: None, error: Some(err.to_string())})
        }
    }


    Json(ResponseBody { shortened_url: Some(request_body.url_to_shorten.to_string()), error: None })
}


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}


#[launch]
fn rocket() -> _ {
    rocket::build()
    .mount("/", routes![index])
    .mount("/v1", routes![shorten_url_handler])
}