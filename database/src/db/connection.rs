use sea_orm::{
    Database,
    DatabaseConnection,
    DbErr,
};
use tokio::sync::OnceCell;

static DB: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn connection() -> Result<&'static DatabaseConnection, DbErr> {
    DB.get_or_try_init(|| async {
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let db = Database::connect(database_url).await?;
        db.get_schema_registry("database::models::*").sync(&db).await?;
        Ok(db)
    })
    .await
}
