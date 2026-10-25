//! MCP-only tools for managing user-owned asset taxonomies and categories.

use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;
use wealthfolio_core::taxonomies::{
    Category, NewCategory, NewTaxonomy, Taxonomy, TaxonomyWithCategories,
};

use crate::env::AgentEnvironment;
use crate::scope::AgentScope;
use crate::tool::{AgentTool, AgentToolAccess, AgentToolError, AgentToolResult};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaxonomyArgs {
    pub name: String,
    pub color: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_single_select: bool,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaxonomyArgs {
    pub id: String,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryArgs {
    pub taxonomy_id: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    pub name: String,
    pub key: String,
    pub color: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCategoryArgs {
    pub taxonomy_id: String,
    pub id: String,
    pub name: String,
    pub key: String,
    pub color: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub icon: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveCategoryArgs {
    pub taxonomy_id: String,
    pub category_id: String,
    pub new_parent_id: Option<String>,
    pub position: i32,
}

fn redact_payload(_: &Value) -> Value {
    serde_json::json!({ "payload": "[redacted]" })
}

fn redact_error_for_audit(_: &AgentToolError) -> String {
    "Taxonomy write failed; error details are redacted from the audit log".to_string()
}

fn map_error(error: impl std::fmt::Display) -> AgentToolError {
    AgentToolError::ExecutionFailed(error.to_string())
}

fn validate_required_text(
    field: &str,
    value: &str,
    max_chars: usize,
) -> Result<(), AgentToolError> {
    if value.trim().is_empty() || value.chars().count() > max_chars {
        return Err(AgentToolError::InvalidInput(format!(
            "{field} must contain 1 to {max_chars} characters"
        )));
    }
    Ok(())
}

fn validate_optional_text(
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

fn require_user_taxonomy(
    taxonomy: Option<TaxonomyWithCategories>,
) -> Result<TaxonomyWithCategories, AgentToolError> {
    let taxonomy =
        taxonomy.ok_or_else(|| AgentToolError::NotFound("Taxonomy not found".to_string()))?;
    if taxonomy.taxonomy.is_system {
        return Err(AgentToolError::InvalidInput(
            "MCP taxonomy writes are limited to user-created taxonomies".to_string(),
        ));
    }
    Ok(taxonomy)
}

pub struct CreateTaxonomy;

#[async_trait::async_trait]
impl AgentTool for CreateTaxonomy {
    fn name(&self) -> &'static str {
        "create_taxonomy"
    }

    fn description(&self) -> &'static str {
        "Create a user-owned asset taxonomy after user confirmation. System \
         taxonomies are never created or modified by this tool."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "minLength": 1, "maxLength": 100 },
                "color": { "type": "string", "minLength": 1, "maxLength": 32 },
                "description": { "type": ["string", "null"], "maxLength": 1000 },
                "isSingleSelect": { "type": "boolean", "default": false },
                "sortOrder": { "type": "integer", "default": 0 }
            },
            "required": ["name", "color"]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[AgentScope::TaxonomiesWrite]
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
        let args: CreateTaxonomyArgs = serde_json::from_value(args)?;
        validate_required_text("name", &args.name, 100)?;
        validate_required_text("color", &args.color, 32)?;
        validate_optional_text("description", args.description.as_deref(), 1000)?;
        let created = env
            .taxonomy_service()
            .create_taxonomy(NewTaxonomy {
                id: None,
                name: args.name,
                color: args.color,
                description: args.description,
                is_system: false,
                is_single_select: args.is_single_select,
                sort_order: args.sort_order,
                scope: "asset".to_string(),
            })
            .await
            .map_err(map_error)?;
        Ok(AgentToolResult {
            content: serde_json::to_value(created)?,
        })
    }
}

pub struct UpdateTaxonomy;

#[async_trait::async_trait]
impl AgentTool for UpdateTaxonomy {
    fn name(&self) -> &'static str {
        "update_taxonomy"
    }

    fn description(&self) -> &'static str {
        "Update a user-owned asset taxonomy's name, color, description, \
         or order. System taxonomies and the taxonomy's selection mode cannot be changed. \
         Confirm the edit with the user first."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "minLength": 1 },
                "name": { "type": "string", "minLength": 1, "maxLength": 100 },
                "color": { "type": "string", "minLength": 1, "maxLength": 32 },
                "description": { "type": ["string", "null"], "maxLength": 1000 },
                "sortOrder": { "type": "integer" }
            },
            "required": ["id", "name", "color", "description", "sortOrder"]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[AgentScope::ClassificationRead, AgentScope::TaxonomiesWrite]
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
        if args.get("description").is_none() {
            return Err(AgentToolError::InvalidInput(
                "description must be supplied; use null to clear it".to_string(),
            ));
        }
        let args: UpdateTaxonomyArgs = serde_json::from_value(args)?;
        validate_required_text("name", &args.name, 100)?;
        validate_required_text("color", &args.color, 32)?;
        validate_optional_text("description", args.description.as_deref(), 1000)?;
        let service = env.taxonomy_service();
        let existing = require_user_taxonomy(service.get_taxonomy(&args.id).map_err(map_error)?)?;
        let taxonomy = existing.taxonomy;
        let updated = service
            .update_taxonomy(Taxonomy {
                id: taxonomy.id,
                name: args.name,
                color: args.color,
                description: args.description,
                is_system: false,
                is_single_select: taxonomy.is_single_select,
                sort_order: args.sort_order,
                created_at: taxonomy.created_at,
                updated_at: taxonomy.updated_at,
                scope: taxonomy.scope,
            })
            .await
            .map_err(map_error)?;
        Ok(AgentToolResult {
            content: serde_json::to_value(updated)?,
        })
    }
}

pub struct CreateCategory;

#[async_trait::async_trait]
impl AgentTool for CreateCategory {
    fn name(&self) -> &'static str {
        "create_category"
    }

    fn description(&self) -> &'static str {
        "Add a category to a user-owned asset taxonomy. The taxonomy and \
         parent category must already exist. Confirm the category with the user."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "taxonomyId": { "type": "string", "minLength": 1 },
                "parentId": { "type": ["string", "null"] },
                "name": { "type": "string", "minLength": 1, "maxLength": 100 },
                "key": { "type": "string", "minLength": 1, "maxLength": 128 },
                "color": { "type": "string", "minLength": 1, "maxLength": 32 },
                "description": { "type": ["string", "null"], "maxLength": 1000 },
                "sortOrder": { "type": "integer", "default": 0 },
                "icon": { "type": ["string", "null"], "maxLength": 64 }
            },
            "required": ["taxonomyId", "name", "key", "color"]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[AgentScope::ClassificationRead, AgentScope::TaxonomiesWrite]
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
        let args: CreateCategoryArgs = serde_json::from_value(args)?;
        validate_required_text("name", &args.name, 100)?;
        validate_required_text("key", &args.key, 128)?;
        validate_required_text("color", &args.color, 32)?;
        validate_optional_text("description", args.description.as_deref(), 1000)?;
        validate_optional_text("icon", args.icon.as_deref(), 64)?;
        let service = env.taxonomy_service();
        let taxonomy =
            require_user_taxonomy(service.get_taxonomy(&args.taxonomy_id).map_err(map_error)?)?;
        if args
            .parent_id
            .as_deref()
            .is_some_and(|id| !taxonomy.categories.iter().any(|category| category.id == id))
        {
            return Err(AgentToolError::InvalidInput(
                "parentId must identify a category in taxonomyId".to_string(),
            ));
        }
        let created = service
            .create_category(NewCategory {
                id: None,
                taxonomy_id: args.taxonomy_id,
                parent_id: args.parent_id,
                name: args.name,
                key: args.key,
                color: args.color,
                description: args.description,
                sort_order: args.sort_order,
                icon: args.icon,
            })
            .await
            .map_err(map_error)?;
        Ok(AgentToolResult {
            content: serde_json::to_value(created)?,
        })
    }
}

pub struct UpdateCategory;

#[async_trait::async_trait]
impl AgentTool for UpdateCategory {
    fn name(&self) -> &'static str {
        "update_category"
    }

    fn description(&self) -> &'static str {
        "Update the name, key, color, description, icon, or order of a category \
         in a user-owned taxonomy. Parent changes use move_category. Confirm \
         the change with the user."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "taxonomyId": { "type": "string", "minLength": 1 },
                "id": { "type": "string", "minLength": 1 },
                "name": { "type": "string", "minLength": 1, "maxLength": 100 },
                "key": { "type": "string", "minLength": 1, "maxLength": 128 },
                "color": { "type": "string", "minLength": 1, "maxLength": 32 },
                "description": { "type": ["string", "null"], "maxLength": 1000 },
                "sortOrder": { "type": "integer" },
                "icon": { "type": ["string", "null"], "maxLength": 64 }
            },
            "required": [
                "taxonomyId", "id", "name", "key", "color",
                "description", "sortOrder", "icon"
            ]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[AgentScope::ClassificationRead, AgentScope::TaxonomiesWrite]
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
        if args.get("description").is_none() || args.get("icon").is_none() {
            return Err(AgentToolError::InvalidInput(
                "description and icon must be supplied; use null to clear a field".to_string(),
            ));
        }
        let args: UpdateCategoryArgs = serde_json::from_value(args)?;
        validate_required_text("name", &args.name, 100)?;
        validate_required_text("key", &args.key, 128)?;
        validate_required_text("color", &args.color, 32)?;
        validate_optional_text("description", args.description.as_deref(), 1000)?;
        validate_optional_text("icon", args.icon.as_deref(), 64)?;
        let service = env.taxonomy_service();
        let taxonomy =
            require_user_taxonomy(service.get_taxonomy(&args.taxonomy_id).map_err(map_error)?)?;
        let existing = taxonomy
            .categories
            .into_iter()
            .find(|category| category.id == args.id)
            .ok_or_else(|| AgentToolError::NotFound("Category not found".to_string()))?;
        let updated = service
            .update_category(Category {
                id: existing.id,
                taxonomy_id: existing.taxonomy_id,
                parent_id: existing.parent_id,
                name: args.name,
                key: args.key,
                color: args.color,
                description: args.description,
                sort_order: args.sort_order,
                created_at: existing.created_at,
                updated_at: existing.updated_at,
                icon: args.icon,
            })
            .await
            .map_err(map_error)?;
        Ok(AgentToolResult {
            content: serde_json::to_value(updated)?,
        })
    }
}

pub struct MoveCategory;

#[async_trait::async_trait]
impl AgentTool for MoveCategory {
    fn name(&self) -> &'static str {
        "move_category"
    }

    fn description(&self) -> &'static str {
        "Move a category in a user-owned taxonomy to a new parent and position. \
         The complete category tree is checked to prevent moving a category \
         inside itself. Confirm the resulting hierarchy with the user."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "taxonomyId": { "type": "string", "minLength": 1 },
                "categoryId": { "type": "string", "minLength": 1 },
                "newParentId": { "type": ["string", "null"] },
                "position": { "type": "integer", "minimum": 0 }
            },
            "required": ["taxonomyId", "categoryId", "newParentId", "position"]
        })
    }

    fn required_scopes(&self) -> &'static [AgentScope] {
        &[AgentScope::ClassificationRead, AgentScope::TaxonomiesWrite]
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
        if args.get("newParentId").is_none() {
            return Err(AgentToolError::InvalidInput(
                "newParentId must be supplied; use null to move the category to the root"
                    .to_string(),
            ));
        }
        let args: MoveCategoryArgs = serde_json::from_value(args)?;
        if args.position < 0 {
            return Err(AgentToolError::InvalidInput(
                "position cannot be negative".to_string(),
            ));
        }
        let service = env.taxonomy_service();
        let taxonomy =
            require_user_taxonomy(service.get_taxonomy(&args.taxonomy_id).map_err(map_error)?)?;
        if !taxonomy
            .categories
            .iter()
            .any(|category| category.id == args.category_id)
        {
            return Err(AgentToolError::NotFound("Category not found".to_string()));
        }
        if let Some(parent_id) = args.new_parent_id.as_deref() {
            if !taxonomy
                .categories
                .iter()
                .any(|category| category.id == parent_id)
            {
                return Err(AgentToolError::InvalidInput(
                    "newParentId must identify a category in taxonomyId".to_string(),
                ));
            }
            if is_descendant(&taxonomy.categories, &args.category_id, parent_id) {
                return Err(AgentToolError::InvalidInput(
                    "A category cannot be moved below itself or one of its descendants".to_string(),
                ));
            }
        }
        let moved = service
            .move_category(
                &args.taxonomy_id,
                &args.category_id,
                args.new_parent_id,
                args.position,
            )
            .await
            .map_err(map_error)?;
        Ok(AgentToolResult {
            content: serde_json::to_value(moved)?,
        })
    }
}

fn is_descendant(categories: &[Category], root_id: &str, candidate_id: &str) -> bool {
    if root_id == candidate_id {
        return true;
    }
    let mut pending = vec![root_id.to_string()];
    while let Some(parent_id) = pending.pop() {
        for child in categories
            .iter()
            .filter(|category| category.parent_id.as_deref() == Some(parent_id.as_str()))
        {
            if child.id == candidate_id {
                return true;
            }
            pending.push(child.id.clone());
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    fn category(id: &str, parent_id: Option<&str>) -> Category {
        let now = NaiveDateTime::default();
        Category {
            id: id.to_string(),
            taxonomy_id: "tax".to_string(),
            parent_id: parent_id.map(str::to_string),
            name: id.to_string(),
            key: id.to_string(),
            color: "#ffffff".to_string(),
            description: None,
            sort_order: 0,
            created_at: now,
            updated_at: now,
            icon: None,
        }
    }

    #[test]
    fn move_rejects_self_and_descendant_parent() {
        let categories = vec![
            category("parent", None),
            category("child", Some("parent")),
            category("grandchild", Some("child")),
        ];
        assert!(is_descendant(&categories, "parent", "parent"));
        assert!(is_descendant(&categories, "parent", "grandchild"));
        assert!(!is_descendant(&categories, "grandchild", "parent"));
    }

    #[test]
    fn taxonomy_write_audit_payload_is_redacted() {
        assert_eq!(
            redact_payload(&serde_json::json!({ "name": "Private taxonomy" })),
            serde_json::json!({ "payload": "[redacted]" })
        );
    }
}
