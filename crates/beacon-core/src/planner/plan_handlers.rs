//! Plan handler operations that return formatted wrapper types for the Planner.

use super::Planner;
use crate::{
    error::Result,
    models::{Plan, PlanFilter, PlanSummary},
    params::{DeletePlan, Id, ListPlans, SearchPlans},
};

impl Planner {
    /// Handle listing plans with optional archived filtering.
    ///
    /// Converts plans to summaries with step count information for consistent
    /// list display across interfaces.
    ///
    /// # Arguments
    ///
    /// * `params` - List parameters containing archived flag
    ///
    /// # Returns
    ///
    /// A PlanSummaries wrapper containing plan summary objects with step counts
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::ListPlans, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = ListPlans { archived: false };
    /// let summaries = planner.list_plans_summary(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn list_plans_summary(
        &self,
        params: &ListPlans,
    ) -> Result<crate::display::PlanSummaries> {
        let filter = Some(PlanFilter::from(params));
        let plans = self.list_plans(filter).await?;
        let summaries: Vec<PlanSummary> = plans.iter().map(Into::into).collect();
        Ok(crate::display::PlanSummaries(summaries))
    }

    /// Handle showing a complete plan with all its steps.
    ///
    /// Retrieves a plan with its associated steps eagerly loaded.
    /// The returned Plan object includes all steps in the steps field.
    ///
    /// # Arguments
    ///
    /// * `params` - ID parameters specifying which plan to retrieve
    ///
    /// # Returns
    ///
    /// An optional Plan containing the plan with its steps loaded,
    /// or None if the plan doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::Id, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = Id { id: 1 };
    /// let plan = planner.show_plan_with_steps(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn show_plan_with_steps(&self, params: &Id) -> Result<Option<Plan>> {
        self.get_plan(params).await
    }

    /// Handle creating a new plan.
    ///
    /// Creates a new plan with the specified parameters and returns
    /// the created plan object for confirmation.
    ///
    /// # Arguments
    ///
    /// * `params` - Creation parameters containing title and optional fields
    ///
    /// # Returns
    ///
    /// The newly created Plan object
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::CreatePlan, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = CreatePlan {
    ///     title: "My Plan".to_string(),
    ///     description: Some("A test plan".to_string()),
    ///     directory: None,
    /// };
    /// let plan = planner.create_plan_result(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn create_plan_result(&self, params: &crate::params::CreatePlan) -> Result<Plan> {
        self.create_plan(params).await
    }



    /// Handle permanently deleting a plan with confirmation.
    ///
    /// Permanently removes a plan and all its associated steps from the
    /// database. This operation cannot be undone. Uses get-before-delete
    /// pattern to return the plan details for confirmation.
    ///
    /// Requires explicit confirmation via the `confirmed` field to prevent
    /// accidental deletion. Returns an error if confirmation is not provided.
    ///
    /// # Arguments
    ///
    /// * `params` - DeletePlan parameters containing plan ID and confirmation flag
    ///
    /// # Returns
    ///
    /// Returns the plan details that were deleted for confirmation,
    /// or None if the plan doesn't exist
    ///
    /// # Errors
    ///
    /// Returns `PlannerError::InvalidInput` if `confirmed` field is false
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::DeletePlan, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = DeletePlan { id: 1, confirmed: true };
    /// let deleted_plan = planner.delete_plan(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn delete_plan(&self, params: &DeletePlan) -> Result<Option<Plan>> {
        // Check confirmation flag first
        if !params.confirmed {
            return Err(crate::PlannerError::InvalidInput {
                field: "confirmed".to_string(),
                reason: "Plan deletion requires explicit confirmation. Set 'confirmed' to true to proceed with permanent deletion.".to_string(),
            });
        }

        // Convert to Id params for internal operations
        let id_params = Id { id: params.id };
        let plan = self.get_plan(&id_params).await?;

        if let Some(ref _plan) = plan {
            // Call the underlying delete_plan_by_id method from plan_ops
            self.delete_plan_by_id(&id_params).await?;
        }

        Ok(plan)
    }

    /// Handle searching for plans in a specific directory with summaries.
    ///
    /// Searches for plans associated with the specified directory path,
    /// with optional archived filtering, and returns them as summaries
    /// with step counts. Includes conditional logic for archived vs active
    /// plans.
    ///
    /// # Arguments
    ///
    /// * `params` - Search parameters containing directory and archived flag
    ///
    /// # Returns
    ///
    /// A PlanSummaries wrapper containing plan summary objects matching the
    /// search criteria
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use beacon_core::{params::SearchPlans, PlannerBuilder};
    /// # async {
    /// let planner = PlannerBuilder::new().build().await?;
    /// let params = SearchPlans {
    ///     directory: "/path/to/project".to_string(),
    ///     archived: false,
    /// };
    /// let summaries = planner.search_plans_summary(&params).await?;
    /// # Result::<(), beacon_core::PlannerError>::Ok(())
    /// # };
    /// ```
    pub async fn search_plans_summary(
        &self,
        params: &SearchPlans,
    ) -> Result<crate::display::PlanSummaries> {
        let plans = if params.archived {
            // For archived plans, use list_plans with directory filter
            let filter = PlanFilter::for_directory(params.directory.clone(), true);
            self.list_plans(Some(filter)).await?
        } else {
            // For active plans, use the specialized search method
            self.search_plans_by_directory(params).await?
        };

        let summaries: Vec<PlanSummary> = plans.iter().map(Into::into).collect();
        Ok(crate::display::PlanSummaries(summaries))
    }
}
