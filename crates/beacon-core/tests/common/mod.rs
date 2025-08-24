use beacon_core::PlannerBuilder;
use tempfile::TempDir;

/// Helper function to create a test planner
pub async fn create_test_planner() -> (TempDir, beacon_core::Planner) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let planner = PlannerBuilder::new()
        .with_database_path(&db_path)
        .build()
        .await
        .expect("Failed to create planner");
    (temp_dir, planner)
}