//! MCP-only financial-planning reads and writes for goals.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wealthfolio_core::goals::{Goal, GoalFundingRuleInput, GoalPlan, NewGoal, SaveGoalPlan};

use crate::env::AgentEnvironment;
use crate::scope::AgentScope;
use crate::tool::{AgentTool, AgentToolAccess, AgentToolError, AgentToolResult};

const MAX_PLAN_JSON_BYTES: usize = 64 * 1024;
const MAX_FUNDING_RULES: usize = 100;
const MAX_GOAL_TITLE_CHARS: usize = 200;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalIdArgs {
    pub goal_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalDetailsOutput {
    pub goal: Goal,
    pub funding_rules: Vec<wealthfolio_core::goals::GoalFundingRule>,
    pub plan: Option<GoalPlan>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGoalArgs {
    pub goal_type: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub target_amount: Option<f64>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub target_date: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGoalArgs {
    pub goal_id: String,
    pub title: String,
    pub description: Option<String>,
    pub target_amount: Option<f64>,
    pub status_lifecycle: String,
    pub priority: i32,
    pub cover_image_key: Option<String>,
    pub currency: Option<String>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceGoalFundingArgs {
    pub goal_id: String,
    pub rules: Vec<GoalFundingRuleInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveGoalPlanArgs {
    pub goal_id: String,
    pub plan_kind: String,
    #[serde(default)]
    pub planner_mode: Option<String>,
    pub settings_json: String,
}

fn redact_payload(_: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "payload": "[redacted]" })
}

fn redact_error_for_audit(_: &AgentToolError) -> String {
    "Financial planning write failed; error details are redacted from the audit log".to_string()
}

fn require_fields(args: &serde_json::Value, fields: &[&str]) -> Result<(), AgentToolError> {
    if let Some(missing) = fields.iter().find(|field| args.get(**field).is_none()) {
        return Err(AgentToolError::InvalidInput(format!(
            "{missing} must be supplied; use null to clear a nullable field"
        )));
    }
    Ok(())
}

fn validate_goal_text(title: &str, target_amount: Option<f64>) -> Result<(), AgentToolError> {
    if title.trim().is_empty() {
        return Err(AgentToolError::InvalidInput(
            "title cannot be empty".to_string(),
        ));
    }
    if title.chars().count() > MAX_GOAL_TITLE_CHARS {
        return Err(AgentToolError::InvalidInput(format!(
            "title must be {MAX_GOAL_TITLE_CHARS} characters or fewer"
        )));
    }
    if target_amount.is_some_and(|amount| !amount.is_finite() || amount < 0.0) {
        return Err(AgentToolError::InvalidInput(
            "targetAmount must be a finite non-negative number".to_string(),
        ));
    }
    Ok(())
}

fn validate_optional_goal_text(
    field: &str,
    value: Option<&str>,
    max_chars: usize,
) -> Result<(), AgentToolError> {
    if value.is_some_and(|value| value.chars().count() > max_chars) {
        return Err(AgentToolError::InvalidInput(format!(
            "{field} must be {max_chars} characters or fewer"
        )));
    }
    Ok(())
}

fn read_scope() -> &'static [AgentScope] {
    &[AgentScope::FinancialPlanningRead]
}

/// Read a goal together with the funding rules and plan needed to edit it safely.
pub struct GetGoalDetails;

#[async_trait::async_trait]
impl AgentTool for GetGoalDetails {
    fn name(&self) -> &'static str {
        "get_goal_details"
    }

    fn description(&self) -> &'static str {
        "Get one goal with its funding rules and saved plan. Use this before \
         replacing funding rules or editing goal details."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "goalId": { "type": "string", "minLength": 1 }
            },
            "required": ["goalId"]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        read_scope()
    }

    fn access_level(&self) -> AgentToolAccess {
        AgentToolAccess::Read
    }

    fn sanitize_error_for_audit(&self, _: &AgentToolError) -> String {
        "Goal detail read failed; error details are redacted from the audit log".to_string()
    }

    async fn call(
        &self,
        env: Arc<dyn AgentEnvironment>,
        args: serde_json::Value,
    ) -> Result<AgentToolResult, AgentToolError> {
        let args: GoalIdArgs = serde_json::from_value(args)?;
        let service = env.goal_service();
        let goal = service
            .get_goal(&args.goal_id)
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        let funding_rules = service
            .get_goal_funding(&args.goal_id)
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        let plan = service
            .get_goal_plan(&args.goal_id)
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(GoalDetailsOutput {
                goal,
                funding_rules,
                plan,
            })?,
        })
    }
}

pub struct CreateGoal;

#[async_trait::async_trait]
impl AgentTool for CreateGoal {
    fn name(&self) -> &'static str {
        "create_goal"
    }

    fn description(&self) -> &'static str {
        "Create a financial goal after the user has confirmed its details. \
         This adds a goal and does not delete or overwrite existing goals."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "goalType": { "type": "string", "minLength": 1, "maxLength": 64 },
                "title": { "type": "string", "minLength": 1, "maxLength": MAX_GOAL_TITLE_CHARS },
                "description": { "type": ["string", "null"], "maxLength": 4000 },
                "targetAmount": { "type": ["number", "null"], "minimum": 0 },
                "priority": { "type": "integer" },
                "currency": { "type": ["string", "null"], "maxLength": 16 },
                "startDate": { "type": ["string", "null"], "maxLength": 64 },
                "targetDate": { "type": ["string", "null"], "maxLength": 64 }
            },
            "required": ["goalType", "title"]
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
        let args: CreateGoalArgs = serde_json::from_value(args)?;
        if args.goal_type.trim().is_empty() || args.goal_type.chars().count() > 64 {
            return Err(AgentToolError::InvalidInput(
                "goalType must contain 1 to 64 characters".to_string(),
            ));
        }
        validate_goal_text(&args.title, args.target_amount)?;
        validate_optional_goal_text("description", args.description.as_deref(), 4000)?;
        validate_optional_goal_text("currency", args.currency.as_deref(), 16)?;
        validate_optional_goal_text("startDate", args.start_date.as_deref(), 64)?;
        validate_optional_goal_text("targetDate", args.target_date.as_deref(), 64)?;
        let created = env
            .goal_service()
            .create_goal(NewGoal {
                id: None,
                goal_type: args.goal_type,
                title: args.title,
                description: args.description,
                target_amount: args.target_amount,
                status_lifecycle: None,
                status_health: None,
                priority: args.priority,
                cover_image_key: None,
                currency: args.currency,
                start_date: args.start_date,
                target_date: args.target_date,
                created_at: None,
                updated_at: None,
            })
            .await
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(created)?,
        })
    }
}

pub struct UpdateGoal;

#[async_trait::async_trait]
impl AgentTool for UpdateGoal {
    fn name(&self) -> &'static str {
        "update_goal"
    }

    fn description(&self) -> &'static str {
        "Update the user-editable fields of an existing goal. Pass all nullable \
         fields explicitly as null when they should be cleared. Computed summary \
         fields and goal type are preserved. This MUTATES data; call only after \
         the user confirms."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "goalId": { "type": "string", "minLength": 1 },
                "title": { "type": "string", "minLength": 1, "maxLength": MAX_GOAL_TITLE_CHARS },
                "description": { "type": ["string", "null"], "maxLength": 4000 },
                "targetAmount": { "type": ["number", "null"], "minimum": 0 },
                "statusLifecycle": { "type": "string", "enum": ["active", "achieved", "archived"] },
                "priority": { "type": "integer" },
                "coverImageKey": { "type": ["string", "null"], "maxLength": 200 },
                "currency": { "type": ["string", "null"], "maxLength": 16 },
                "startDate": { "type": ["string", "null"], "maxLength": 64 },
                "targetDate": { "type": ["string", "null"], "maxLength": 64 }
            },
            "required": [
                "goalId", "title", "description", "targetAmount",
                "statusLifecycle", "priority", "coverImageKey",
                "currency", "startDate", "targetDate"
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
        require_fields(
            &args,
            &[
                "description",
                "targetAmount",
                "coverImageKey",
                "currency",
                "startDate",
                "targetDate",
            ],
        )?;
        let args: UpdateGoalArgs = serde_json::from_value(args)?;
        validate_goal_text(&args.title, args.target_amount)?;
        validate_optional_goal_text("description", args.description.as_deref(), 4000)?;
        validate_optional_goal_text("coverImageKey", args.cover_image_key.as_deref(), 200)?;
        validate_optional_goal_text("currency", args.currency.as_deref(), 16)?;
        validate_optional_goal_text("startDate", args.start_date.as_deref(), 64)?;
        validate_optional_goal_text("targetDate", args.target_date.as_deref(), 64)?;
        let service = env.goal_service();
        let existing = service
            .get_goal(&args.goal_id)
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        let updated = service
            .update_goal(Goal {
                id: existing.id,
                goal_type: existing.goal_type,
                title: args.title,
                description: args.description,
                target_amount: args.target_amount,
                status_lifecycle: args.status_lifecycle,
                status_health: existing.status_health,
                priority: args.priority,
                cover_image_key: args.cover_image_key,
                currency: args.currency,
                start_date: args.start_date,
                target_date: args.target_date,
                summary_current_value: existing.summary_current_value,
                summary_progress: existing.summary_progress,
                projected_completion_date: existing.projected_completion_date,
                projected_value_at_target_date: existing.projected_value_at_target_date,
                created_at: existing.created_at,
                updated_at: existing.updated_at,
                summary_target_amount: existing.summary_target_amount,
            })
            .await
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(updated)?,
        })
    }
}

pub struct ReplaceGoalFunding;

#[async_trait::async_trait]
impl AgentTool for ReplaceGoalFunding {
    fn name(&self) -> &'static str {
        "replace_goal_funding"
    }

    fn description(&self) -> &'static str {
        "Replace all funding rules for a goal with the supplied complete rules \
         list (an empty list clears them). Read goal details first and get the \
         user's confirmation before calling. Domain validation prevents \
         over-allocation."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "goalId": { "type": "string", "minLength": 1 },
                "rules": {
                    "type": "array",
                    "maxItems": MAX_FUNDING_RULES,
                    "items": {
                        "type": "object",
                        "properties": {
                            "accountId": { "type": "string", "minLength": 1 },
                            "sharePercent": { "type": "number", "minimum": 0, "maximum": 100 },
                            "taxBucket": { "type": ["string", "null"] }
                        },
                        "required": ["accountId", "sharePercent"]
                    }
                }
            },
            "required": ["goalId", "rules"]
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
        let args: ReplaceGoalFundingArgs = serde_json::from_value(args)?;
        if args.rules.len() > MAX_FUNDING_RULES {
            return Err(AgentToolError::InvalidInput(format!(
                "At most {MAX_FUNDING_RULES} funding rules are allowed"
            )));
        }
        let updated = env
            .goal_service()
            .save_goal_funding(&args.goal_id, args.rules)
            .await
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(updated)?,
        })
    }
}

pub struct SaveGoalPlanTool;

#[async_trait::async_trait]
impl AgentTool for SaveGoalPlanTool {
    fn name(&self) -> &'static str {
        "save_goal_plan"
    }

    fn description(&self) -> &'static str {
        "Save a validated goal plan. Read the current goal details first. \
         This replaces the plan settings and may change the financial \
         projections; call only after user confirmation."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "goalId": { "type": "string", "minLength": 1 },
                "planKind": { "type": "string", "enum": ["retirement", "save_up"] },
                "plannerMode": { "type": ["string", "null"] },
                "settingsJson": {
                    "type": "string",
                    "maxLength": MAX_PLAN_JSON_BYTES,
                    "description": "Complete JSON settings object validated by the goal service."
                }
            },
            "required": ["goalId", "planKind", "plannerMode", "settingsJson"]
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
        require_fields(&args, &["plannerMode"])?;
        let args: SaveGoalPlanArgs = serde_json::from_value(args)?;
        if args.settings_json.len() > MAX_PLAN_JSON_BYTES {
            return Err(AgentToolError::InvalidInput(format!(
                "settingsJson must be {MAX_PLAN_JSON_BYTES} bytes or fewer"
            )));
        }
        serde_json::from_str::<serde_json::Value>(&args.settings_json).map_err(|_| {
            AgentToolError::InvalidInput("settingsJson must contain valid JSON".to_string())
        })?;
        if args
            .planner_mode
            .as_deref()
            .is_some_and(|mode| !matches!(mode, "fire" | "traditional"))
        {
            return Err(AgentToolError::InvalidInput(
                "plannerMode must be fire or traditional".to_string(),
            ));
        }
        let saved = env
            .goal_service()
            .save_goal_plan(SaveGoalPlan {
                goal_id: args.goal_id,
                plan_kind: args.plan_kind,
                planner_mode: args.planner_mode,
                settings_json: args.settings_json,
                summary_json: None,
            })
            .await
            .map_err(|error| AgentToolError::ExecutionFailed(error.to_string()))?;
        Ok(AgentToolResult {
            content: serde_json::to_value(saved)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_json_size_limit_is_applied_to_utf8_bytes() {
        let args = SaveGoalPlanArgs {
            goal_id: "goal-1".to_string(),
            plan_kind: "save_up".to_string(),
            planner_mode: None,
            settings_json: "x".repeat(MAX_PLAN_JSON_BYTES + 1),
        };
        assert!(args.settings_json.len() > MAX_PLAN_JSON_BYTES);
    }

    #[test]
    fn goal_write_audit_payload_is_redacted() {
        let args = serde_json::json!({
            "goalId": "goal-1",
            "settingsJson": "{\"salary\": 250000}"
        });
        assert_eq!(
            redact_payload(&args),
            serde_json::json!({ "payload": "[redacted]" })
        );
    }
}
