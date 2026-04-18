use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder,
};

use crate::entity::project;

pub struct ProjectRepository;

impl ProjectRepository {
    pub async fn list_paginated<C: ConnectionTrait>(
        db: &C,
        page: u64,
        page_size: u64,
    ) -> Result<(Vec<project::Model>, u64), sea_orm::DbErr> {
        let paginator = project::Entity::find()
            .order_by_desc(project::Column::UpdatedAt)
            .paginate(db, page_size);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page.saturating_sub(1)).await?;
        Ok((items, total))
    }

    pub async fn find_by_id<C: ConnectionTrait>(
        db: &C,
        id: &str,
    ) -> Result<Option<project::Model>, sea_orm::DbErr> {
        project::Entity::find_by_id(id.to_owned()).one(db).await
    }

    pub async fn create<C: ConnectionTrait>(
        db: &C,
        model: project::ActiveModel,
    ) -> Result<project::Model, sea_orm::DbErr> {
        model.insert(db).await
    }

    pub async fn delete<C: ConnectionTrait>(db: &C, id: &str) -> Result<u64, sea_orm::DbErr> {
        let result = project::Entity::delete_many()
            .filter(project::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn update<C: ConnectionTrait>(
        db: &C,
        model: project::ActiveModel,
    ) -> Result<project::Model, sea_orm::DbErr> {
        model.update(db).await
    }

    pub fn active_model_from(model: &project::Model) -> project::ActiveModel {
        project::ActiveModel {
            id: Set(model.id.clone()),
            name: Set(model.name.clone()),
            slug: Set(model.slug.clone()),
            description: Set(model.description.clone()),
            color: Set(model.color.clone()),
            created_at: Set(model.created_at.clone()),
            updated_at: Set(model.updated_at.clone()),
        }
    }
}
