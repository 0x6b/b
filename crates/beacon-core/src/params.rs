//! Parameter structures for Beacon operations
//!
//! This module contains shared parameter structures that can be used across different
//! interfaces (CLI, MCP, etc.) without framework-specific derives or dependencies.
//! These structures provide a clean interface for passing data between different
//! layers of the application.
//!
//! ## Architecture: Parameter Wrapper Pattern
//!
//! This module implements a parameter wrapper pattern that enables clean separation
//! of concerns between the core domain logic and interface-specific frameworks:
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   CLI Args      │    │   MCP Params    │    │  Core Params    │
//! │  (clap derives) │───▶│ (serde derives) │───▶│ (minimal deps)  │
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//! ```
//!
//! ### Benefits
//!
//! 1. **Separation of Concerns**: Core parameter structures remain independent
//!    of UI framework dependencies (clap, serde, schemars).
//!
//! 2. **Interface Flexibility**: Each interface (CLI, MCP, future REST API)
//!    can add its own framework-specific derives without polluting core logic.
//!
//! 3. **Conditional Compilation**: Features like JSON schema generation can be
//!    enabled only where needed, keeping core lightweight.
//!
//! 4. **Type Safety**: Wrapper pattern ensures compile-time verification of
//!    parameter conversion between layers.
//!
//! ### Usage Pattern
//!
//! Interface layers create wrapper structs that:
//! - Add framework-specific derives (clap::Args, schemars::JsonSchema, etc.)
//! - Use transparent serialization (`#[serde(transparent)]`)
//! - Convert to core parameters via `.into()` or accessor methods
//!
//! ```ignore
//! // In CLI module
//! #[derive(Args)]
//! pub struct CreatePlanArgs {
//!     pub title: String,
//!     // ... clap-specific attributes
//! }
//!
//! impl Into<CreatePlanParams> for CreatePlanArgs {
//!     fn into(self) -> CreatePlanParams {
//!         CreatePlanParams {
//!             title: self.title,
//!             description: self.description,
//!             directory: self.directory,
//!         }
//!     }
//! }
//!
//! // In MCP module  
//! #[derive(Deserialize, JsonSchema)]
//! #[serde(transparent)]
//! struct CreatePlanRequest(beacon_core::params::CreatePlanParams);
//! ```
//!
//! ### Adding New Parameters
//!
//! To add a new parameter structure:
//!
//! 1. **Define core structure** in this module with minimal dependencies
//! 2. **Add interface wrappers** in CLI/MCP modules with appropriate derives
//! 3. **Implement conversions** between wrapper and core types
//! 4. **Update planner methods** to accept core parameter types
//!
//! Example:
//! ```ignore
//! // 1. In beacon-core/src/params.rs
//! #[derive(Debug, Clone)]
//! pub struct NewOperationParams {
//!     pub field1: String,
//!     pub field2: Option<i32>,
//! }
//!
//! // 2. In beacon-cli/src/cli.rs  
//! #[derive(Args)]
//! pub struct NewOperationArgs {
//!     pub field1: String,
//!     #[arg(short, long)]
//!     pub field2: Option<i32>,
//! }
//!
//! impl Into<NewOperationParams> for NewOperationArgs {
//!     fn into(self) -> NewOperationParams {
//!         NewOperationParams {
//!             field1: self.field1,
//!             field2: self.field2,
//!         }
//!     }
//! }
//!
//! // 3. In beacon-cli/src/mcp.rs
//! #[derive(Deserialize, JsonSchema)]
//! #[serde(transparent)]
//! struct NewOperationRequest(beacon_core::params::NewOperationParams);
//! ```

use serde::{Deserialize, Serialize};
#[cfg(feature = "schema")]
use schemars::JsonSchema;

/// Generic parameters for operations requiring just an ID.
///
/// Used for operations like show_plan, archive_plan, unarchive_plan, show_step, claim_step.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct IdParams {
    /// The ID of the resource to operate on
    pub id: u64,
}

/// Parameters for creating a new plan.
///
/// Used to create a new task plan with a title, optional description, and
/// optional working directory association.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct CreatePlanParams {
    /// Title of the plan (required)
    pub title: String,
    /// Optional detailed description of the plan
    pub description: Option<String>,
    /// Optional working directory for the plan
    pub directory: Option<String>,
}

/// Parameters for listing plans.
///
/// Controls whether to show archived or active plans.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct ListPlansParams {
    /// Whether to show archived plans instead of active ones
    #[serde(default)]
    pub archived: bool,
}

/// Parameters for searching plans by directory.
///
/// Allows filtering plans by directory path and archived status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SearchPlansParams {
    /// Directory path to search for plans
    pub directory: String,
    /// Whether to include archived plans in search results
    #[serde(default)]
    pub archived: bool,
}

/// Base parameters for step creation and modification.
///
/// Contains the common fields used when creating or modifying steps.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct StepCreateParams {
    /// ID of the plan to add the step to
    pub plan_id: u64,
    /// Title of the step (required)
    pub title: String,
    /// Optional detailed description of the step
    pub description: Option<String>,
    /// Optional acceptance criteria for the step
    pub acceptance_criteria: Option<String>,
    /// References (URLs, file paths, etc.)
    #[serde(default)]
    pub references: Vec<String>,
}

/// Parameters for inserting a step at a specific position.
///
/// Extends step creation parameters with position information for inserting
/// steps at specific locations within a plan's step order.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct InsertStepParams {
    /// Base step creation parameters
    #[serde(flatten)]
    pub step: StepCreateParams,
    /// Position to insert the step (0-indexed)
    pub position: u32,
}

/// Parameters for swapping the order of two steps.
///
/// Used to reorder steps within a plan by swapping their positions.
/// Both steps must belong to the same plan.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct SwapStepsParams {
    /// ID of the first step to swap
    pub step1_id: u64,
    /// ID of the second step to swap
    pub step2_id: u64,
}

/// Parameters for updating an existing step.
///
/// Allows partial updates to step properties. When changing status to 'done',
/// the result field should be provided to document what was accomplished.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(JsonSchema))]
pub struct UpdateStepParams {
    /// Step ID to update (required)
    pub id: u64,
    /// New status for the step ('todo', 'inprogress', or 'done')
    pub status: Option<String>,
    /// Updated title of the step
    pub title: Option<String>,
    /// Updated detailed description of the step
    pub description: Option<String>,
    /// Updated acceptance criteria for the step
    pub acceptance_criteria: Option<String>,
    /// Updated references (URLs, file paths, etc.)
    pub references: Option<Vec<String>>,
    /// Result description - required when changing status to 'done'.
    ///
    /// This field documents what was actually accomplished when completing the
    /// step. It will be ignored when:
    /// - Changing status to 'todo' or 'inprogress'
    /// - Updating other fields without changing status
    /// - Creating a new step (steps always start as 'todo')
    ///
    /// Example: "Implemented user authentication using JWT tokens with
    /// refresh token rotation. Added middleware for route protection and
    /// created login/logout endpoints. All tests passing."
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}