use std::collections::{HashMap, HashSet};

use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
    QueryOrder,
};

use crate::entity::{tag, task_tag};

pub struct TagRepository;

impl TagRepository {
    pub async fn list_by_project<C: ConnectionTrait>(
        db: &C,
        project_id: &str,
    ) -> Result<Vec<tag::Model>, sea_orm::DbErr> {
        tag::Entity::find()
            .filter(tag::Column::ProjectId.eq(project_id))
            .order_by_asc(tag::Column::Name)
            .all(db)
            .await
    }

    pub async fn find_by_id<C: ConnectionTrait>(
        db: &C,
        id: &str,
    ) -> Result<Option<tag::Model>, sea_orm::DbErr> {
        tag::Entity::find_by_id(id.to_owned()).one(db).await
    }

    pub async fn find_many_by_ids<C: ConnectionTrait>(
        db: &C,
        project_id: &str,
        ids: &[String],
    ) -> Result<Vec<tag::Model>, sea_orm::DbErr> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        tag::Entity::find()
            .filter(tag::Column::ProjectId.eq(project_id))
            .filter(tag::Column::Id.is_in(ids.iter().cloned()))
            .all(db)
            .await
    }

    pub async fn create<C: ConnectionTrait>(
        db: &C,
        model: tag::ActiveModel,
    ) -> Result<tag::Model, sea_orm::DbErr> {
        model.insert(db).await
    }

    pub async fn update<C: ConnectionTrait>(
        db: &C,
        model: tag::ActiveModel,
    ) -> Result<tag::Model, sea_orm::DbErr> {
        model.update(db).await
    }

    pub async fn delete<C: ConnectionTrait>(db: &C, id: &str) -> Result<u64, sea_orm::DbErr> {
        let result = tag::Entity::delete_many()
            .filter(tag::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn list_task_tags_by_project<C: ConnectionTrait>(
        db: &C,
        project_id: &str,
        task_ids: Option<&[String]>,
    ) -> Result<Vec<task_tag::Model>, sea_orm::DbErr> {
        let mut query = task_tag::Entity::find().filter(task_tag::Column::ProjectId.eq(project_id));
        if let Some(task_ids) = task_ids {
            query = query.filter(task_tag::Column::TaskId.is_in(task_ids.iter().cloned()));
        }
        query.all(db).await
    }

    pub async fn replace_task_tags<C: ConnectionTrait>(
        db: &C,
        project_id: &str,
        task_id: &str,
        tag_ids: &[String],
        created_at: &str,
    ) -> Result<(), sea_orm::DbErr> {
        task_tag::Entity::delete_many()
            .filter(task_tag::Column::ProjectId.eq(project_id))
            .filter(task_tag::Column::TaskId.eq(task_id))
            .exec(db)
            .await?;

        for tag_id in tag_ids {
            task_tag::ActiveModel {
                project_id: Set(project_id.to_owned()),
                task_id: Set(task_id.to_owned()),
                tag_id: Set(tag_id.clone()),
                created_at: Set(created_at.to_owned()),
            }
            .insert(db)
            .await?;
        }

        Ok(())
    }

    pub async fn attach_tag<C: ConnectionTrait>(
        db: &C,
        project_id: &str,
        task_id: &str,
        tag_id: &str,
        created_at: &str,
    ) -> Result<(), sea_orm::DbErr> {
        let exists = task_tag::Entity::find()
            .filter(task_tag::Column::TaskId.eq(task_id))
            .filter(task_tag::Column::TagId.eq(tag_id))
            .one(db)
            .await?
            .is_some();

        if !exists {
            task_tag::ActiveModel {
                project_id: Set(project_id.to_owned()),
                task_id: Set(task_id.to_owned()),
                tag_id: Set(tag_id.to_owned()),
                created_at: Set(created_at.to_owned()),
            }
            .insert(db)
            .await?;
        }

        Ok(())
    }

    pub async fn detach_tag<C: ConnectionTrait>(
        db: &C,
        task_id: &str,
        tag_id: &str,
    ) -> Result<(), sea_orm::DbErr> {
        task_tag::Entity::delete_many()
            .filter(task_tag::Column::TaskId.eq(task_id))
            .filter(task_tag::Column::TagId.eq(tag_id))
            .exec(db)
            .await?;
        Ok(())
    }

    pub fn build_tag_map(task_tags: &[task_tag::Model]) -> HashMap<String, Vec<String>> {
        let mut tag_map: HashMap<String, Vec<String>> = HashMap::new();
        for task_tag in task_tags {
            tag_map
                .entry(task_tag.task_id.clone())
                .or_default()
                .push(task_tag.tag_id.clone());
        }
        tag_map
    }

    pub fn ensure_all_ids_exist(found: &[tag::Model], requested: &[String]) -> bool {
        let found_ids: HashSet<&str> = found.iter().map(|item| item.id.as_str()).collect();
        requested.iter().all(|id| found_ids.contains(id.as_str()))
    }

    pub fn active_model_from(model: &tag::Model) -> tag::ActiveModel {
        tag::ActiveModel {
            id: Set(model.id.clone()),
            project_id: Set(model.project_id.clone()),
            name: Set(model.name.clone()),
            color: Set(model.color.clone()),
            created_at: Set(model.created_at.clone()),
            updated_at: Set(model.updated_at.clone()),
        }
    }
}
