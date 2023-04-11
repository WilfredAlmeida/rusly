#[macro_use]
extern crate rocket;
use rand::Rng;
use rocket::{
    fairing::AdHoc,
    http::hyper::request,
    response::Redirect,
    serde::{json::Json, Deserialize, Serialize},
    Build, Rocket,
};
use rocket_sync_db_pools::{
    database,
    rusqlite::{params, Connection},
};

use std::time::{SystemTime, UNIX_EPOCH};

use regex::Regex;

use url::Url;

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

static HOST_URI: &str = "http://127.0.0.1:8001";

#[post("/shorten", data = "<request_body>")]
async fn shorten_url_handler(request_body: Json<RequestBody>, db: Db) -> Json<ResponseBody> {
    // let db = match Connection::open("urls.db") {
    //     Ok(c) => c,
    //     Err(e) => {
    //         println!("1");
    //         eprintln!("{}", e.to_string());
    //         return Json(ResponseBody {
    //             shortened_url: None,
    //             error: Some(e.to_string()),
    //         });
    //     }
    // };

    // match db.execute(
    //     "CREATE TABLE IF NOT EXISTS urls (
    //     id TEXT(7) PRIMARY KEY,
    //     fullUrl TEXT(512) NOT NULL,
    //     time INTEGER NOT NULL
    // )",
    //     params![],
    // ) {
    //     Ok(result) => {
    //         println!("{}", result)
    //     }
    //     Err(err) => {
    //         println!("TABLE CREATION ERROR");
    //         eprintln!("{}", err.to_string());
    //         return Json(ResponseBody {
    //             shortened_url: None,
    //             error: Some(err.to_string()),
    //         });
    //     }
    // }

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
    // let db = match Connection::open("urls.db") {
    //     Ok(c) => c,
    //     Err(e) => {
    //         println!("1");
    //         eprintln!("{}", e.to_string());
    //         return None;
    //     }
    // };

    let query = format!("SELECT fullUrl FROM urls WHERE id='{}'", murl);
    
    let link_to_redirect_to = match db.run(move |conn|{
        conn.query_row(&query, [], |row| row.get::<_, String>(0))
    }).await {
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
    rocket::build().attach(stage())
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
            .mount("/", routes![index])
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
