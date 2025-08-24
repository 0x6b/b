//! Prompt templates for MCP server

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

/// Get predefined prompt templates for task planning
pub fn get_prompt_templates() -> Vec<PromptTemplate> {
    vec![
        PromptTemplate {
            name: "plan".to_string(),
            description: "Create a structured action plan using Beacon's MCP tools".to_string(),
            template: r#"You are **Beacon Planner**, expert at creating well-structured task plans.

# Goal
{goal}

# Your Task
Create a comprehensive plan to achieve this goal using Beacon's MCP tools.

# Step 1: Check Existing Plans
First, use `search_plans` to check for existing plans in the current directory. If relevant plans exist, consider whether to build upon them or create fresh.

# Step 2: Create the Plan
Use `create_plan` with:
- **title**: Concise summary (5-7 words)
- **description**: Clear explanation of approach and expected outcome
- **directory**: (optional - defaults to current directory)

# Step 3: Define Steps
For each logical unit of work, use `add_step` with the plan_id.

## Step Structure Template
```
title: "[Action Verb] [Specific Target]"

description: |
  - **Context**: [Why this step is needed, current state, in detail technically]
  - **Approach**: [How to accomplish this]
  - **Scope**: [What's included/excluded]
  - **Tools/Commands**: [Specific tools or commands to use]
  - **Files**: [Key files/directories involved]

acceptance_criteria: |
  - [ ] [Specific measurable outcome]
  - [ ] [Test command and expected result]
  - [ ] [Quality metric to meet]
  - [ ] [Validation check]

references: ["file.rs", "docs/api.md", "tests/test.rs"]
```

## Step Types to Include

### Analysis Steps
- Understand current implementation
- Identify dependencies and constraints
- Document findings in step result

### Implementation Steps  
- Make specific code changes
- Include test coverage
- Follow project conventions

### Validation Steps
- Run tests and checks
- Verify acceptance criteria
- Ensure no regressions

### Integration Steps
- Connect components
- Verify system behavior
- Test rollback procedures

## Quality Guidelines

Each step should be:
- **Atomic**: Can be completed independently
- **Clear**: Self-contained with all context
- **Verifiable**: Has measurable acceptance criteria
- **Safe**: Includes rollback plan if risky

The complete plan should have:
- 5-10 well-defined steps
- Clear dependencies between steps
- Validation checkpoints
- Risk mitigation for complex operations

## Output
Create a plan that provides everything needed for successful execution. Each step should contain sufficient context that any agent can claim and complete it."#.to_string(),
            arguments: vec![
                PromptTemplateArg {
                    name: "goal".to_string(),
                    description: "The goal or outcome to create a plan for".to_string(),
                    required: true,
                },
            ],
        },
        PromptTemplate {
            name: "do".to_string(),
            description: "Execute a plan by launching focused subagents for each step".to_string(),
            template: r#"You are orchestrating the execution of a Beacon plan by launching focused subagents for each step.

# Plan to Execute
Plan ID: {plan_id}

# Execution Strategy
You will act as an orchestrator, launching specialized subagents, as parrallel if possible, to handle individual steps while maintaining overall progress tracking.

## Step 1: Locate the Plan
{plan_id ? "Use the provided plan_id" : "Use `search_plans` with the current directory to find the most recent active plan"}

## Step 2: Review the Plan
Call `show_plan(id: plan_id)` to understand:
- Overall goal and approach
- All steps and their current status
- Dependencies between steps
- Which steps can be parallelized

## Step 3: Execute Steps via Subagents

For each step with status "todo":

### 3.1 Claim the Step
```
claim_step(id: step_id)
```
This atomically reserves the step for your subagent.

### 3.2 Prepare Subagent Context
Call `show_step(id: step_id)` to gather:
- Step description with full context
- Acceptance criteria
- References and relevant files

### 3.3 Launch Focused Subagent

Create a subagent with a **focused, specific prompt**:

```
You are a specialized subagent tasked with completing a specific step.

## Your Mission
[Step title from show_step]

## Context
[Description from show_step, including Context, Approach, Scope, Tools, and Files sections]

## Success Criteria
[Acceptance criteria from show_step]
Each criterion must be verifiably met before considering the task complete.

## References
[List of relevant files/docs from show_step]

## Your Task
1. Execute the work described above
2. Stay focused on ONLY this specific step
3. Validate each acceptance criterion
4. Document what you accomplished

## Constraints
- Do not work on other steps
- Do not make changes outside the defined scope
- If blocked, document the specific issue
- Provide detailed evidence of success

## Deliverable
Upon completion, provide:
- Detailed description of what was accomplished
- Evidence that each acceptance criterion was met
- Any important findings or deviations
- Test results or validation output
```

### 3.4 Monitor Subagent Progress
While the subagent works:
- Let it focus on the specific task
- Avoid interrupting unless necessary
- Trust it to complete the defined scope

### 3.5 Capture Subagent Results
When the subagent completes, use its output to:
```
update_step(
  id: step_id,
  status: "done",
  result: "[Subagent's detailed report of what was accomplished, validation results, and evidence of success]"
)
```

### 3.6 Handle Subagent Blockers
If the subagent reports a blocker:
```
update_step(
  id: step_id,
  description: description + "\n\nBLOCKER: [Specific issue reported by subagent]",
  status: "inprogress"  // Keep claimed while resolving
)
```
Then either:
- Launch a new subagent with additional context
- Escalate for human intervention
- Try alternative approach

## Step 4: Orchestration Patterns

### Parallel Execution
When steps have no dependencies:
- Claim multiple steps simultaneously
- Launch multiple subagents in parallel
- Each subagent works independently
- Collect results as they complete

### Sequential Execution
When steps have dependencies:
- Wait for prerequisite steps to complete
- Pass relevant results to dependent step subagents
- Ensure outputs flow correctly between steps

### Complex Step Handling
If a step is too large for one subagent:
- Consider using `insert_step` to break it down
- Launch multiple specialized subagents for sub-tasks
- Coordinate their outputs into the final result

## Step 5: Progress Management

Periodically:
- Call `show_plan(id: plan_id)` to review overall progress
- Identify next steps ready for execution
- Check for any blocked steps needing attention
- Determine if additional subagents should be launched

## Subagent Launch Guidelines

### Keep Subagents Focused
- One step per subagent
- Clear, specific objectives
- Defined scope and constraints
- Explicit success criteria

### Provide Complete Context
Each subagent should receive:
- The full step description
- All acceptance criteria
- Relevant file references
- Any results from prerequisite steps

### Enable Independence
Subagents should be able to:
- Work without additional guidance
- Make decisions within their scope
- Validate their own success
- Report clear results

## Quality Assurance

Before marking any step done:
- Verify the subagent met ALL acceptance criteria
- Review the documented results
- Ensure no regressions were introduced
- Confirm the work aligns with the plan's goal

## Completion

When all steps show status "done":
- Review the complete plan with `show_plan`
- Verify the overall goal was achieved
- Consider archiving the plan if appropriate
- Document any lessons learned

Remember: You are the orchestrator. Your role is to launch focused subagents with clear missions, track progress, and ensure the plan succeeds through coordinated execution."#.to_string(),
            arguments: vec![
                PromptTemplateArg {
                    name: "plan_id".to_string(),
                    description: "The ID of the plan to execute (if not provided, will search for latest plan in current directory)".to_string(),
                    required: false,
                },
            ],
        },
    ]
}