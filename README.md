![](https://cdn.hashnode.com/res/hashnode/image/upload/v1681761478810/703795b8-de70-4b73-9842-2908e232f26b.png)

# Rusly: URL Shortener built using Rust

Check out [Rusly](https://wilfredalmeida.github.io/rusly-ui/)

Rusly is built using the [Rocket](https://rocket.rs/) framework

Check out the [system design of Rusly here](https://blog.wilfredalmeida.com/rusly)

Check out Rusly's simple HTML, CSS UI [repo](https://github.com/WilfredAlmeida/rusly-ui)

# API Endpoints

Let's take a quick look at the available endpoints

| Route | Description | Request Body | Response |
| --- | --- | --- | --- |
| `/` GET | The default home route | *None* | Hello Heckerr |
| `/v1/shorten` POST | Takes in the URL to shorten and returns the shortened URL. | \- `url_to_shorten`: String url to shorten | \- `shortened_url`: A shortened string URL. |
| | | \- `custom_link`: Optional, strictly 7-character alphabetic custom shortened URL string | \- `error`: Error message string | 
| `/<short-url` GET | Permanently redirects to the specified short URL string | *None* | *None* |

# Database

Currently, the SQLite database is used as an embedded database.

## Database Schema

<table><tbody><tr><td colspan="1" rowspan="1"><p><code>id</code></p></td><td colspan="1" rowspan="1"><p>VARCHAR(7) PRIMARY KEY</p></td></tr><tr><td colspan="1" rowspan="1"><p><code>fullUrl</code></p></td><td colspan="1" rowspan="1"><p>VARCHAR(1024) NOT NULL</p></td></tr><tr><td colspan="1" rowspan="1"><p><code>timestamp</code></p></td><td colspan="1" rowspan="1"><p>INTEGER NOT NULL</p></td></tr></tbody></table>

### Schema Description

`id`: The randomly generated string acts as the shortened URL string. This being a primary key ensures that there are no duplicate short URLs.

In the case of custom short URLs, it won't be allowed either.

Since the length of the short URL is 7 characters, there are over 8 billion possible combinations that are sufficient and uniqueness is maintained by the primary key.

`fullUrl`: The full URL string. Is fetched and the user is redirected permanently.

`timestamp`: The UNIX timestamp of entry. It is provided in the Rust code in the insert query. The commit time of the record and this timestamp may vary.

---

# Concurrency

Rocket is multi-threaded by default, it spins up worker threads and divides the workload. More details [here](https://rocket.rs/v0.5-rc/guide/upgrading/#configuration)

### Database Concurrency

Rocket provides [database pools](https://rocket.rs/v0.5-rc/guide/state/#databases) for providing database connection objects via [state](https://rocket.rs/v0.5-rc/guide/state/)

It provides a [rusqlite](https://docs.rs/rusqlite/latest/rusqlite/) connection out of the box, with additional benefits of thread safety, and connection pooling.

---

# Running Locally

1. [Install Rust](https://rustup.rs/)  
    You might need a specific version for Rocket, check the docs once.
    
2. Clone this repo
    
3. Run `cargo run`  
    It'll take some and Rusly will be up & running on `127.0.0.1:8001`
    

## My Versions

Following are the versions I used

```rust
rustc 1.68.2
cargo 1.68.2
```

---

# Contributing

Please refer to the CONTRIBUTING.md file and the open issues for any contribution and communication.

[Get in touch with me](https://links.wilfredalmeida.com/)