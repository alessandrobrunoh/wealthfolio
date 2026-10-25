//! MCP-only contribution-limit create/update tools.

use std::collections::HashSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wealthfolio_core::limits::NewContributionLimit;

use crate::env::AgentEnvironment;
use crate::scope::AgentScope;
use crate::tool::{AgentTool, AgentToolAccess, AgentToolError, AgentToolResult};

const MAX_ACCOUNT_IDS: usize = 100;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateContributionLimitArgs {
    pub group_name: String,
    pub contribution_year: i32,
    pub limit_amount: f64,
    #[serde(default)]
    pub account_ids: Option<Vec<String>>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateContributionLimitArgs {
    pub id: String,
    pub group_name: String,
    pub contribution_year: i32,
    pub limit_amount: f64,
    #[serde(default)]
    pub account_ids: Option<Vec<String>>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContributionLimitWriteOutput {
    pub saved: wealthfolio_core::limits::ContributionLimit,
}

fn validate_limit(
    group_name: &str,
    contribution_year: i32,
    limit_amount: f64,
    account_ids: Option<&[String]>,
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<(), AgentToolError> {
    if group_name.trim().is_empty() || group_name.chars().count() > 100 {
        return Err(AgentToolError::InvalidInput(
            "groupName must contain 1 to 100 characters".to_string(),
        ));
    }
    if !(1900..=9999).contains(&contribution_year) {
        return Err(AgentToolError::InvalidInput(
            "contributionYear must be between 1900 and 9999".to_string(),
        ));
    }
    if !limit_amount.is_finite() || limit_amount < 0.0 {
        return Err(AgentToolError::InvalidInput(
            "limitAmount must be a finite non-negative number".to_string(),
        ));
    }
    if account_ids.is_some_and(|ids| ids.len() > MAX_ACCOUNT_IDS) {
        return Err(AgentToolError::InvalidInput(format!(
            "At most {MAX_ACCOUNT_IDS} accountIds are allowed"
        )));
    }
    if let Some(account_ids) = account_ids {
        let mut unique_ids = HashSet::with_capacity(account_ids.len());
        if account_ids
            .iter()
            .any(|id| id.trim().is_empty() || !unique_ids.insert(id))
        {
            return Err(AgentToolError::InvalidInput(
                "accountIds must contain unique, non-empty IDs".to_string(),
            ));
        }
    }
    if start_date
        .into_iter()
        .chain(end_date)
        .any(|date| date.chars().count() > 64)
    {
        return Err(AgentToolError::InvalidInput(
            "startDate and endDate must be 64 characters or fewer".to_string(),
        ));
    }
    Ok(())
}

fn redact_payload(_: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "payload": "[redacted]" })
}

fn redact_error_for_audit(_: &AgentToolError) -> String {
    "Contribution-limit write failed; error details are redacted from the audit log".to_string()
}

fn account_ids_to_storage(account_ids: Option<Vec<String>>) -> Option<String> {
    account_ids.and_then(|ids| (!ids.is_empty()).then(|| ids.join(",")))
}

pub struct CreateContributionLimit;

#[async_trait::async_trait]
impl AgentTool for CreateContributionLimit {
    fn name(&self) -> &'static str {
        "create_contribution_limit"
    }

    fn description(&self) -> &'static str {
        "Create a contribution limit with an optional set of account IDs. \
         This adds a planning record and does not change account activity."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "groupName": { "type": "string", "minLength": 1, "maxLength": 100 },
                "contributionYear": { "type": "integer", "minimum": 1900, "maximum": 9999 },
                "limitAmount": { "type": "number", "minimum": 0 },
                "accountIds": {
                    "type": ["array", "null"],
                    "maxItems": MAX_ACCOUNT_IDS,
                    "items": { "type": "string", "minLength": 1 },
                    "uniqueItems": true
                },
                "startDate": { "type": ["string", "null"], "maxLength": 64 },
                "endDate": { "type": ["string", "null"], "maxLength": 64 }
            },
            "required": ["groupName", "contributionYear", "limitAmount"]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[AgentScope::FinancialPlanningWrite]
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
        let args: CreateContributionLimitArgs = serde_json::from_value(args)?;
        validate_limit(
            &args.group_name,
            args.contribution_year,
            args.limit_amount,
            args.account_ids.as_deref(),
            args.start_date.as_deref(),
            args.end_date.as_deref(),
        )?;
        let saved = env
            .contribution_limit_service()
            .create_contribution_limit(NewContributionLimit {
                id: None,
                group_name: args.group_name,
                contribution_year: args.contribution_year,
                limit_amount: args.limit_amount,
                account_ids: account_ids_to_storage(args.account_ids),
                start_date: args.start_date,
                end_date: args.end_date,
            })
            .await
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(ContributionLimitWriteOutput { saved })?,
        })
    }
}

pub struct UpdateContributionLimit;

#[async_trait::async_trait]
impl AgentTool for UpdateContributionLimit {
    fn name(&self) -> &'static str {
        "update_contribution_limit"
    }

    fn description(&self) -> &'static str {
        "Replace the editable fields of an existing contribution limit. Pass \
         the complete desired accountIds list. This MUTATES planning data; \
         call only after the user confirms."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "minLength": 1 },
                "groupName": { "type": "string", "minLength": 1, "maxLength": 100 },
                "contributionYear": { "type": "integer", "minimum": 1900, "maximum": 9999 },
                "limitAmount": { "type": "number", "minimum": 0 },
                "accountIds": {
                    "type": ["array", "null"],
                    "maxItems": MAX_ACCOUNT_IDS,
                    "items": { "type": "string", "minLength": 1 },
                    "uniqueItems": true
                },
                "startDate": { "type": ["string", "null"], "maxLength": 64 },
                "endDate": { "type": ["string", "null"], "maxLength": 64 }
            },
            "required": [
                "id", "groupName", "contributionYear", "limitAmount",
                "accountIds", "startDate", "endDate"
            ]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[
            AgentScope::FinancialPlanningRead,
            AgentScope::FinancialPlanningWrite,
        ]
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
        if ["accountIds", "startDate", "endDate"]
            .iter()
            .any(|field| args.get(*field).is_none())
        {
            return Err(AgentToolError::InvalidInput(
                "accountIds, startDate, and endDate must be supplied; use null to clear a field"
                    .to_string(),
            ));
        }
        let args: UpdateContributionLimitArgs = serde_json::from_value(args)?;
        validate_limit(
            &args.group_name,
            args.contribution_year,
            args.limit_amount,
            args.account_ids.as_deref(),
            args.start_date.as_deref(),
            args.end_date.as_deref(),
        )?;
        let saved = env
            .contribution_limit_service()
            .update_contribution_limit(
                &args.id,
                NewContributionLimit {
                    id: Some(args.id.clone()),
                    group_name: args.group_name,
                    contribution_year: args.contribution_year,
                    limit_amount: args.limit_amount,
                    account_ids: account_ids_to_storage(args.account_ids),
                    start_date: args.start_date,
                    end_date: args.end_date,
                },
            )
            .await
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(ContributionLimitWriteOutput { saved })?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_limit_values_and_account_id_bounds() {
        assert!(validate_limit("TFSA", 2025, 7000.0, None, None, None).is_ok());
        assert!(validate_limit(" ", 2025, 7000.0, None, None, None).is_err());
        assert!(validate_limit("TFSA", 2025, f64::NAN, None, None, None).is_err());
        let too_many = vec!["account".to_string(); MAX_ACCOUNT_IDS + 1];
        assert!(validate_limit("TFSA", 2025, 7000.0, Some(&too_many), None, None).is_err());
    }

    #[test]
    fn empty_account_selection_is_stored_as_unset() {
        assert_eq!(account_ids_to_storage(Some(vec![])), None);
        assert_eq!(
            account_ids_to_storage(Some(vec!["a".to_string(), "b".to_string()])),
            Some("a,b".to_string())
        );
    }
}
