use std::{collections::HashMap, future};

use actix_web::{
    error::{ErrorInternalServerError, ErrorNotFound},
    get,
    http::{
        header::{from_one_raw_str, USER_AGENT},
        StatusCode,
    },
    web, App, HttpResponse, HttpServer, Responder, Result,
};
use lib::msw::{
    crawler::Spots,
    forecast::{ForecastAPI, UnitType},
};
use lib::ui;
use serde::Deserialize;

const TERMINAL_USER_AGENTS: [&str; 12] = [
    "aiohttp",
    "curl",
    "fetch",
    "http_get",
    "httpie",
    "lwp-request",
    "openbsd ftp",
    "powershell",
    "python-httpx",
    "python-requests",
    "wget",
    "xh",
];

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let spot_data = web::Data::new(Spots::new()?);
    HttpServer::new(move || {
        App::new()
            .service(index)
            .service(ping)
            .service(rip)
            .service(list_spots)
            .service(get_spot)
            .app_data(spot_data.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;
    Ok(())
}

/// Units option wrapper. Exists for actix query params parsing.
#[derive(Copy, Clone, Debug, Deserialize)]
struct Units {
    units: Option<UnitType>,
}

// TODO maybe simple ascii art for home page? with example calls?
#[get("/")]
async fn index(
    spots: web::Data<Spots>,
    units: web::Query<Units>,
    render: RenderChoice,
) -> impl Responder {
    get_spot_inner("pipeline", units.units, spots, render).await
}

#[get("/ping")]
async fn ping() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

#[get("/rip")]
async fn rip(render: RenderChoice) -> impl Responder {
    render.into_response(ui::Rip)
}

#[get("/{spot_id}")]
async fn get_spot(
    spot_name: web::Path<String>,
    units: web::Query<Units>,
    spots: web::Data<Spots>,
    render: RenderChoice,
) -> Result<HttpResponse> {
    get_spot_inner(spot_name.as_ref(), units.units, spots, render).await
}

async fn get_spot_inner(
    spot_name: impl Into<String>,
    units: Option<UnitType>,
    spots: web::Data<Spots>,
    render: RenderChoice,
) -> Result<HttpResponse> {
    let spot_name = spot_name.into();
    let spot_id = spot_name.parse::<u16>().or_else(|_| {
        spots
            .get_id(&*spot_name)
            .ok_or_else(|| ErrorNotFound("spot name not found"))
    })?;
    let forecast = ForecastAPI::new()
        .units(units)
        .get(spot_id)
        .await
        .map_err(|e| ErrorInternalServerError(e.to_string()))?;
    Ok(render.into_response(forecast))
}

#[get("/spots")]
async fn list_spots(
    spots: web::Data<Spots>,
    search: web::Query<HashMap<String, String>>,
    render: RenderChoice,
) -> impl Responder {
    let mut spot_list = spots.into_vec();
    if let Some(s) = search.keys().next() {
        spot_list.retain(|(name, _)| name.contains(&**s));
    }
    render.into_response(spot_list)
}

enum RenderChoice {
    Terminal,
    Browser,
}

impl actix_web::FromRequest for RenderChoice {
    type Error = actix_web::error::ParseError;
    type Future = std::future::Ready<Result<Self, Self::Error>>;

    #[inline]
    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let header = req.headers().get(USER_AGENT);
        let res = from_one_raw_str(header).map(|user_agent: String| {
            if TERMINAL_USER_AGENTS
                .iter()
                .any(|agent| user_agent.contains(agent))
            {
                RenderChoice::Terminal
            } else {
                RenderChoice::Browser
            }
        });
        future::ready(res)
    }
}

impl RenderChoice {
    fn into_response(self, view: impl Into<ui::View>) -> HttpResponse {
        match self {
            RenderChoice::Terminal => {
                HttpResponse::build(StatusCode::OK).body(ui::render::<ui::Terminal>(view))
            }
            RenderChoice::Browser => HttpResponse::build(StatusCode::OK)
                .content_type("text/html; charset=utf-8")
                .body(ui::render::<ui::Browser>(view)),
        }
    }
}
