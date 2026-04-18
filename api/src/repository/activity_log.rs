use sea_orm::{ActiveModelTrait, ConnectionTrait};

use crate::entity::activity_log;

pub struct ActivityLogRepository;

impl ActivityLogRepository {
    pub async fn create<C: ConnectionTrait>(
        db: &C,
        model: activity_log::ActiveModel,
    ) -> Result<activity_log::Model, sea_orm::DbErr> {
        model.insert(db).await
    }
}
