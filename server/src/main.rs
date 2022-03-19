use actix_web::{
    error::ErrorInternalServerError, get, web, App, HttpResponse, HttpServer, Responder, Result,
};
use lib::msw::forecast::ForecastAPI;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello).service(spot))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[get("/{spot_id}")]
async fn spot(path: web::Path<u16>) -> Result<impl Responder> {
    let spot_id = path.into_inner();
    // TODO use a custom error enum for 404s on missing spot names
    // https://actix.rs/docs/errors/
    let forecast = ForecastAPI::new()
        .get(spot_id)
        .await
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;
    Ok(format!("{:#?}", forecast))
}
