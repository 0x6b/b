//! Prompt templates for MCP server

use std::sync::LazyLock;

/// Argument definition for a prompt template
#[derive(Debug, Clone)]
pub struct PromptTemplateArg {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Definition of a prompt template
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    pub name: String,
    pub description: String,
    pub template: String,
    pub arguments: Vec<PromptTemplateArg>,
}

pub static PROMPT_TEMPLATES: LazyLock<Vec<PromptTemplate>> = LazyLock::new(|| {
    vec![
        PromptTemplate {
            name: "plan".to_string(),
            description: "Create a structured action plan using Beacon's MCP tools".to_string(),
            template: include_str!("../../templates/plan.md").to_string(),
            arguments: vec![PromptTemplateArg {
                name: "goal".to_string(),
                description: "The goal or outcome to create a plan for".to_string(),
                required: true,
            }],
        },
        PromptTemplate {
            name: "do".to_string(),
            description: "Execute a plan by launching focused subagents for each step".to_string(),
            template: include_str!("../../templates/execute.md").to_string(),
            arguments: vec![PromptTemplateArg {
                name: "plan_id".to_string(),
                description: "The ID of the plan to execute (if not provided, will search for latest plan in current directory)".to_string(),
                required: false,
            }],
        },
    ]
});

