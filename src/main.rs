#[macro_use]
extern crate rocket;
use rand::Rng;
use rocket::{
    fairing::{AdHoc, Fairing, Info, Kind},
    response::Redirect,
    serde::{json::Json, Deserialize, Serialize},
    Build, Rocket, request::{FromRequest, Outcome, self}, Request, http::{HeaderMap, Header}, Response
};
use rocket_sync_db_pools::{
    database,
    rusqlite::{params, Connection},
};

use url::Url;
use std::{time::{SystemTime, UNIX_EPOCH}, convert::Infallible};

use regex::Regex;


/// Catches all OPTION requests in order to get the CORS related Fairing triggered.
#[options("/<_..>")]
fn all_options() {
    /* Intentionally left empty */
}

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, PATCH, PUT, DELETE, HEAD, OPTIONS, GET",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}


#[database("rusqlite")]
struct Db(Connection);

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct RequestBody {
    url_to_shorten: Option<String>,
    custom_link: Option<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ResponseBody {
    shortened_url: Option<String>,
    error: Option<String>,
}
struct RequestHeaders<'h>(&'h HeaderMap<'h>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RequestHeaders<'r> {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let request_headers = request.headers();
        Outcome::Success(RequestHeaders(&request_headers))
    }
}


#[post("/shorten", data = "<request_body>")]
async fn shorten_url_handler(headers: RequestHeaders<'_>, request_body: Json<RequestBody>, db: Db) -> Json<ResponseBody> {
    let host_uri = format!("https://{}",headers.0.get_one("Host").unwrap());


    let url_to_shorten = match &request_body.url_to_shorten {
        Some(s) => {
            if is_url_valid(s.to_string()) {
                s.to_string()
            } else {
                String::from("NULL")
            }
        }
        _ => String::from("NULL"),
    };

    if url_to_shorten == "NULL" {
        return Json(ResponseBody {
            shortened_url: None,
            error: Some(String::from("Invalid URL")),
        });
    }

    let shorten_string = match &request_body.custom_link {
        Some(s) => {
            if s.len() != 7 || !is_custom_link_valid(s) {
                return Json(ResponseBody {
                    shortened_url: None,
                    error: Some(String::from(
                        "custom_link length should be 7 alphabetic characters",
                    )),
                });
            }

            String::from(s)
        }
        _ => generate_shortened_url(7),
    };

    //ToDo: A better way to do this and avoid clone
    let shorten_string_clone = shorten_string.clone();
    let url_to_shorten_clone = url_to_shorten.clone();
    match db
        .run(move |conn| {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            conn.execute(
                "INSERT INTO urls (id,fullUrl,time) VALUES (?1, ?2, ?3)",
                params![shorten_string_clone, url_to_shorten_clone, timestamp],
            )
        })
        .await
    {
        Ok(result) => {
            if result == 1 {
                println!("Data Inserted");
                println!("{}", result);

                return Json(ResponseBody {
                    shortened_url: Some(format!("{}/{}", host_uri, shorten_string)),
                    error: None,
                });
            }
        }
        Err(err) => {
            println!("Insertion Failed");
            eprintln!("{}", err.to_string());

            let error_message = if err.to_string() == "UNIQUE constraint failed: urls.id" {
                String::from("Custom Link Already Exists")
            } else {
                err.to_string()
            };

            return Json(ResponseBody {
                shortened_url: None,
                error: Some(error_message),
            });
        }
    };

    Json(ResponseBody {
        shortened_url: Some(String::from("")),
        error: None,
    })
}

#[get("/<murl>")]
async fn index(murl: String, db: Db) -> Option<Redirect> {
    let query = format!("SELECT fullUrl FROM urls WHERE id='{}'", murl);

    let link_to_redirect_to = match db
        .run(move |conn| conn.query_row(&query, [], |row| row.get::<_, String>(0)))
        .await
    {
        Ok(s) => s,
        Err(e) => {
            println!("SELECT ERROR");
            eprint!("{}", e.to_string());
            return Some(Redirect::permanent("https://google.com/1234qwer"));
        }
    };

    return Some(Redirect::permanent(link_to_redirect_to));
}

#[get("/")]
async fn index_default() -> String {
    return String::from("Hello Heckerr");
}


#[launch]
fn rocket() -> _ {
    rocket::build().attach(stage()).attach(Cors)
}

async fn init_db(rocket: Rocket<Build>) -> Rocket<Build> {
    Db::get_one(&rocket)
        .await
        .expect("database mounted")
        .run(|conn| {
            conn.execute(
                r#"CREATE TABLE IF NOT EXISTS urls (
                id TEXT(7) PRIMARY KEY,
                fullUrl TEXT(512) NOT NULL,
                time INTEGER NOT NULL
            )"#,
                params![],
            )
        })
        .await
        .expect("can init rusqlite DB");

    rocket
}

pub fn stage() -> AdHoc {

    AdHoc::on_ignite("Rusqlite Stage", |rocket| async {
        rocket
            .attach(Db::fairing())
            .attach(AdHoc::on_ignite("Rusqlite Init", init_db))
            .mount("/", routes![index_default,index])
            .mount("/v1", routes![shorten_url_handler])
    })
}

//Regex checkcing of custom link
fn is_custom_link_valid(link_param: &str) -> bool {
    let re = Regex::new(r"[^a-zA-Z-]+").unwrap();
    return !re.is_match(link_param);
}

fn generate_shortened_url(length: usize) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

    (0..length)
        .map(|_| {
            let index = rand::thread_rng().gen_range(1..CHARSET.len());
            CHARSET[index] as char
        })
        .collect()
}

fn is_url_valid(url_str: String) -> bool {
    match Url::parse(&url_str) {
        Ok(_) => true,
        Err(_) => false,
    }
}
