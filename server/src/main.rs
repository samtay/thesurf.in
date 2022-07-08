use std::{fs::File, io::BufReader};

use actix_web::{
    error::{ErrorInternalServerError, ErrorNotFound},
    get, web, App, HttpResponse, HttpServer, Responder, Result,
};
use lib::msw::{crawler::Spots, forecast::ForecastAPI};
use lib::ui;

// std::io::Error::new(std::io::ErrorKind::Other, e)

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let spot_data = web::Data::new(Spots::new()?);
    HttpServer::new(move || {
        App::new()
            .service(ping)
            .service(spot)
            .service(test_todo_remove)
            .app_data(spot_data.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;
    Ok(())
}

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

#[get("/{spot_id}")]
async fn spot(spot_name: web::Path<String>, spots: web::Data<Spots>) -> Result<impl Responder> {
    let spot_id = spots
        .get_id(&**spot_name)
        .ok_or_else(|| ErrorNotFound("spot name not found"))?;
    let forecast = ForecastAPI::new()
        .get(spot_id)
        .await
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;
    Ok(ui::render::<ui::Terminal>(forecast))
}

#[get("/test/")]
async fn test_todo_remove() -> Result<impl Responder> {
    let file = File::open("./test/msw/forecast.json")?;
    let reader = BufReader::new(file);
    let fc = serde_json::from_reader(reader)?;
    let output = ui::render::<ui::Terminal>(fc);
    Ok(output)
}

// TODO add tests for each endpoint
