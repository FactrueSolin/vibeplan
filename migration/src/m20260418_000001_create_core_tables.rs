use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Projects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Projects::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Projects::Name).string().not_null())
                    .col(ColumnDef::new(Projects::Slug).string().not_null())
                    .col(ColumnDef::new(Projects::Description).text())
                    .col(
                        ColumnDef::new(Projects::Color)
                            .string()
                            .not_null()
                            .default("#2563eb"),
                    )
                    .col(ColumnDef::new(Projects::CreatedAt).string().not_null())
                    .col(ColumnDef::new(Projects::UpdatedAt).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_projects_slug")
                    .table(Projects::Table)
                    .col(Projects::Slug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_projects_updated_at")
                    .table(Projects::Table)
                    .col((Projects::UpdatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TaskStatuses::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskStatuses::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TaskStatuses::ProjectId).string().not_null())
                    .col(ColumnDef::new(TaskStatuses::Name).string().not_null())
                    .col(ColumnDef::new(TaskStatuses::Color).string().not_null())
                    .col(
                        ColumnDef::new(TaskStatuses::SortOrder)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TaskStatuses::IsDone)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(TaskStatuses::IsHidden)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(TaskStatuses::CreatedAt).string().not_null())
                    .col(ColumnDef::new(TaskStatuses::UpdatedAt).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_statuses_project")
                            .from(TaskStatuses::Table, TaskStatuses::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_task_statuses_project_name")
                    .table(TaskStatuses::Table)
                    .col(TaskStatuses::ProjectId)
                    .col(TaskStatuses::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_task_statuses_project_id_id")
                    .table(TaskStatuses::Table)
                    .col(TaskStatuses::ProjectId)
                    .col(TaskStatuses::Id)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_task_statuses_project_sort")
                    .table(TaskStatuses::Table)
                    .col(TaskStatuses::ProjectId)
                    .col(TaskStatuses::SortOrder)
                    .col(TaskStatuses::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Tasks::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tasks::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Tasks::ProjectId).string().not_null())
                    .col(ColumnDef::new(Tasks::StatusId).string().not_null())
                    .col(ColumnDef::new(Tasks::Title).string().not_null())
                    .col(ColumnDef::new(Tasks::Description).text())
                    .col(ColumnDef::new(Tasks::Priority).string())
                    .col(ColumnDef::new(Tasks::Position).big_integer().not_null())
                    .col(ColumnDef::new(Tasks::StartDate).string())
                    .col(ColumnDef::new(Tasks::DueDate).string())
                    .col(ColumnDef::new(Tasks::CompletedAt).string())
                    .col(ColumnDef::new(Tasks::ArchivedAt).string())
                    .col(ColumnDef::new(Tasks::CreatedAt).string().not_null())
                    .col(ColumnDef::new(Tasks::UpdatedAt).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tasks_project")
                            .from(Tasks::Table, Tasks::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tasks_project_status")
                            .from_tbl(Tasks::Table)
                            .from_col(Tasks::ProjectId)
                            .from_col(Tasks::StatusId)
                            .to_tbl(TaskStatuses::Table)
                            .to_col(TaskStatuses::ProjectId)
                            .to_col(TaskStatuses::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .check(
                        Expr::col(Tasks::Priority)
                            .is_null()
                            .or(Expr::col(Tasks::Priority)
                                .is_in(["low", "medium", "high", "urgent"])),
                    )
                    .check(
                        Expr::col(Tasks::StartDate)
                            .is_null()
                            .or(Expr::col(Tasks::DueDate).is_null())
                            .or(Expr::col(Tasks::StartDate).lte(Expr::col(Tasks::DueDate))),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_tasks_project_id_id")
                    .table(Tasks::Table)
                    .col(Tasks::ProjectId)
                    .col(Tasks::Id)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_tasks_active_board \
                 ON tasks(project_id, status_id, position ASC) WHERE archived_at IS NULL;",
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS idx_tasks_active_updated_at \
                 ON tasks(project_id, updated_at DESC) WHERE archived_at IS NULL;",
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_project_due_date")
                    .table(Tasks::Table)
                    .col(Tasks::ProjectId)
                    .col(Tasks::DueDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tasks_project_archived_at")
                    .table(Tasks::Table)
                    .col(Tasks::ProjectId)
                    .col(Tasks::ArchivedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TaskComments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TaskComments::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TaskComments::TaskId).string().not_null())
                    .col(ColumnDef::new(TaskComments::AuthorName).string().not_null())
                    .col(ColumnDef::new(TaskComments::Content).text().not_null())
                    .col(ColumnDef::new(TaskComments::CreatedAt).string().not_null())
                    .col(ColumnDef::new(TaskComments::UpdatedAt).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_comments_task")
                            .from(TaskComments::Table, TaskComments::TaskId)
                            .to(Tasks::Table, Tasks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_task_comments_task_created_at")
                    .table(TaskComments::Table)
                    .col(TaskComments::TaskId)
                    .col(TaskComments::CreatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Tags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tags::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(Tags::ProjectId).string().not_null())
                    .col(ColumnDef::new(Tags::Name).string().not_null())
                    .col(ColumnDef::new(Tags::Color).string().not_null())
                    .col(ColumnDef::new(Tags::CreatedAt).string().not_null())
                    .col(ColumnDef::new(Tags::UpdatedAt).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tags_project")
                            .from(Tags::Table, Tags::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_tags_project_name")
                    .table(Tags::Table)
                    .col(Tags::ProjectId)
                    .col(Tags::Name)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_tags_project_id_id")
                    .table(Tags::Table)
                    .col(Tags::ProjectId)
                    .col(Tags::Id)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tags_project_updated_at")
                    .table(Tags::Table)
                    .col(Tags::ProjectId)
                    .col((Tags::UpdatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TaskTags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TaskTags::ProjectId).string().not_null())
                    .col(ColumnDef::new(TaskTags::TaskId).string().not_null())
                    .col(ColumnDef::new(TaskTags::TagId).string().not_null())
                    .col(ColumnDef::new(TaskTags::CreatedAt).string().not_null())
                    .primary_key(Index::create().col(TaskTags::TaskId).col(TaskTags::TagId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_tags_project")
                            .from(TaskTags::Table, TaskTags::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_tags_task")
                            .from_tbl(TaskTags::Table)
                            .from_col(TaskTags::ProjectId)
                            .from_col(TaskTags::TaskId)
                            .to_tbl(Tasks::Table)
                            .to_col(Tasks::ProjectId)
                            .to_col(Tasks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_task_tags_tag")
                            .from_tbl(TaskTags::Table)
                            .from_col(TaskTags::ProjectId)
                            .from_col(TaskTags::TagId)
                            .to_tbl(Tags::Table)
                            .to_col(Tags::ProjectId)
                            .to_col(Tags::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_task_tags_project_tag")
                    .table(TaskTags::Table)
                    .col(TaskTags::ProjectId)
                    .col(TaskTags::TagId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ActivityLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ActivityLogs::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ActivityLogs::ProjectId).string().not_null())
                    .col(ColumnDef::new(ActivityLogs::TaskId).string())
                    .col(ColumnDef::new(ActivityLogs::EventType).string().not_null())
                    .col(ColumnDef::new(ActivityLogs::PayloadJson).text().not_null())
                    .col(ColumnDef::new(ActivityLogs::CreatedAt).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_activity_logs_project")
                            .from(ActivityLogs::Table, ActivityLogs::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_activity_logs_task")
                            .from(ActivityLogs::Table, ActivityLogs::TaskId)
                            .to(Tasks::Table, Tasks::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_activity_logs_project_created_at")
                    .table(ActivityLogs::Table)
                    .col(ActivityLogs::ProjectId)
                    .col((ActivityLogs::CreatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_activity_logs_task_created_at")
                    .table(ActivityLogs::Table)
                    .col(ActivityLogs::TaskId)
                    .col((ActivityLogs::CreatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ActivityLogs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TaskTags::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tags::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TaskComments::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tasks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TaskStatuses::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
    Name,
    Slug,
    Description,
    Color,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TaskStatuses {
    Table,
    Id,
    ProjectId,
    Name,
    Color,
    SortOrder,
    IsDone,
    IsHidden,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Tasks {
    Table,
    Id,
    ProjectId,
    StatusId,
    Title,
    Description,
    Priority,
    Position,
    StartDate,
    DueDate,
    CompletedAt,
    ArchivedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TaskComments {
    Table,
    Id,
    TaskId,
    AuthorName,
    Content,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Tags {
    Table,
    Id,
    ProjectId,
    Name,
    Color,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum TaskTags {
    Table,
    ProjectId,
    TaskId,
    TagId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum ActivityLogs {
    Table,
    Id,
    ProjectId,
    TaskId,
    EventType,
    PayloadJson,
    CreatedAt,
}
