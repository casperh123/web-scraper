use actix_web::{get, web, HttpResponse};
use crate::{
    websites::service,
    AppState,
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
    match service::list_websites(&state.db).await {
        Ok(websites) => Ok(HttpResponse::Ok().json(websites)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}
