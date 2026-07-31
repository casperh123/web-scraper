use database::db::website_operations::{get_websites, get_websites_by_page};
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

pub async fn list_websites_paginated(db: &DatabaseConnection, page: u64, page_size: u64) -> Result<Vec<WebsiteDto>, DbErr> {
    let websites = get_websites_by_page(db, page, page_size)
        .await?
        .items
        .into_iter()
        .map(WebsiteDto::from)
        .collect();

    Ok(websites)
}
