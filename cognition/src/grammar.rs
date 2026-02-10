use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clause {
    pub subject: Option<String>,
    pub action: Option<String>,
    pub object: Option<String>,
    pub intent: Option<String>, // e.g., "question", "statement", "command"
}

pub struct ClauseAnalyzer;

impl ClauseAnalyzer {
    pub fn analyze(text: &str) -> Clause {
        // Simple heuristic parser without external NLP models
        let lower = text.trim().to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();

        if words.is_empty() {
            return Clause { subject: None, action: None, object: None, intent: None };
        }

        let mut intent = "statement".to_string();
        if text.ends_with('?') {
            intent = "question".to_string();
        } else if words[0] == "learn" || words[0] == "remember" || words[0] == "forget" {
            intent = "command".to_string();
        }

        // Basic SVO detection (Subject Verb Object)
        // Assume English structure: S V O
        // Heuristic: If "is", "has", "does" present, split around it.

        let mut subject = None;
        let mut action = None;
        let mut object = None;

        if let Some(verb_pos) = words.iter().position(|&w| w == "is" || w == "has" || w == "does" || w == "eats" || w == "likes") {
            if verb_pos > 0 {
                subject = Some(words[0..verb_pos].join(" "));
                action = Some(words[verb_pos].to_string());
                if verb_pos + 1 < words.len() {
                    object = Some(words[verb_pos+1..].join(" "));
                }
            }
        } else if words.len() >= 3 {
            // Fallback: Word 0 = S, Word 1 = V, Rest = O
            subject = Some(words[0].to_string());
            action = Some(words[1].to_string());
            object = Some(words[2..].join(" "));
        }

        Clause {
            subject,
            action,
            object,
            intent: Some(intent),
        }
    }
}

// Conversation State
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationState {
    pub history: Vec<String>,
    pub last_topic: Option<String>,
    pub turn_count: usize,
}

impl ConversationState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_turn(&mut self, text: &str) {
        self.history.push(text.to_string());
        self.turn_count += 1;
        // Simple topic tracking: last noun? (stub)
    }
}

// Question Decomposition
pub struct QuestionDecomposition;

impl QuestionDecomposition {
    pub fn decompose(question: &str) -> Vec<String> {
        // "How do I build a house?" -> ["What is a house?", "What are parts of a house?", "How to assemble parts?"]
        // Stub logic for now
        let mut sub_questions = Vec::new();
        if question.to_lowercase().starts_with("how") {
            sub_questions.push(format!("What is the goal of {}?", question));
            sub_questions.push("What are the necessary steps?".to_string());
        } else {
            sub_questions.push(question.to_string());
        }
        sub_questions
    }
}
