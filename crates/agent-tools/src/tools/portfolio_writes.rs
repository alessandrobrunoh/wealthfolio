//! MCP-only portfolio create/update tools.

use std::sync::Arc;

use serde::Deserialize;
use wealthfolio_core::portfolios::{NewPortfolio, PortfolioUpdate};

use crate::env::AgentEnvironment;
use crate::scope::AgentScope;
use crate::tool::{AgentTool, AgentToolAccess, AgentToolError, AgentToolResult};

const MAX_PORTFOLIO_ACCOUNTS: usize = 100;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePortfolioArgs {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
    pub account_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePortfolioArgs {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
    pub account_ids: Vec<String>,
}

fn redact_payload(_: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "payload": "[redacted]" })
}

fn redact_error_for_audit(_: &AgentToolError) -> String {
    "Portfolio write failed; error details are redacted from the audit log".to_string()
}

fn validate_portfolio_text(name: &str, description: Option<&str>) -> Result<(), AgentToolError> {
    if name.trim().is_empty() || name.chars().count() > 200 {
        return Err(AgentToolError::InvalidInput(
            "name must contain 1 to 200 characters".to_string(),
        ));
    }
    if description.is_some_and(|text| text.chars().count() > 1000) {
        return Err(AgentToolError::InvalidInput(
            "description must be 1000 characters or fewer".to_string(),
        ));
    }
    Ok(())
}

pub struct CreatePortfolio;

#[async_trait::async_trait]
impl AgentTool for CreatePortfolio {
    fn name(&self) -> &'static str {
        "create_portfolio"
    }

    fn description(&self) -> &'static str {
        "Create a named reporting portfolio from existing account IDs after \
         the user confirms the membership. This adds a portfolio but does not \
         move or change account data."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "minLength": 1, "maxLength": 200 },
                "description": { "type": ["string", "null"], "maxLength": 1000 },
                "sortOrder": { "type": "integer", "default": 0 },
                "accountIds": {
                    "type": "array",
                    "items": { "type": "string", "minLength": 1 },
                    "maxItems": MAX_PORTFOLIO_ACCOUNTS,
                    "minItems": 1,
                    "uniqueItems": true
                }
            },
            "required": ["name", "accountIds"]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[AgentScope::PortfoliosWrite]
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
        let args: CreatePortfolioArgs = serde_json::from_value(args)?;
        validate_portfolio_text(&args.name, args.description.as_deref())?;
        if args.account_ids.len() > MAX_PORTFOLIO_ACCOUNTS {
            return Err(AgentToolError::InvalidInput(format!(
                "At most {MAX_PORTFOLIO_ACCOUNTS} accountIds are allowed"
            )));
        }
        let created = env
            .portfolio_service()
            .create_portfolio(NewPortfolio {
                name: args.name,
                description: args.description,
                sort_order: args.sort_order,
                account_ids: args.account_ids,
            })
            .await
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(created)?,
        })
    }
}

pub struct UpdatePortfolio;

#[async_trait::async_trait]
impl AgentTool for UpdatePortfolio {
    fn name(&self) -> &'static str {
        "update_portfolio"
    }

    fn description(&self) -> &'static str {
        "Update a saved portfolio's name, description, order, or full account \
         membership. Pass the complete desired accountIds list because this \
         replaces the existing membership. This MUTATES data; call only after \
         the user confirms the change."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "minLength": 1 },
                "name": { "type": "string", "minLength": 1, "maxLength": 200 },
                "description": { "type": ["string", "null"], "maxLength": 1000 },
                "sortOrder": { "type": "integer" },
                "accountIds": {
                    "type": "array",
                    "items": { "type": "string", "minLength": 1 },
                    "maxItems": MAX_PORTFOLIO_ACCOUNTS,
                    "minItems": 1,
                    "uniqueItems": true
                }
            },
            "required": ["id", "name", "description", "sortOrder", "accountIds"]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[AgentScope::AccountsRead, AgentScope::PortfoliosWrite]
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
        if args.get("description").is_none() || args.get("sortOrder").is_none() {
            return Err(AgentToolError::InvalidInput(
                "description and sortOrder must be supplied to replace a portfolio".to_string(),
            ));
        }
        let args: UpdatePortfolioArgs = serde_json::from_value(args)?;
        validate_portfolio_text(&args.name, args.description.as_deref())?;
        if args.account_ids.len() > MAX_PORTFOLIO_ACCOUNTS {
            return Err(AgentToolError::InvalidInput(format!(
                "At most {MAX_PORTFOLIO_ACCOUNTS} accountIds are allowed"
            )));
        }
        let updated = env
            .portfolio_service()
            .update_portfolio(PortfolioUpdate {
                id: args.id,
                name: args.name,
                description: args.description,
                sort_order: args.sort_order,
                account_ids: args.account_ids,
            })
            .await
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(updated)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portfolio_write_audit_summary_redacts_account_membership() {
        let args = serde_json::json!({
            "name": "Household",
            "accountIds": ["private-account-id"]
        });
        assert_eq!(
            redact_payload(&args),
            serde_json::json!({ "payload": "[redacted]" })
        );
    }
}
