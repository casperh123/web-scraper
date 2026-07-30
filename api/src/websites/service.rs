use database::{db::db::get_websites, models::website};
use sea_orm::{DatabaseConnection, DbErr};

pub async fn list_websites(
    db: &DatabaseConnection,
) -> Result<Vec<website::Model>, DbErr> {
    let websites = get_websites(db).await?;

    Ok(websites)
}
