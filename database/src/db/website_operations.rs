use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryOrder};

use crate::{db::pagination::{PaginatedResult}, models::website::{
    self, ActiveModel, Entity as Website, Model
}};

pub async fn get_websites_by_url(db: &DatabaseConnection, url: &str) -> Result<Option<Model>, DbErr> {
    Website::find_by_id(url).one(db).await
}

pub async fn get_websites(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
    Website::find().all(db).await
}

pub async fn get_websites_by_page(db: &DatabaseConnection, page: u64, page_size: u64) -> Result<PaginatedResult<Model>, DbErr> {
    let paginator = Website::find()
        .order_by(website::Column::LinksCrawled, sea_orm::Order::Desc)
        .paginate(db, page_size);

    let total_pages = paginator.num_pages().await?;
    let total_items = paginator.num_items().await?;
    let items = paginator.fetch_page(page).await?;
        
    Ok(PaginatedResult{
        items,
        total_pages,
        total_items
    })
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
