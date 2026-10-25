use crate::profiles::ProfileAccess;

use wealthfolio_ai::{
    AiProvidersResponse, CreateCustomProviderRequest, ListModelsResponse, ProviderApiError,
    SetDefaultProviderRequest, UpdateCustomProviderRequest, UpdateProviderSettingsRequest,
};

use super::error::CommandResult;

#[tauri::command]
pub async fn get_ai_providers(context: ProfileAccess) -> CommandResult<AiProvidersResponse> {
    let context = context.context()?;
    Ok(context.ai_provider_service().get_ai_providers()?)
}

#[tauri::command]
pub async fn update_ai_provider_settings(
    context: ProfileAccess,
    request: UpdateProviderSettingsRequest,
) -> CommandResult<()> {
    let context = context.context()?;
    context
        .ai_provider_service()
        .update_provider_settings(request)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn create_custom_ai_provider(
    context: ProfileAccess,
    request: CreateCustomProviderRequest,
) -> CommandResult<String> {
    let context = context.context()?;
    Ok(context
        .ai_provider_service()
        .create_custom_provider(request)
        .await?)
}

#[tauri::command]
pub async fn update_custom_ai_provider(
    context: ProfileAccess,
    request: UpdateCustomProviderRequest,
) -> CommandResult<()> {
    let context = context.context()?;
    context
        .ai_provider_service()
        .update_custom_provider(request)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_custom_ai_provider(
    context: ProfileAccess,
    provider_id: String,
) -> CommandResult<()> {
    let context = context.context()?;
    context
        .ai_provider_service()
        .delete_custom_provider(&provider_id)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn set_default_ai_provider(
    context: ProfileAccess,
    request: SetDefaultProviderRequest,
) -> CommandResult<()> {
    let context = context.context()?;
    context
        .ai_provider_service()
        .set_default_provider(request)
        .await?;
    Ok(())
}

/// List available models from a provider.
/// Fetches models from the provider's API using backend-stored secrets.
/// Frontend never needs to send API keys - they are retrieved internally.
#[tauri::command]
pub async fn list_ai_models(
    context: ProfileAccess,
    provider_id: String,
) -> Result<ListModelsResponse, ProviderApiError> {
    let context = context.context()?;
    context
        .ai_provider_service()
        .list_models(&provider_id)
        .await
}
