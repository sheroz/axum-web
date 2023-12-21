use axum::{extract::State, response::IntoResponse, routing::{get, post}, Router, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::application::repository::user_repository::get_user_by_username;
use crate::shared::state::SharedState;
use jsonwebtoken as jwt;

#[derive(Debug, Serialize, Deserialize)]
struct LoginUser {
    username: String,
    password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn routes() -> Router<SharedState> {
    Router::new()
        .route("/login", post(login_handler))
        .route("/logout", get(logout_handler))
}

async fn login_handler(State(state): State<SharedState>, Json(login): Json<LoginUser>) -> impl IntoResponse {
    tracing::debug!("entered: login_handler()");
    tracing::trace!("login: {:#?}", login);
    if let Some(user) = get_user_by_username(&login.username, &state).await {
        if user.password_hash == login.password_hash {
            let time_now = chrono::Utc::now();
            let jwt_claims = JwtClaims {
                sub: user.id.to_string(),
                iat: time_now.timestamp() as usize,
                exp: (time_now + chrono::Duration::minutes(60)).timestamp() as usize,
            };

            let access_token = jwt::encode(
                &jwt::Header::default(),
                &jwt_claims,
                &jwt::EncodingKey::from_secret(state.config.jwt_secret.as_ref()),
            ).unwrap();

            // use redis::AsyncCommands;
            // let _: ()  = state.redis.sadd("sessions".to_string(), user.id.to_string()).await.unwrap();

            let json = json!({"access_token": access_token, "token_type": "Bearer"});
            return Json(json).into_response()
        }
    }
    StatusCode::FORBIDDEN.into_response()
}

async fn logout_handler(State(_state): State<SharedState>) -> impl IntoResponse {
    tracing::debug!("entered: logout_handler()");
    StatusCode::FORBIDDEN
}
