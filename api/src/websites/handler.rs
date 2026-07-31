use actix_web::{get, web, HttpResponse};
use crate::{
    AppState, websites::{service}
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/websites")
            .service(list_websites), 
    );
}

#[get("")]
pub async fn list_websites(
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let websites = service::list_websites(&state.db)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(websites))
}
