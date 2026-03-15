use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "websites")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub url: String,
    pub average_ttfb_ms: i32,
    pub links_crawled: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
