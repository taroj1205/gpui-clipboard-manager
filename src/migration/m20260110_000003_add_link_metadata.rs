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
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ClipboardEntries::Table)
                    .add_column(ColumnDef::new(ClipboardEntries::LinkTitle).string())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ClipboardEntries::Table)
                    .add_column(ColumnDef::new(ClipboardEntries::LinkDescription).string())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ClipboardEntries::Table)
                    .add_column(ColumnDef::new(ClipboardEntries::LinkSiteName).string())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ClipboardEntries::Table)
                    .drop_column(ClipboardEntries::LinkUrl)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ClipboardEntries::Table)
                    .drop_column(ClipboardEntries::LinkTitle)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ClipboardEntries::Table)
                    .drop_column(ClipboardEntries::LinkDescription)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ClipboardEntries::Table)
                    .drop_column(ClipboardEntries::LinkSiteName)
                    .to_owned(),
            )
            .await?;

        Ok(())
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
