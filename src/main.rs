use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};
use lazy_static::lazy_static;
use serde::Serialize;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use phone_data::PhoneData;

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
        Message { code: 0, result: "ok".to_owned(), data: Some(data), success: true }
    }
    pub fn err(message: &str) -> Self {
        Message { code: -1, result: message.to_owned(), data: None, success: false }
    }
}


async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Debug, Deserialize)]
struct IParams {
    phone: String,
}

#[get("/query")]
async fn query_phone(info: web::Query<IParams>) -> impl Responder {
    let params: IParams = info.into_inner();
    let phone = params.phone;
    let msg = match STATE.phone_data.find(&phone) {
        Ok(info) => Message::ok(info),
        Err(_) => Message::err("查询失败")
    };
    HttpResponse::Ok().json(msg)
}

async fn query_phone2(phone: web::Path<String>) -> impl Responder {
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

#[derive(Debug, Deserialize)]
struct DemoPa {
    province: String,
}

#[post("/hey")]
async fn manual_hello(pa: web::Json<DemoPa>) -> impl Responder {
    let _pa = pa.into_inner();
    println!("province is : {}", _pa.province);
    HttpResponse::Ok().body("Hey there!")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(echo)
            .service(manual_hello)
            .service(query_phone)
            .route("/", web::get().to(hello))
            .route("/query2/{phone}", web::get().to(query_phone2))
    }).workers(200)
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}