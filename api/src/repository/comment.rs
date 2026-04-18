use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
    QueryOrder,
};

use crate::entity::task_comment;

pub struct CommentRepository;

impl CommentRepository {
    pub async fn list_by_task<C: ConnectionTrait>(
        db: &C,
        task_id: &str,
    ) -> Result<Vec<task_comment::Model>, sea_orm::DbErr> {
        task_comment::Entity::find()
            .filter(task_comment::Column::TaskId.eq(task_id))
            .order_by_asc(task_comment::Column::CreatedAt)
            .all(db)
            .await
    }

    pub async fn find_by_id<C: ConnectionTrait>(
        db: &C,
        id: &str,
    ) -> Result<Option<task_comment::Model>, sea_orm::DbErr> {
        task_comment::Entity::find_by_id(id.to_owned())
            .one(db)
            .await
    }

    pub async fn create<C: ConnectionTrait>(
        db: &C,
        model: task_comment::ActiveModel,
    ) -> Result<task_comment::Model, sea_orm::DbErr> {
        model.insert(db).await
    }

    pub async fn update<C: ConnectionTrait>(
        db: &C,
        model: task_comment::ActiveModel,
    ) -> Result<task_comment::Model, sea_orm::DbErr> {
        model.update(db).await
    }

    pub async fn delete<C: ConnectionTrait>(
        db: &C,
        comment_id: &str,
    ) -> Result<u64, sea_orm::DbErr> {
        let result = task_comment::Entity::delete_many()
            .filter(task_comment::Column::Id.eq(comment_id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    pub fn active_model_from(model: &task_comment::Model) -> task_comment::ActiveModel {
        task_comment::ActiveModel {
            id: Set(model.id.clone()),
            task_id: Set(model.task_id.clone()),
            author_name: Set(model.author_name.clone()),
            content: Set(model.content.clone()),
            created_at: Set(model.created_at.clone()),
            updated_at: Set(model.updated_at.clone()),
        }
    }
}
