use database::db::website_operations::get_websites;
use sea_orm::{DatabaseConnection, DbErr};

use crate::websites::dto::WebsiteDto;

pub async fn list_websites(db: &DatabaseConnection) -> Result<Vec<WebsiteDto>, DbErr> {
    let websites = get_websites(db)
        .await?
        .into_iter()
        .map(WebsiteDto::from)
        .collect();

    Ok(websites)
}
