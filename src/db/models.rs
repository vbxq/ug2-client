use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

pub mod discord_build {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "discord_builds")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(unique)]
        pub build_hash: String,
        pub channel: String,
        pub build_date: DateTimeWithTimeZone,
        pub global_env: Option<Json>,
        pub scripts: Json,
        pub index_scripts: Json,
        pub is_patched: bool,
        pub is_active: bool,
        pub created_at: DateTimeWithTimeZone,
        pub updated_at: DateTimeWithTimeZone,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(has_many = "super::asset_cache::Entity")]
        AssetCache,
    }

    impl Related<super::asset_cache::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::AssetCache.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod asset_cache {
    use super::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
    #[sea_orm(table_name = "asset_cache")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub build_hash: String,
        pub asset_name: String,
        pub content_type: String,
        pub file_size: i64,
        pub is_patched: bool,
        pub created_at: DateTimeWithTimeZone,
        pub last_accessed: DateTimeWithTimeZone,
    }

    // read if cute
    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {
        #[sea_orm(
            belongs_to = "super::discord_build::Entity",
            from = "Column::BuildHash",
            to = "super::discord_build::Column::BuildHash"
        )]
        DiscordBuild,
    }

    impl Related<super::discord_build::Entity> for Entity {
        fn to() -> RelationDef {
            Relation::DiscordBuild.def()
        }
    }

    impl ActiveModelBehavior for ActiveModel {}
}
