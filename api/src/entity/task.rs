use sea_orm::entity::prelude::*;

use super::sea_orm_active_enums::TaskPriority;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "tasks")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub project_id: String,
    pub status_id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
    pub position: i64,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub completed_at: Option<String>,
    pub archived_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Project,
    #[sea_orm(
        belongs_to = "super::task_status::Entity",
        from = "Column::StatusId",
        to = "super::task_status::Column::Id",
        on_update = "NoAction",
        on_delete = "Restrict"
    )]
    TaskStatus,
    #[sea_orm(has_many = "super::task_comment::Entity")]
    TaskComment,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::task_status::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskStatus.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
