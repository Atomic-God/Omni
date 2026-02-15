use std::collections::VecDeque;
use log::{info, warn};

#[derive(Clone, Debug)]
pub struct Plan {
    pub steps: VecDeque<String>,
    pub status: PlanStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlanStatus {
    Pending,
    Active,
    Completed,
    Failed,
}

pub struct Planner {
    pub current_plan: Option<Plan>,
    pub goal_history: Vec<String>,
}

impl Planner {
    pub fn new() -> Self {
        Self {
            current_plan: None,
            goal_history: Vec::new(),
        }
    }

    pub fn decompose_goal(&mut self, goal: &str) -> Plan {
        info!("Decomposing goal: {}", goal);
        // Rule-based decomposition for Phase 1
        let mut steps = VecDeque::new();

        if goal.contains("create") && goal.contains("file") {
            steps.push_back("generate_content".to_string());
            steps.push_back("write_file".to_string());
            steps.push_back("verify_file".to_string());
        } else if goal.contains("research") {
            steps.push_back("search_knowledge".to_string());
            steps.push_back("summarize".to_string());
        } else {
            // Default generic step
            steps.push_back(format!("execute_{}", goal.replace(" ", "_")));
        }

        let plan = Plan {
            steps,
            status: PlanStatus::Pending,
        };

        self.current_plan = Some(plan.clone());
        plan
    }

    pub fn update_plan(&mut self, success: bool) {
        if let Some(plan) = &mut self.current_plan {
            if success {
                plan.steps.pop_front();
                if plan.steps.is_empty() {
                    plan.status = PlanStatus::Completed;
                    info!("Plan completed successfully.");
                } else {
                    plan.status = PlanStatus::Active;
                }
            } else {
                plan.status = PlanStatus::Failed;
                warn!("Step failed. Triggering replan...");
                // Simple replan: Retry once or abort
                // Industrial: Add "analyze_failure" step
                plan.steps.push_front("analyze_failure".to_string());
                plan.status = PlanStatus::Active;
            }
        }
    }
}
