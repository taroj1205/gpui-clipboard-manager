use std::path::Path;

use sea_orm::{
    ColumnTrait, Condition, ConnectOptions, Database, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};

use crate::migration::Migrator;
use crate::storage::entity::{Column, Entity, Model};
use sea_orm_migration::MigratorTrait;

#[cfg(target_os = "windows")]
use crate::clipboard::ClipboardEntry;
#[cfg(target_os = "windows")]
use sea_orm::{ActiveModelTrait, Set};
#[cfg(target_os = "windows")]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_os = "windows")]
use crate::storage::entity::ActiveModel;

pub async fn open_db(path: &Path) -> anyhow::Result<DatabaseConnection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let db_url = sqlite_url(path);
    let options = ConnectOptions::new(db_url);
    let db = Database::connect(options).await?;
    Migrator::up(&db, None).await?;
    Ok(db)
}

#[cfg(target_os = "windows")]
pub async fn load_last_hash(db: &DatabaseConnection) -> anyhow::Result<Option<String>> {
    let hash = Entity::find()
        .select_only()
        .column(Column::ContentHash)
        .order_by_desc(Column::Id)
        .into_tuple::<String>()
        .one(db)
        .await?;
    Ok(hash)
}

pub async fn load_entries_page(
    db: &DatabaseConnection,
    query: Option<&str>,
    offset: u64,
    limit: u64,
) -> anyhow::Result<Vec<Model>> {
    let mut select = Entity::find()
        .order_by_desc(Column::Id)
        .offset(offset)
        .limit(limit);

    if let Some(query) = query {
        let mut condition = Condition::all();
        for token in query.split_whitespace() {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            condition = condition.add(
                Condition::any()
                    .add(Column::Content.contains(token))
                    .add(Column::TextContent.contains(token))
                    .add(Column::OcrText.contains(token))
                    .add(Column::FilePaths.contains(token))
                    .add(Column::LinkUrl.contains(token))
                    .add(Column::LinkTitle.contains(token))
                    .add(Column::LinkDescription.contains(token))
                    .add(Column::LinkSiteName.contains(token))
                    .add(Column::SourceAppTitle.contains(token))
                    .add(Column::SourceExePath.contains(token)),
            );
        }
        select = select.filter(condition);
    }

    let entries = select.all(db).await?;
    Ok(entries)
}

#[cfg(target_os = "windows")]
pub async fn insert_clipboard_entry(
    db: &DatabaseConnection,
    entry: &ClipboardEntry,
) -> anyhow::Result<()> {
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let model = ActiveModel {
        content: Set(entry.content.clone()),
        created_at: Set(created_at),
        content_type: Set(entry.content_type.clone()),
        content_hash: Set(entry.content_hash.clone()),
        text_content: Set(entry.text_content.clone()),
        ocr_text: Set(entry.ocr_text.clone()),
        image_path: Set(entry.image_path.clone()),
        file_paths: Set(entry.file_paths.clone()),
        link_url: Set(entry.link_url.clone()),
        link_title: Set(entry.link_title.clone()),
        link_description: Set(entry.link_description.clone()),
        link_site_name: Set(entry.link_site_name.clone()),
        source_app_title: Set(entry.source_app_title.clone()),
        source_exe_path: Set(entry.source_exe_path.clone()),
        ..Default::default()
    };
    model.insert(db).await?;
    Ok(())
}

fn sqlite_url(path: &Path) -> String {
    let raw = path.to_string_lossy().replace('\\', "/");
    format!("sqlite:///{raw}?mode=rwc")
}
