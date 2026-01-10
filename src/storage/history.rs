use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectOptions, Database, DatabaseConnection,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};

use crate::migration::Migrator;
use crate::storage::entity::{ActiveModel, Column, Entity, Model};
use sea_orm_migration::MigratorTrait;

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

pub async fn load_recent_entries(
    db: &DatabaseConnection,
    limit: u64,
) -> anyhow::Result<Vec<Model>> {
    let entries = Entity::find()
        .order_by_desc(Column::Id)
        .limit(limit)
        .all(db)
        .await?;
    Ok(entries)
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
                    .add(Column::SourceAppTitle.contains(token))
                    .add(Column::SourceExePath.contains(token)),
            );
        }
        select = select.filter(condition);
    }

    let entries = select.all(db).await?;
    Ok(entries)
}

pub async fn insert_clipboard_entry(
    db: &DatabaseConnection,
    content_type: &str,
    content_hash: &str,
    content: &str,
    text_content: Option<&str>,
    ocr_text: Option<&str>,
    image_path: Option<&str>,
    file_paths: Option<&str>,

    source_app_title: Option<&str>,
    source_exe_path: Option<&str>,
) -> anyhow::Result<()> {
    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let model = ActiveModel {
        content: Set(content.to_string()),
        created_at: Set(created_at),
        content_type: Set(content_type.to_string()),
        content_hash: Set(content_hash.to_string()),
        text_content: Set(text_content.map(str::to_string)),
        ocr_text: Set(ocr_text.map(str::to_string)),
        image_path: Set(image_path.map(str::to_string)),
        file_paths: Set(file_paths.map(str::to_string)),

        source_app_title: Set(source_app_title.map(str::to_string)),
        source_exe_path: Set(source_exe_path.map(str::to_string)),
        ..Default::default()
    };
    model.insert(db).await?;
    Ok(())
}

fn sqlite_url(path: &Path) -> String {
    let raw = path.to_string_lossy().replace('\\', "/");
    format!("sqlite:///{raw}?mode=rwc")
}
