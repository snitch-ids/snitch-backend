use crate::api::AppState;
use crate::errors::APIError;
use crate::model::user::UserID;
use crate::persistence::redis::NotificationSettings;
use actix_identity::Identity;
use actix_web::{get, post, services, web, Responder};
use log::info;

#[post("/notification_settings")]
pub(crate) async fn set_notification_settings(
    id: Identity,
    notification_settings: web::Json<NotificationSettings>,
    state: web::Data<AppState>,
) -> Result<(), APIError> {
    info!("generate new notification_settings request");
    let user_id: UserID = id.id().unwrap().into();
    state
        .persist
        .lock()
        .await
        .set_notification_settings(&user_id, notification_settings.into_inner())
        .await;
    Ok(())
}

#[get("/notification_settings")]
pub(crate) async fn get_notification_settings(
    id: Identity,
    state: web::Data<AppState>,
) -> Result<impl Responder, APIError> {
    info!("get notification_settings request");
    let user_id: UserID = id.id().unwrap().into();
    let notification_settings = state
        .persist
        .lock()
        .await
        .get_notification_settings(&user_id)
        .await;
    Ok(web::Json(notification_settings))
}

pub fn get_notification_services() -> (get_notification_settings, set_notification_settings) {
    services![get_notification_settings, set_notification_settings]
}
