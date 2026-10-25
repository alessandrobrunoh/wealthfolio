use std::sync::Arc;

use crate::{error::ApiResult, main_lib::AppState};
use axum::{
    extract::Path,
    routing::{delete, get, post, put},
    Json, Router,
};
use wealthfolio_ai::{
    AiProvidersResponse, CreateCustomProviderRequest, ListModelsResponse,
    SetDefaultProviderRequest, UpdateCustomProviderRequest, UpdateProviderSettingsRequest,
};

async fn get_ai_providers(
    axum::Extension(state): axum::Extension<Arc<AppState>>,
) -> ApiResult<Json<AiProvidersResponse>> {
    let response = state.ai_provider_service.get_ai_providers()?;
    Ok(Json(response))
}

async fn update_provider_settings(
    axum::Extension(state): axum::Extension<Arc<AppState>>,
    Json(request): Json<UpdateProviderSettingsRequest>,
) -> ApiResult<Json<()>> {
    state
        .ai_provider_service
        .update_provider_settings(request)
        .await?;
    Ok(Json(()))
}

async fn create_custom_provider(
    axum::Extension(state): axum::Extension<Arc<AppState>>,
    Json(request): Json<CreateCustomProviderRequest>,
) -> ApiResult<Json<String>> {
    let provider_id = state
        .ai_provider_service
        .create_custom_provider(request)
        .await?;
    Ok(Json(provider_id))
}

async fn update_custom_provider(
    axum::Extension(state): axum::Extension<Arc<AppState>>,
    Json(request): Json<UpdateCustomProviderRequest>,
) -> ApiResult<Json<()>> {
    state
        .ai_provider_service
        .update_custom_provider(request)
        .await?;
    Ok(Json(()))
}

async fn delete_custom_provider(
    axum::Extension(state): axum::Extension<Arc<AppState>>,
    Path(provider_id): Path<String>,
) -> ApiResult<Json<()>> {
    state
        .ai_provider_service
        .delete_custom_provider(&provider_id)
        .await?;
    Ok(Json(()))
}

async fn set_default_provider(
    axum::Extension(state): axum::Extension<Arc<AppState>>,
    Json(request): Json<SetDefaultProviderRequest>,
) -> ApiResult<Json<()>> {
    state
        .ai_provider_service
        .set_default_provider(request)
        .await?;
    Ok(Json(()))
}

/// List available models from a provider.
/// Fetches models from the provider's API using backend-stored secrets.
/// Frontend never needs to send API keys - they are retrieved internally.
async fn list_models(
    axum::Extension(state): axum::Extension<Arc<AppState>>,
    Path(provider_id): Path<String>,
) -> ApiResult<Json<ListModelsResponse>> {
    let response = state.ai_provider_service.list_models(&provider_id).await?;
    Ok(Json(response))
}

pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/ai/providers", get(get_ai_providers))
        .route("/ai/providers/settings", put(update_provider_settings))
        .route(
            "/ai/providers/custom",
            post(create_custom_provider).put(update_custom_provider),
        )
        .route(
            "/ai/providers/custom/{provider_id}",
            delete(delete_custom_provider),
        )
        .route("/ai/providers/default", post(set_default_provider))
        .route("/ai/providers/{provider_id}/models", get(list_models))
}
