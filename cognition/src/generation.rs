pub struct LanguageGenerator;

impl LanguageGenerator {
    pub fn synthesize_sentence(subject: &str, relation: &str, object: &str) -> String {
        // Simple template for now, but modularized for future grammar expansion
        if relation == "is" {
            format!("{} is a {}.", subject, object)
        } else {
            format!("The {} {} the {}.", subject, relation, object)
        }
    }

    pub fn compose_explanation(chain: &[String]) -> String {
        if chain.len() < 2 { return String::new(); }

        let mut explanation = String::new();
        explanation.push_str(&format!("{} is related to {}", chain[0], chain[1]));

        for i in 2..chain.len() {
            explanation.push_str(&format!(", which implies {}", chain[i]));
        }

        explanation.push('.');
        explanation
    }

    pub fn explain_plan(path: &[String]) -> String {
        if path.is_empty() { return "No plan steps.".to_string(); }
        let mut text = format!("Plan to reach {}:\n", path.last().unwrap());
        for (i, step) in path.iter().enumerate() {
            text.push_str(&format!("{}. {}\n", i + 1, step));
        }
        text
    }
}
