use sea_orm::{ActiveValue::Set, ActiveModelTrait, DbConn, EntityTrait};
use crate::db::website::{ActiveModel, Entity as Website, Model}; 


pub async fn get_websites(db: &DbConn) -> Vec<Model> {
    Website::find().all(db).await.expect("hej")
}

pub async fn add_website(db: &DbConn, url: String, average_ttfb_ms: i32, links_crawled: i32)  {
    let _ = ActiveModel {
        url: Set(url),
        average_ttfb_ms: Set(average_ttfb_ms),
        links_crawled: Set(links_crawled)
    }
    .insert(db)
    .await;
}
