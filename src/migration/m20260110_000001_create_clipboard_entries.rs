use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ClipboardEntries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ClipboardEntries::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ClipboardEntries::Content)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ClipboardEntries::CreatedAt)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ClipboardEntries::ContentType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ClipboardEntries::ContentHash)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ClipboardEntries::TextContent).string())
                    .col(ColumnDef::new(ClipboardEntries::ImagePath).string())
                    .col(ColumnDef::new(ClipboardEntries::FilePaths).string())
                    .col(ColumnDef::new(ClipboardEntries::SourceAppTitle).string())
                    .col(ColumnDef::new(ClipboardEntries::SourceExePath).string())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_clipboard_created_at")
                    .table(ClipboardEntries::Table)
                    .if_not_exists()
                    .col(ClipboardEntries::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_clipboard_content_hash")
                    .table(ClipboardEntries::Table)
                    .if_not_exists()
                    .col(ClipboardEntries::ContentHash)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ClipboardEntries::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ClipboardEntries {
    Table,
    Id,
    Content,
    CreatedAt,
    ContentType,
    ContentHash,
    TextContent,
    ImagePath,
    FilePaths,
    SourceAppTitle,
    SourceExePath,
}
