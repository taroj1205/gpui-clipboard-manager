use sea_orm_migration::prelude::*;

mod m20260110_000001_create_clipboard_entries;
mod m20260110_000002_add_ocr_text;
mod m20260110_000003_add_link_metadata;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260110_000001_create_clipboard_entries::Migration),
            Box::new(m20260110_000002_add_ocr_text::Migration),
            Box::new(m20260110_000003_add_link_metadata::Migration),
        ]
    }
}
