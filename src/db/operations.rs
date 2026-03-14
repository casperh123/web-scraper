use sea_orm::{ActiveValue::Set, ActiveModelTrait, DbConn, EntityTrait};
use crate::db::website::{ActiveModel, Entity as Website, Model}; 


pub async fn get_websites(db: &DbConn) -> Vec<Model> {
    Website::find().all(db).await.expect("hej")
}

pub async fn add_website(db: &DbConn, url: String) -> Model {
    ActiveModel {
        url: Set(url),
        ..Default::default()
        
    }
    .insert(db)
    .await
    .expect("Failed to insert Website")
}
