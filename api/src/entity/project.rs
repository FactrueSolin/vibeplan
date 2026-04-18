use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub color: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::task_status::Entity")]
    TaskStatus,
    #[sea_orm(has_many = "super::task::Entity")]
    Task,
    #[sea_orm(has_many = "super::tag::Entity")]
    Tag,
    #[sea_orm(has_many = "super::activity_log::Entity")]
    ActivityLog,
}

impl Related<super::task_status::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskStatus.def()
    }
}

impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl Related<super::tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tag.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
