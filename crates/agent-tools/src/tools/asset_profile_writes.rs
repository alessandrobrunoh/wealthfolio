//! MCP-only asset profile reads and user-managed asset writes.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use wealthfolio_core::assets::{
    Asset, AssetKind, AssetProfile, NewAsset, QuoteMode, UpdateAssetProfile,
};

use crate::env::AgentEnvironment;
use crate::scope::AgentScope;
use crate::tool::{AgentTool, AgentToolAccess, AgentToolError, AgentToolResult};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIdArgs {
    pub asset_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateManualAssetArgs {
    pub kind: AssetKind,
    pub name: String,
    #[serde(default)]
    pub display_code: Option<String>,
    pub quote_ccy: String,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAssetProfileArgs {
    pub asset_id: String,
    pub name: Option<String>,
    pub display_code: Option<String>,
    pub notes: String,
    pub quote_mode: QuoteMode,
    pub quote_ccy: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetSummary {
    pub id: String,
    pub kind: AssetKind,
    pub name: Option<String>,
    pub display_code: Option<String>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub quote_mode: QuoteMode,
    pub quote_ccy: String,
    pub instrument_type: Option<wealthfolio_core::assets::InstrumentType>,
    pub instrument_symbol: Option<String>,
    pub instrument_exchange_mic: Option<String>,
}

impl From<Asset> for AssetSummary {
    fn from(asset: Asset) -> Self {
        Self {
            id: asset.id,
            kind: asset.kind,
            name: asset.name,
            display_code: asset.display_code,
            notes: asset.notes,
            is_active: asset.is_active,
            quote_mode: asset.quote_mode,
            quote_ccy: asset.quote_ccy,
            instrument_type: asset.instrument_type,
            instrument_symbol: asset.instrument_symbol,
            instrument_exchange_mic: asset.instrument_exchange_mic,
        }
    }
}

fn redact_payload(_: &Value) -> Value {
    serde_json::json!({ "payload": "[redacted]" })
}

fn redact_error_for_audit(_: &AgentToolError) -> String {
    "Asset write failed; error details are redacted from the audit log".to_string()
}

fn is_manual_asset_kind(kind: &AssetKind) -> bool {
    matches!(
        kind,
        AssetKind::Property
            | AssetKind::Vehicle
            | AssetKind::Collectible
            | AssetKind::PreciousMetal
            | AssetKind::PrivateEquity
            | AssetKind::Liability
            | AssetKind::Other
    )
}

pub struct GetAssetProfile;

#[async_trait::async_trait]
impl AgentTool for GetAssetProfile {
    fn name(&self) -> &'static str {
        "get_asset_profile"
    }

    fn description(&self) -> &'static str {
        "Get user-editable descriptive and pricing fields for one asset. \
         Provider configuration and internal metadata are not returned."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "assetId": { "type": "string", "minLength": 1 }
            },
            "required": ["assetId"]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[AgentScope::HoldingsRead]
    }

    fn access_level(&self) -> AgentToolAccess {
        AgentToolAccess::Read
    }

    fn sanitize_error_for_audit(&self, _: &AgentToolError) -> String {
        "Asset profile read failed; error details are redacted from the audit log".to_string()
    }

    async fn call(
        &self,
        env: Arc<dyn AgentEnvironment>,
        args: Value,
    ) -> Result<AgentToolResult, AgentToolError> {
        let args: AssetIdArgs = serde_json::from_value(args)?;
        let profile: AssetProfile = env
            .asset_service()
            .get_asset_profile(&args.asset_id)
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(AssetSummary::from(profile.asset))?,
        })
    }
}

pub struct CreateManualAsset;

#[async_trait::async_trait]
impl AgentTool for CreateManualAsset {
    fn name(&self) -> &'static str {
        "create_manual_asset"
    }

    fn description(&self) -> &'static str {
        "Create a manually priced alternative asset, such as property, a \
         vehicle, or a liability. Market securities and FX instruments are \
         excluded; use the existing symbol-resolution flow for those."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "kind": {
                    "type": "string",
                    "enum": [
                        "PROPERTY", "VEHICLE", "COLLECTIBLE",
                        "PRECIOUS_METAL", "PRIVATE_EQUITY", "LIABILITY", "OTHER"
                    ]
                },
                "name": { "type": "string", "minLength": 1, "maxLength": 200 },
                "displayCode": { "type": ["string", "null"], "maxLength": 64 },
                "quoteCcy": { "type": "string", "minLength": 1, "maxLength": 16 },
                "notes": { "type": ["string", "null"], "maxLength": 4096 }
            },
            "required": ["kind", "name", "quoteCcy"]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[AgentScope::HoldingsWrite]
    }

    fn access_level(&self) -> AgentToolAccess {
        AgentToolAccess::Write
    }

    fn is_additive(&self) -> bool {
        true
    }

    fn sanitize_args_for_audit(&self, args: &Value) -> Value {
        redact_payload(args)
    }

    fn sanitize_error_for_audit(&self, error: &AgentToolError) -> String {
        redact_error_for_audit(error)
    }

    async fn call(
        &self,
        env: Arc<dyn AgentEnvironment>,
        args: Value,
    ) -> Result<AgentToolResult, AgentToolError> {
        let args: CreateManualAssetArgs = serde_json::from_value(args)?;
        if !is_manual_asset_kind(&args.kind) {
            return Err(AgentToolError::InvalidInput(
                "Only manual alternative asset kinds are supported".to_string(),
            ));
        }
        if args.name.trim().is_empty() || args.name.chars().count() > 200 {
            return Err(AgentToolError::InvalidInput(
                "name must contain 1 to 200 characters".to_string(),
            ));
        }
        if args.quote_ccy.trim().is_empty() || args.quote_ccy.chars().count() > 16 {
            return Err(AgentToolError::InvalidInput(
                "quoteCcy must contain 1 to 16 characters".to_string(),
            ));
        }
        let created = env
            .asset_service()
            .create_asset(NewAsset {
                id: None,
                kind: args.kind,
                name: Some(args.name),
                display_code: args.display_code,
                is_active: true,
                quote_mode: QuoteMode::Manual,
                quote_ccy: args.quote_ccy,
                instrument_type: None,
                instrument_symbol: None,
                instrument_exchange_mic: None,
                provider_config: None,
                provider_id: None,
                provider_symbol: None,
                notes: args.notes,
                metadata: None,
            })
            .await
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(AssetSummary::from(created))?,
        })
    }
}

pub struct UpdateAssetProfileTool;

#[async_trait::async_trait]
impl AgentTool for UpdateAssetProfileTool {
    fn name(&self) -> &'static str {
        "update_asset_profile"
    }

    fn description(&self) -> &'static str {
        "Update an asset's name, display code, notes, pricing mode, or quote \
         currency. Instrument identity, provider configuration, and metadata \
         are preserved. This MUTATES valuation data; confirm with the user."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "assetId": { "type": "string", "minLength": 1 },
                "name": { "type": ["string", "null"], "maxLength": 200 },
                "displayCode": { "type": ["string", "null"], "maxLength": 64 },
                "notes": { "type": "string", "maxLength": 4096 },
                "quoteMode": { "type": "string", "enum": ["MARKET", "MANUAL"] },
                "quoteCcy": { "type": "string", "minLength": 1, "maxLength": 16 }
            },
            "required": ["assetId", "name", "displayCode", "notes", "quoteMode", "quoteCcy"]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[AgentScope::HoldingsRead, AgentScope::HoldingsWrite]
    }

    fn access_level(&self) -> AgentToolAccess {
        AgentToolAccess::Write
    }

    fn sanitize_args_for_audit(&self, args: &Value) -> Value {
        redact_payload(args)
    }

    fn sanitize_error_for_audit(&self, error: &AgentToolError) -> String {
        redact_error_for_audit(error)
    }

    async fn call(
        &self,
        env: Arc<dyn AgentEnvironment>,
        args: Value,
    ) -> Result<AgentToolResult, AgentToolError> {
        if args.get("name").is_none() || args.get("displayCode").is_none() {
            return Err(AgentToolError::InvalidInput(
                "name and displayCode must be supplied; use null to clear a field".to_string(),
            ));
        }
        let args: UpdateAssetProfileArgs = serde_json::from_value(args)?;
        if args.quote_ccy.trim().is_empty() || args.quote_ccy.chars().count() > 16 {
            return Err(AgentToolError::InvalidInput(
                "quoteCcy must contain 1 to 16 characters".to_string(),
            ));
        }
        let updated = env
            .asset_service()
            .update_asset_profile(
                &args.asset_id,
                UpdateAssetProfile {
                    name: args.name,
                    display_code: args.display_code,
                    notes: args.notes,
                    kind: None,
                    quote_mode: Some(args.quote_mode),
                    quote_ccy: Some(args.quote_ccy),
                    instrument_type: None,
                    instrument_symbol: None,
                    instrument_exchange_mic: None,
                    provider_config: None,
                    metadata: None,
                },
            )
            .await
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(AssetSummary::from(updated))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_asset_tool_excludes_market_identity_kinds() {
        assert!(is_manual_asset_kind(&AssetKind::Property));
        assert!(is_manual_asset_kind(&AssetKind::Liability));
        assert!(!is_manual_asset_kind(&AssetKind::Investment));
        assert!(!is_manual_asset_kind(&AssetKind::Fx));
    }

    #[test]
    fn asset_mutation_audit_payload_is_redacted() {
        let args = serde_json::json!({
            "assetId": "asset-1",
            "notes": "private notes",
            "quoteCcy": "USD"
        });
        assert_eq!(
            redact_payload(&args),
            serde_json::json!({ "payload": "[redacted]" })
        );
    }
}
