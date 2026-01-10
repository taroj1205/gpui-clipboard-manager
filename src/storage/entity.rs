use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "clipboard_entries")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub content: String,
    pub created_at: i64,
    pub content_type: String,
    pub content_hash: String,
    pub text_content: Option<String>,
    pub ocr_text: Option<String>,
    pub image_path: Option<String>,

    pub file_paths: Option<String>,
    pub link_url: Option<String>,
    pub link_title: Option<String>,
    pub link_description: Option<String>,
    pub link_site_name: Option<String>,
    pub source_app_title: Option<String>,
    pub source_exe_path: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
