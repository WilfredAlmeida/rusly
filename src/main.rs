#[macro_use]
extern crate rocket;
use rocket::{
    response::Redirect,
    serde::{json::Json, Deserialize, Serialize},
};

use rusqlite::{Connection};

use rand::distributions::{Alphanumeric, DistString};

use std::time::{SystemTime, UNIX_EPOCH};

use regex::Regex;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct RequestBody {
    url_to_shorten: String,
    custom_link: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ResponseBody {
    shortened_url: Option<String>,
    error: Option<String>,
}

static HOST_URI: &str = "http://127.0.0.1:8001";

#[post("/shorten", data = "<request_body>")]
fn shorten_url_handler(request_body: Json<RequestBody>) -> Json<ResponseBody> {
    let db = match Connection::open("urls.db") {
        Ok(c) => c,
        Err(e) => {
            println!("1");
            eprintln!("{}", e.to_string());
            return Json(ResponseBody {
                shortened_url: None,
                error: Some(e.to_string()),
            });
        }
    };

    match db.execute(
        "CREATE TABLE IF NOT EXISTS urls (
        id TEXT(7) PRIMARY KEY,
        fullUrl TEXT(512) NOT NULL,
        time INTEGER NOT NULL
    )",
        (),
    ) {
        Ok(result) => {
            println!("{}", result)
        }
        Err(err) => {
            println!("TABLE CREATION ERROR");
            eprintln!("{}", err.to_string());
            return Json(ResponseBody {
                shortened_url: None,
                error: Some(err.to_string()),
            });
        }
    }

    let shorten_string = match &request_body.custom_link{
        Some(s)=> {

            if s.len() != 7 || !is_custom_link_valid(s) {
                
                return Json(ResponseBody {
                    shortened_url: None,
                    error: Some(String::from("custom_link length should be 7 alphabetic characters")),
                });
            }

            String::from(s)
        },
        _=>Alphanumeric.sample_string(&mut rand::thread_rng(), 7)
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    match db.execute(
        "INSERT INTO urls (id,fullUrl,time) VALUES (?1, ?2, ?3)",
        (&shorten_string, &request_body.url_to_shorten, timestamp),
    ) {
        Ok(result) => {
            if result == 1 {
                println!("Data Inserted");
                println!("{}", result);

                return Json(ResponseBody {
                    shortened_url: Some(format!("{}/{}", HOST_URI, shorten_string)),
                    error: None,
                });
            }
        }
        Err(err) => {
            println!("Insertion Failed");
            eprintln!("{}", err.to_string());

            let error_message = if err.to_string() == "UNIQUE constraint failed: urls.id" {
                String::from("Custom Link Already Exists")
            } else {err.to_string()};

            return Json(ResponseBody {
                shortened_url: None,
                error: Some(error_message),
            }); 
        }
    };

    Json(ResponseBody {
        shortened_url: Some(request_body.url_to_shorten.to_string()),
        error: None,
    })
}

#[get("/<murl>")]
fn index(murl: String) -> Option<Redirect> {
    
    let db = match Connection::open("urls.db") {
        Ok(c) => c,
        Err(e) => {
            println!("1");
            eprintln!("{}", e.to_string());
            return None;
        }
    };

    let query = format!("SELECT fullUrl FROM urls WHERE id='{}'",murl);
    let link_to_redirect_to = match db.query_row(&query, [], |row| {
        row.get::<_,String>(0)
    }) {
        Ok(s) => s,
        Err(e) => {
            println!("SELECT ERROR");
            eprint!("{}", e.to_string());
            return Some(Redirect::permanent("https://google.com/1234qwer"));
        }
    };

    return Some(Redirect::permanent(link_to_redirect_to));
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .mount("/v1", routes![shorten_url_handler])
}

//Regex checkcing of custom link
fn is_custom_link_valid(link_param: &str) -> bool {
    let re = Regex::new(r"[^a-zA-Z0-9-]+").unwrap();
    return !re.is_match(link_param);
}