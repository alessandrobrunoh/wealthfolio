//! MCP-only account create/update tools.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wealthfolio_core::accounts::{account_types, Account, AccountUpdate, NewAccount, TrackingMode};

use crate::env::AgentEnvironment;
use crate::scope::AgentScope;
use crate::tool::{AgentTool, AgentToolAccess, AgentToolError, AgentToolResult};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountArgs {
    pub name: String,
    pub account_type: String,
    pub currency: String,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub is_default: Option<bool>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub tracking_mode: Option<TrackingMode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccountArgs {
    pub account_id: String,
    pub name: String,
    pub account_type: String,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub clear_group: bool,
    #[serde(default)]
    pub is_default: Option<bool>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub is_archived: Option<bool>,
    #[serde(default)]
    pub tracking_mode: Option<TrackingMode>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountWriteResult {
    pub id: String,
    pub name: String,
    pub account_type: String,
    pub group: Option<String>,
    pub currency: String,
    pub is_default: bool,
    pub is_active: bool,
    pub is_archived: bool,
    pub tracking_mode: TrackingMode,
}

impl From<Account> for AccountWriteResult {
    fn from(account: Account) -> Self {
        Self {
            id: account.id,
            name: account.name,
            account_type: account.account_type,
            group: account.group,
            currency: account.currency,
            is_default: account.is_default,
            is_active: account.is_active,
            is_archived: account.is_archived,
            tracking_mode: account.tracking_mode,
        }
    }
}

fn valid_account_type(account_type: &str) -> bool {
    matches!(
        account_type,
        account_types::SECURITIES
            | account_types::CASH
            | account_types::CREDIT_CARD
            | account_types::CRYPTOCURRENCY
    )
}

fn validate_account_name(name: &str) -> Result<(), AgentToolError> {
    if name.trim().is_empty() || name.chars().count() > 200 {
        return Err(AgentToolError::InvalidInput(
            "name must contain 1 to 200 characters".to_string(),
        ));
    }
    Ok(())
}

fn redact_payload(_: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "payload": "[redacted]" })
}

fn redact_error_for_audit(_: &AgentToolError) -> String {
    "Account write failed; error details are redacted from the audit log".to_string()
}

/// Create a manual account without setting broker identifiers or credentials.
pub struct CreateAccount;

#[async_trait::async_trait]
impl AgentTool for CreateAccount {
    fn name(&self) -> &'static str {
        "create_account"
    }

    fn description(&self) -> &'static str {
        "Create a manual account after the user has confirmed the details. This \
         MUTATES financial data. Broker identifiers and credentials cannot be \
         set through this tool."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "minLength": 1, "maxLength": 200 },
                "accountType": {
                    "type": "string",
                    "enum": ["SECURITIES", "CASH", "CREDIT_CARD", "CRYPTOCURRENCY"]
                },
                "currency": { "type": "string", "minLength": 1, "maxLength": 16 },
                "group": { "type": ["string", "null"], "maxLength": 100 },
                "isDefault": { "type": "boolean", "default": false },
                "isActive": { "type": "boolean", "default": true },
                "trackingMode": {
                    "type": "string",
                    "enum": ["TRANSACTIONS", "HOLDINGS", "NOT_SET"]
                }
            },
            "required": ["name", "accountType", "currency"]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[AgentScope::AccountsWrite]
    }

    fn access_level(&self) -> AgentToolAccess {
        AgentToolAccess::Write
    }

    fn is_additive(&self) -> bool {
        true
    }

    fn sanitize_args_for_audit(&self, args: &serde_json::Value) -> serde_json::Value {
        redact_payload(args)
    }

    fn sanitize_error_for_audit(&self, error: &AgentToolError) -> String {
        redact_error_for_audit(error)
    }

    async fn call(
        &self,
        env: Arc<dyn AgentEnvironment>,
        args: serde_json::Value,
    ) -> Result<AgentToolResult, AgentToolError> {
        let args: CreateAccountArgs = serde_json::from_value(args)?;
        validate_account_name(&args.name)?;
        if args.currency.trim().is_empty() || args.currency.chars().count() > 16 {
            return Err(AgentToolError::InvalidInput(
                "currency must contain 1 to 16 characters".to_string(),
            ));
        }
        if args
            .group
            .as_deref()
            .is_some_and(|group| group.chars().count() > 100)
        {
            return Err(AgentToolError::InvalidInput(
                "group must be 100 characters or fewer".to_string(),
            ));
        }
        if !valid_account_type(&args.account_type) {
            return Err(AgentToolError::InvalidInput(
                "Unsupported accountType".to_string(),
            ));
        }
        let created = env
            .account_service()
            .create_account(NewAccount {
                id: None,
                name: args.name,
                account_type: args.account_type,
                group: args.group,
                currency: args.currency,
                is_default: args.is_default.unwrap_or(false),
                is_active: args.is_active.unwrap_or(true),
                platform_id: None,
                account_number: None,
                meta: None,
                provider: None,
                provider_account_id: None,
                is_archived: false,
                tracking_mode: args.tracking_mode.unwrap_or_default(),
            })
            .await
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(AccountWriteResult::from(created))?,
        })
    }
}

/// Update account settings without exposing provider-owned fields.
pub struct UpdateAccount;

#[async_trait::async_trait]
impl AgentTool for UpdateAccount {
    fn name(&self) -> &'static str {
        "update_account"
    }

    fn description(&self) -> &'static str {
        "Update a manual account's name, type, group, visibility, archive state, \
         or tracking mode. Omitted values are preserved; set clearGroup only \
         when the group should be cleared. Read the account first. This \
         MUTATES financial data; call only after user confirmation."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "accountId": { "type": "string", "minLength": 1 },
                "name": { "type": "string", "minLength": 1, "maxLength": 200 },
                "accountType": {
                    "type": "string",
                    "enum": ["SECURITIES", "CASH", "CREDIT_CARD", "CRYPTOCURRENCY"]
                },
                "group": { "type": "string", "minLength": 1, "maxLength": 100 },
                "clearGroup": { "type": "boolean", "default": false },
                "isDefault": { "type": "boolean" },
                "isActive": { "type": "boolean" },
                "isArchived": { "type": "boolean" },
                "trackingMode": {
                    "type": "string",
                    "enum": ["TRANSACTIONS", "HOLDINGS", "NOT_SET"]
                }
            },
            "required": ["accountId", "name", "accountType"]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[AgentScope::AccountsRead, AgentScope::AccountsWrite]
    }

    fn access_level(&self) -> AgentToolAccess {
        AgentToolAccess::Write
    }

    fn sanitize_args_for_audit(&self, args: &serde_json::Value) -> serde_json::Value {
        redact_payload(args)
    }

    fn sanitize_error_for_audit(&self, error: &AgentToolError) -> String {
        redact_error_for_audit(error)
    }

    async fn call(
        &self,
        env: Arc<dyn AgentEnvironment>,
        args: serde_json::Value,
    ) -> Result<AgentToolResult, AgentToolError> {
        let args: UpdateAccountArgs = serde_json::from_value(args)?;
        validate_account_name(&args.name)?;
        if !valid_account_type(&args.account_type) {
            return Err(AgentToolError::InvalidInput(
                "Unsupported accountType".to_string(),
            ));
        }

        let service = env.account_service();
        let existing = service
            .get_account(&args.account_id)
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        if args.clear_group && args.group.is_some() {
            return Err(AgentToolError::InvalidInput(
                "Specify either group or clearGroup, not both".to_string(),
            ));
        }
        if args
            .group
            .as_deref()
            .is_some_and(|group| group.chars().count() > 100)
        {
            return Err(AgentToolError::InvalidInput(
                "group must be 100 characters or fewer".to_string(),
            ));
        }
        let updated = service
            .update_account(AccountUpdate {
                id: Some(existing.id),
                name: args.name,
                account_type: args.account_type,
                group: if args.clear_group {
                    None
                } else {
                    args.group.or(existing.group.clone())
                },
                is_default: args.is_default.unwrap_or(existing.is_default),
                is_active: args.is_active.unwrap_or(existing.is_active),
                platform_id: existing.platform_id,
                account_number: existing.account_number,
                meta: existing.meta,
                provider: existing.provider,
                provider_account_id: existing.provider_account_id,
                is_archived: args.is_archived.or(Some(existing.is_archived)),
                tracking_mode: args.tracking_mode.or(Some(existing.tracking_mode)),
            })
            .await
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(AccountWriteResult::from(updated))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_write_audit_summary_never_records_financial_details() {
        let args = serde_json::json!({
            "name": "Brokerage",
            "accountNumber": "sensitive-account-number",
            "currency": "USD"
        });
        assert_eq!(
            redact_payload(&args),
            serde_json::json!({ "payload": "[redacted]" })
        );
    }

    #[test]
    fn account_type_allowlist_matches_supported_manual_account_types() {
        assert!(valid_account_type("SECURITIES"));
        assert!(valid_account_type("CREDIT_CARD"));
        assert!(!valid_account_type("BROKER_CONNECTED"));
        assert!(!valid_account_type("securities"));
    }

    #[test]
    fn account_write_audit_error_hides_service_details() {
        let error = AgentToolError::ExecutionFailed("account number 1234".to_string());
        let audit_error = redact_error_for_audit(&error);
        assert!(!audit_error.contains("1234"));
    }
}
