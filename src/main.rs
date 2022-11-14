use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use lazy_static::lazy_static;
use serde::Serialize;
use serde_derive::Serialize;

use crate::lib::PhoneData;

mod lib;

struct AppState {
    pub phone_data: PhoneData,
}

lazy_static! {
    static ref STATE: AppState  = AppState {
        phone_data: PhoneData::new().unwrap(),
    };
}


#[derive(Debug, Serialize)]
struct Message<T>
    where T: Serialize
{
    code: i32,
    data: Option<T>,
    success: bool,
    result: String,
}

impl<T: Serialize> Message<T> {
    pub fn ok(data: T) -> Self {
        Message { code: 1, result: "ok".to_owned(), data: Some(data), success: true }
    }
    pub fn err(message: &str) -> Self {
        Message { code: -1, result: message.to_owned(), data: None, success: false }
    }
}


#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/query/{phone}")]
async fn query_phone(phone: web::Path<String>) -> impl Responder {
    let str = phone.into_inner();
    let msg = match STATE.phone_data.find(&str) {
        Ok(info) => Message::ok(info),
        Err(_) => Message::err("查询失败")
    };
    HttpResponse::Ok().json(msg)
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(query_phone)
            .route("/hey", web::get().to(manual_hello))
    }).workers(200)
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}