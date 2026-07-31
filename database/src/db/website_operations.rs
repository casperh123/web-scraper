use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait};

use crate::models::website::{
    ActiveModel,
    Entity as Website,
    Model,
};

pub async fn get_websites_by_url(db: &DatabaseConnection, url: &str) -> Result<Option<Model>, DbErr> {
    Website::find_by_id(url).one(db).await
}

pub async fn get_websites(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
    Website::find().all(db).await
}

pub async fn add_website(
    db: &DatabaseConnection,
    url: String,
    average_ttfb_ms: i32,
    links_crawled: i32,
) -> Result<Model, DbErr> {
    ActiveModel {
        url: Set(url),
        average_ttfb_ms: Set(average_ttfb_ms),
        links_crawled: Set(links_crawled),
        ..Default::default()
    }
    .insert(db)
    .await
}
