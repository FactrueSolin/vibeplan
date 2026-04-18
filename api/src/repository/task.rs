use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, ConnectionTrait, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder,
};

use crate::{
    dto::task::{ArchivedFilter, SortOrder, TaskListQuery, TaskSortBy},
    entity::{task, task_status, task_tag},
};

pub struct TaskRepository;

impl TaskRepository {
    pub async fn find_by_id<C: ConnectionTrait>(
        db: &C,
        id: &str,
    ) -> Result<Option<task::Model>, sea_orm::DbErr> {
        task::Entity::find_by_id(id.to_owned()).one(db).await
    }

    pub async fn list_by_project_paginated<C: ConnectionTrait>(
        db: &C,
        project_id: &str,
        query: &TaskListQuery,
    ) -> Result<(Vec<task::Model>, u64), sea_orm::DbErr> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).clamp(1, 100);
        let mut select = task::Entity::find().filter(task::Column::ProjectId.eq(project_id));

        if let Some(q) = query
            .q
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            select = select.filter(
                Condition::any()
                    .add(task::Column::Title.contains(q))
                    .add(task::Column::Description.contains(q)),
            );
        }

        if let Some(status_id) = &query.status_id {
            select = select.filter(task::Column::StatusId.eq(status_id));
        }

        if let Some(priority) = &query.priority {
            select = select.filter(task::Column::Priority.eq(priority.clone()));
        }

        if let Some(tag_id) = &query.tag_id {
            let task_ids: Vec<String> = task_tag::Entity::find()
                .filter(task_tag::Column::ProjectId.eq(project_id))
                .filter(task_tag::Column::TagId.eq(tag_id))
                .all(db)
                .await?
                .into_iter()
                .map(|item| item.task_id)
                .collect();

            if task_ids.is_empty() {
                return Ok((Vec::new(), 0));
            }

            select = select.filter(task::Column::Id.is_in(task_ids));
        }

        match query.archived.clone().unwrap_or(ArchivedFilter::Exclude) {
            ArchivedFilter::Exclude => {
                select = select.filter(task::Column::ArchivedAt.is_null());
            }
            ArchivedFilter::Only => {
                select = select.filter(task::Column::ArchivedAt.is_not_null());
            }
            ArchivedFilter::Include => {}
        }

        let (sort_by, sort_order) = (
            query.sort_by.clone().unwrap_or(TaskSortBy::UpdatedAt),
            query.sort_order.clone().unwrap_or(SortOrder::Desc),
        );
        let ascending = sort_order == SortOrder::Asc;
        select = match sort_by {
            TaskSortBy::UpdatedAt => {
                if ascending {
                    select.order_by_asc(task::Column::UpdatedAt)
                } else {
                    select.order_by_desc(task::Column::UpdatedAt)
                }
            }
            TaskSortBy::DueDate => {
                if ascending {
                    select.order_by_asc(task::Column::DueDate)
                } else {
                    select.order_by_desc(task::Column::DueDate)
                }
            }
            TaskSortBy::CreatedAt => {
                if ascending {
                    select.order_by_asc(task::Column::CreatedAt)
                } else {
                    select.order_by_desc(task::Column::CreatedAt)
                }
            }
            TaskSortBy::Position => {
                if ascending {
                    select.order_by_asc(task::Column::Position)
                } else {
                    select.order_by_desc(task::Column::Position)
                }
            }
        };

        let paginator = select.paginate(db, page_size);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page.saturating_sub(1)).await?;
        Ok((items, total))
    }

    pub async fn list_by_project<C: ConnectionTrait>(
        db: &C,
        project_id: &str,
        include_archived: bool,
    ) -> Result<Vec<task::Model>, sea_orm::DbErr> {
        let mut query = task::Entity::find()
            .filter(task::Column::ProjectId.eq(project_id))
            .order_by_asc(task::Column::StatusId)
            .order_by_asc(task::Column::Position);
        if !include_archived {
            query = query.filter(task::Column::ArchivedAt.is_null());
        }
        query.all(db).await
    }

    pub async fn list_by_status<C: ConnectionTrait>(
        db: &C,
        status_id: &str,
        include_archived: bool,
    ) -> Result<Vec<task::Model>, sea_orm::DbErr> {
        let mut query = task::Entity::find()
            .filter(task::Column::StatusId.eq(status_id))
            .order_by_asc(task::Column::Position);
        if !include_archived {
            query = query.filter(task::Column::ArchivedAt.is_null());
        }
        query.all(db).await
    }

    pub async fn list_by_ids<C: ConnectionTrait>(
        db: &C,
        ids: &[String],
    ) -> Result<Vec<task::Model>, sea_orm::DbErr> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        task::Entity::find()
            .filter(task::Column::Id.is_in(ids.iter().cloned()))
            .all(db)
            .await
    }

    pub async fn create<C: ConnectionTrait>(
        db: &C,
        model: task::ActiveModel,
    ) -> Result<task::Model, sea_orm::DbErr> {
        model.insert(db).await
    }

    pub async fn update<C: ConnectionTrait>(
        db: &C,
        model: task::ActiveModel,
    ) -> Result<task::Model, sea_orm::DbErr> {
        model.update(db).await
    }

    pub async fn delete<C: ConnectionTrait>(db: &C, id: &str) -> Result<u64, sea_orm::DbErr> {
        let result = task::Entity::delete_many()
            .filter(task::Column::Id.eq(id))
            .exec(db)
            .await?;
        Ok(result.rows_affected)
    }

    pub async fn max_position<C: ConnectionTrait>(
        db: &C,
        status_id: &str,
    ) -> Result<Option<i64>, sea_orm::DbErr> {
        task::Entity::find()
            .filter(task::Column::StatusId.eq(status_id))
            .order_by_desc(task::Column::Position)
            .one(db)
            .await
            .map(|item| item.map(|item| item.position))
    }

    pub async fn count_archived<C: ConnectionTrait>(
        db: &C,
        project_id: &str,
    ) -> Result<u64, sea_orm::DbErr> {
        task::Entity::find()
            .filter(task::Column::ProjectId.eq(project_id))
            .filter(task::Column::ArchivedAt.is_not_null())
            .count(db)
            .await
    }

    pub async fn count_done<C: ConnectionTrait>(
        db: &C,
        project_id: &str,
    ) -> Result<u64, sea_orm::DbErr> {
        let done_statuses = task_status::Entity::find()
            .filter(task_status::Column::ProjectId.eq(project_id))
            .filter(task_status::Column::IsDone.eq(true))
            .all(db)
            .await?;
        let done_ids: Vec<String> = done_statuses.into_iter().map(|status| status.id).collect();
        if done_ids.is_empty() {
            return Ok(0);
        }

        task::Entity::find()
            .filter(task::Column::ProjectId.eq(project_id))
            .filter(task::Column::ArchivedAt.is_null())
            .filter(task::Column::StatusId.is_in(done_ids))
            .count(db)
            .await
    }

    pub async fn count_active<C: ConnectionTrait>(
        db: &C,
        project_id: &str,
    ) -> Result<u64, sea_orm::DbErr> {
        task::Entity::find()
            .filter(task::Column::ProjectId.eq(project_id))
            .filter(task::Column::ArchivedAt.is_null())
            .count(db)
            .await
    }

    pub fn active_model_from(model: &task::Model) -> task::ActiveModel {
        task::ActiveModel {
            id: Set(model.id.clone()),
            project_id: Set(model.project_id.clone()),
            status_id: Set(model.status_id.clone()),
            title: Set(model.title.clone()),
            description: Set(model.description.clone()),
            priority: Set(model.priority.clone()),
            position: Set(model.position),
            start_date: Set(model.start_date.clone()),
            due_date: Set(model.due_date.clone()),
            completed_at: Set(model.completed_at.clone()),
            archived_at: Set(model.archived_at.clone()),
            created_at: Set(model.created_at.clone()),
            updated_at: Set(model.updated_at.clone()),
        }
    }
}
