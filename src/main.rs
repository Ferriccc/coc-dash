mod structs;
mod utils;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};
use tokio::time::{sleep, Duration};

use actix_cors::Cors;
use actix_files as fs;
use actix_web::{
    get,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use structs::{ApiResponse, Battle, ContestResult};
use tera::Tera;

use tokio::sync::Mutex;
use tokio::sync::OnceCell;

static BATTLES: OnceCell<Mutex<Vec<ContestResult>>> = OnceCell::const_new();

async fn fetch_at_intervals() {
    loop {
        let contests = match tokio::task::spawn_blocking(|| utils::get_all_contests()).await {
            Ok(battles) => battles,
            _ => continue,
        };

        { // so that mutex lock drops going out of scope
            let data = BATTLES.get_or_init(|| async { Mutex::new(vec![]) }).await;
            let mut battles = data.lock().await;
            *battles = contests; // Update the global battles list
        }

        sleep(Duration::from_secs(60)).await; // Sleep should be awaited
    }
}

#[get("/")]
async fn homepage(tmpl: web::Data<Tera>) -> impl Responder {
    let contests = BATTLES
        .get_or_init(|| async { Mutex::new(vec![]) })
        .await
        .lock()
        .await
        .clone();
    let mut ctx = tera::Context::new();
    ctx.insert("battles", &contests);
    let rendered = tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

#[get("/battle/{contest_name}")]
async fn standings(tmpl: web::Data<Tera>, contest_name: web::Path<String>) -> impl Responder {
    let contests = BATTLES
        .get_or_init(|| async { Mutex::new(vec![]) })
        .await
        .lock()
        .await
        .clone();

    let contest_name = contest_name.into_inner();
    let mut contests = contests; // Mutable version

    match contests.iter_mut().find(|c| c.contest.name == contest_name) {
        Some(contest_result) => {
            utils::calculate_scores(&mut contest_result.rows);
            let teams = utils::sort_rows_by_team(&mut contest_result.rows);

            let mut ctx = tera::Context::new();
            ctx.insert("teams", &teams);
            ctx.insert("battle", &contest_result);
            let rendered = tmpl.render("standings.html", &ctx).unwrap();
            HttpResponse::Ok().content_type("text/html").body(rendered)
        }
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/yeahfullysecure/{group_code}/{contest_id}")]
async fn add_battle(path: web::Path<(String, String)>) -> impl Responder {
    let (group_code, contest_id) = path.into_inner();

    // Create a Battle struct from the path parameters
    let new_battle = Battle {
        group_code: group_code,
        contest_id: contest_id,
    };

    // Define the path for battles.json in the data folder
    let file_path = Path::new("data/battles.json");

    // Read the current battles from the file
    let mut battles_file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Failed to open battles.json");
        }
    };

    let mut data = String::new();
    if let Err(_) = battles_file.read_to_string(&mut data) {
        return HttpResponse::InternalServerError().body("Failed to open battles.json");
    }

    // Parse the current JSON content
    let mut battles: Vec<Battle> = match serde_json::from_str(&data) {
        Ok(battles) => battles,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Failed to open battles.json");
        }
    };

    // Add the new battle
    battles.push(new_battle);

    // Write the updated battles back to the file
    let battles_json = match serde_json::to_string(&battles) {
        Ok(json) => json,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to serialize the updated data");
        }
    };

    let mut battles_file = match OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)
    {
        Ok(file) => file,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to open battles.json for writing");
        }
    };

    if let Err(_) = battles_file.write_all(battles_json.as_bytes()) {
        return HttpResponse::InternalServerError().body("Failed to write to battles.json");
    }

    HttpResponse::Ok().body("Battle added successfully")
}

#[actix_web::main]
async fn main() {
    // Start fetch_at_intervals() in a background task
    tokio::spawn(fetch_at_intervals());

    let tera = Tera::new("templates/**/*").expect("Failed to initialize Tera");

    println!("Running server...");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(tera.clone()))
            .service(homepage)
            .service(standings)
            .service(add_battle)
            .service(fs::Files::new("/static", "static").show_files_listing())
    })
    .bind("0.0.0.0:8080")
    .expect("failed to bind to given address")
    .run()
    .await
    .expect("failed to start http server");
}
