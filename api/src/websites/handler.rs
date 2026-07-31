use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use crate::{
    AppState, websites::service::{self}
};


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/websites")
            .service(list_websites)
    );
}

#[derive(Deserialize)]
struct Pagination {
    page: Option<u64>,
    page_size: Option<u64>,
}

#[get("")]
pub async fn list_websites(
    state: web::Data<AppState>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, actix_web::Error> {
    let page = pagination.page.unwrap_or(0);
    let page_size = pagination.page_size.unwrap_or(20);

    let websites =
        service::list_websites_paginated(&state.db, page, page_size)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(websites))
}
