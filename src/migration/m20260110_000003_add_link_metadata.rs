use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ClipboardEntries::Table)
                    .add_column(ColumnDef::new(ClipboardEntries::LinkUrl).string())
                    .add_column(ColumnDef::new(ClipboardEntries::LinkTitle).string())
                    .add_column(ColumnDef::new(ClipboardEntries::LinkDescription).string())
                    .add_column(ColumnDef::new(ClipboardEntries::LinkSiteName).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ClipboardEntries::Table)
                    .drop_column(ClipboardEntries::LinkUrl)
                    .drop_column(ClipboardEntries::LinkTitle)
                    .drop_column(ClipboardEntries::LinkDescription)
                    .drop_column(ClipboardEntries::LinkSiteName)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ClipboardEntries {
    Table,
    LinkUrl,
    LinkTitle,
    LinkDescription,
    LinkSiteName,
}
