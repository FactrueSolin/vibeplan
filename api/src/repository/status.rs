use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder,
};

use crate::entity::{task, task_status};

pub struct StatusRepository;

impl StatusRepository {
    pub async fn list_by_project<C: ConnectionTrait>(
        db: &C,
        project_id: &str,
    ) -> Result<Vec<task_status::Model>, sea_orm::DbErr> {
        task_status::Entity::find()
            .filter(task_status::Column::ProjectId.eq(project_id))
            .order_by_asc(task_status::Column::SortOrder)
            .order_by_asc(task_status::Column::CreatedAt)
            .all(db)
            .await
    }

    pub async fn find_by_id<C: ConnectionTrait>(
        db: &C,
        id: &str,
    ) -> Result<Option<task_status::Model>, sea_orm::DbErr> {
        task_status::Entity::find_by_id(id.to_owned()).one(db).await
    }

    pub async fn create<C: ConnectionTrait>(
        db: &C,
        model: task_status::ActiveModel,
    ) -> Result<task_status::Model, sea_orm::DbErr> {
        model.insert(db).await
    }

    pub async fn update<C: ConnectionTrait>(
        db: &C,
        model: task_status::ActiveModel,
    ) -> Result<task_status::Model, sea_orm::DbErr> {
        model.update(db).await
    }

    pub async fn delete<C: ConnectionTrait>(db: &C, id: &str) -> Result<u64, sea_orm::DbErr> {
        let result = task_status::Entity::delete_many()
            .filter(task_status::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn max_sort_order<C: ConnectionTrait>(
        db: &C,
        project_id: &str,
    ) -> Result<Option<i64>, sea_orm::DbErr> {
        task_status::Entity::find()
            .filter(task_status::Column::ProjectId.eq(project_id))
            .order_by_desc(task_status::Column::SortOrder)
            .one(db)
            .await
            .map(|item| item.map(|item| item.sort_order))
    }

    pub async fn count_active_tasks<C: ConnectionTrait>(
        db: &C,
        status_id: &str,
    ) -> Result<u64, sea_orm::DbErr> {
        task::Entity::find()
            .filter(task::Column::StatusId.eq(status_id))
            .filter(task::Column::ArchivedAt.is_null())
            .count(db)
            .await
    }

    pub async fn delete_archived_tasks<C: ConnectionTrait>(
        db: &C,
        status_id: &str,
    ) -> Result<u64, sea_orm::DbErr> {
        let result = task::Entity::delete_many()
            .filter(task::Column::StatusId.eq(status_id))
            .filter(task::Column::ArchivedAt.is_not_null())
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    pub fn active_model_from(model: &task_status::Model) -> task_status::ActiveModel {
        task_status::ActiveModel {
            id: Set(model.id.clone()),
            project_id: Set(model.project_id.clone()),
            name: Set(model.name.clone()),
            color: Set(model.color.clone()),
            sort_order: Set(model.sort_order),
            is_done: Set(model.is_done),
            is_hidden: Set(model.is_hidden),
            created_at: Set(model.created_at.clone()),
            updated_at: Set(model.updated_at.clone()),
        }
    }
}
