#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::response::{status, Redirect};
use rocket::{Request};

use std::io::{BufReader, Read};
use std::fs::File;
use std::env;
use std::path::{Path, PathBuf};

#[macro_use]
extern crate lazy_static;
use regex::Regex;
lazy_static! {
  static ref TRAILING_PDF_EXT: Regex = Regex::new("[.]pdf$").unwrap();
  static ref TRAILING_ZIP_EXT: Regex = Regex::new("[.]zip$").unwrap();
  static ref AR5IV_PAPERS_ROOT_DIR: String =
    env::var("AR5IV_PAPERS_ROOT_DIR").unwrap_or_else(|_| String::from("/data/arxmliv"));
}

const NEW_HOME : &str = "https://ar5iv.labs.arxiv.org";

#[get("/")]
async fn about() -> Redirect {
  Redirect::to(NEW_HOME)
}

#[get("/html/<id>")]
async fn get_html(
  id: &str,
) -> Redirect {
  Redirect::to(format!("{}/html/{}",NEW_HOME,id))
}

#[get("/html/<field>/<id>")]
async fn get_field_html(
  field: &str,
  id: &str,
) -> Redirect {
  Redirect::to(format!("{}/{}/{}", NEW_HOME, field, id))
}

#[get("/abs/<field>/<id>")]
async fn abs_field(field: &str, id: &str) -> Redirect {
  Redirect::to(format!("{}/html/{}/{}", NEW_HOME,field,id))
}
#[get("/abs/<id>")]
async fn abs(id: &str) -> Redirect {
  Redirect::to(format!("{}/html/{}", NEW_HOME,id))
}

#[get("/papers/<field>/<id>")]
async fn vanity_style_field(field: &str, id: &str) -> Redirect {
  Redirect::to(format!("{}/html/{}/{}", NEW_HOME,field,id))
}
#[get("/papers/<id>")]
async fn vanity_style(id: &str) -> Redirect {
  Redirect::to(format!("{}/html/{}", NEW_HOME,id))
}

#[get("/pdf/<field>/<id>")]
async fn pdf_field(field: &str, id: String) -> Redirect {
  let id_core: String = (*TRAILING_PDF_EXT.replace(&id, "")).to_owned();
  Redirect::to(format!("{}/html/{}/{}", NEW_HOME,field,id_core))
}
#[get("/pdf/<id>")]
async fn pdf(id: String) -> Redirect {
  let id_core: String = (*TRAILING_PDF_EXT.replace(&id, "")).to_owned();
  Redirect::to(format!("{}/html/{}", NEW_HOME,id_core))
}

#[catch(404)]
fn general_not_found() -> Redirect {
  Redirect::to(NEW_HOME)
}

#[get("/log/<id>")]
async fn get_log(
  id: &str,
) -> Redirect {
  Redirect::to(format!("{}/log/{}", NEW_HOME,id))
}
#[get("/log/<field>/<id>")]
async fn get_field_log(
  field: &str,
  id: &str,
) -> Redirect {
  Redirect::to(format!("{}/log/{}/{}", NEW_HOME,field,id))
}

#[get("/source/<id>")]
async fn get_source_zip(id: &str) -> Option<(ContentType, Vec<u8>)> {
  let id_core: String = (*TRAILING_ZIP_EXT.replace(id, "")).to_owned();
  fetch_zip(None, &id_core)
}
#[get("/source/<field>/<id>", rank = 2)]
async fn get_field_source_zip(field: &str, id: &str) -> Option<(ContentType, Vec<u8>)> {
  let id_core: String = (*TRAILING_ZIP_EXT.replace(id, "")).to_owned();
  fetch_zip(Some(field), &id_core)
}

#[get("/feeling_lucky")]
async fn feeling_lucky() -> Redirect {
  Redirect::to(format!("{}/feeling_lucky", NEW_HOME))
}

#[get("/robots.txt")]
fn robots_txt() -> (ContentType, &'static str) {
  (ContentType::Plain, 
r###"User-agent: *
Disallow: *
"###) }

#[catch(default)]
fn default_catcher(status: Status, req: &Request<'_>) -> status::Custom<String> {
  let msg = format!("{} ({})", status, req.uri());
  status::Custom(status, msg)
}

pub fn fetch_zip(field_opt: Option<&str>, id: &str) -> Option<(ContentType, Vec<u8>)> {
  if let Some(paper_path) = build_source_zip_path(field_opt, id) {
    let zipf = File::open(&paper_path).unwrap();
    let mut reader = BufReader::new(zipf);
    let mut payload = Vec::new();
    reader.read_to_end(&mut payload).ok();
    if payload.is_empty() {
      None
    } else {
      Some((ContentType::ZIP, payload))
    }
  } else {
    None
  }
}
fn build_source_zip_path(field_opt: Option<&str>, id: &str) -> Option<PathBuf> {
  let id_base = &id[0..4];
  let field = field_opt.unwrap_or("");
  let paper_path_str = format!(
    "{}/{}/{}{}/{}{}.zip",
    *AR5IV_PAPERS_ROOT_DIR, id_base, field, id, field, id
  );
  let paper_path = Path::new(&paper_path_str);
  if paper_path.exists() {
    Some(paper_path.to_path_buf())
  } else {
    None
  }
}


#[launch]
fn rocket() -> _ {
  rocket::build()
    .mount(
      "/",
      routes![
        abs,
        abs_field,
        pdf,
        pdf_field,
        vanity_style,
        vanity_style_field,
        get_html,
        get_field_html,
        get_log,
        get_field_log,
        get_source_zip,
        get_field_source_zip,
        about,
        feeling_lucky,
        robots_txt
      ],
    )
    .register("/", catchers![general_not_found, default_catcher])
}
