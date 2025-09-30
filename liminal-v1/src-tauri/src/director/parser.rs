use super::runbook::{AgentRole, Runbook, Turn};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Unknown agent role: {0}")]
    UnknownRole(String),
    #[error("Invalid turn number: {0}")]
    InvalidTurnNumber(String),
}

pub struct RunbookParser {
    content: String,
}

impl RunbookParser {
    pub fn new(content: String) -> Self {
        Self { content }
    }

    pub fn parse(&self) -> Result<Runbook, ParseError> {
        let mut epoch_id = String::new();
        let mut goal = String::new();
        let mut turns = Vec::new();

        let mut current_text = String::new();
        let mut in_heading = false;
        let mut in_paragraph = false;

        let parser = Parser::new(&self.content);
        let events: Vec<Event> = parser.collect();

        for i in 0..events.len() {
            match &events[i] {
                Event::Start(Tag::Heading {
                    level: _,
                    id: _,
                    classes: _,
                    attrs: _,
                }) => {
                    in_heading = true;
                    current_text.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    in_heading = false;
                    let heading = current_text.trim();

                    if heading.starts_with("Runbook:") {
                        epoch_id = heading
                            .strip_prefix("Runbook:")
                            .unwrap_or("")
                            .trim()
                            .to_string();
                    } else if heading.starts_with("Turn ") {
                        if let Some(turn) = self.parse_turn(heading, &events, i)? {
                            turns.push(turn);
                        }
                    }
                }
                Event::Start(Tag::Paragraph) => {
                    in_paragraph = true;
                }
                Event::End(TagEnd::Paragraph) => {
                    in_paragraph = false;
                }
                Event::Text(text) => {
                    if in_heading {
                        current_text.push_str(text);
                    } else if in_paragraph && current_text.is_empty() {
                        current_text = text.to_string();
                    }
                }
                _ => {}
            }
        }

        let lines: Vec<&str> = self.content.lines().collect();
        for line in lines {
            if line.starts_with("**Epoch Goal:**") {
                goal = line
                    .strip_prefix("**Epoch Goal:**")
                    .unwrap_or("")
                    .trim()
                    .to_string();
                break;
            }
        }

        if epoch_id.is_empty() {
            return Err(ParseError::MissingField("epoch_id".to_string()));
        }
        if goal.is_empty() {
            return Err(ParseError::MissingField("goal".to_string()));
        }

        let mut runbook = Runbook::new(epoch_id, goal);
        for turn in turns {
            runbook.add_turn(turn);
        }
        runbook.build_dependency_graph();

        Ok(runbook)
    }

    fn parse_turn(
        &self,
        heading: &str,
        _events: &[Event],
        _start_idx: usize,
    ) -> Result<Option<Turn>, ParseError> {
        let turn_num = self.extract_turn_number(heading)?;

        let lines: Vec<&str> = self.content.lines().collect();
        let mut in_turn = false;
        let mut specialist: Option<AgentRole> = None;
        let mut parallel_group: Option<usize> = None;
        let mut prompt = String::new();
        let mut acceptance_criteria = Vec::new();
        let mut metadata = HashMap::new();
        let mut in_prompt_block = false;
        let mut in_acceptance_block = false;

        for line in lines {
            if line.starts_with(&format!("## Turn {}", turn_num)) {
                in_turn = true;
                continue;
            }

            if in_turn {
                if line.starts_with("## Turn ") && !line.contains(&format!("Turn {}", turn_num)) {
                    break;
                }

                if line.starts_with("**Specialist:**") {
                    let role_str = line.strip_prefix("**Specialist:**").unwrap_or("").trim();
                    specialist = AgentRole::from_str(role_str).or_else(|| {
                        if role_str.contains("Systems") {
                            Some(AgentRole::Systems)
                        } else if role_str.contains("Interface") {
                            Some(AgentRole::Interface)
                        } else if role_str.contains("Router") {
                            Some(AgentRole::Router)
                        } else if role_str.contains("Testing") {
                            Some(AgentRole::Testing)
                        } else if role_str.contains("Research") {
                            Some(AgentRole::Research)
                        } else if role_str.contains("Director") {
                            Some(AgentRole::Director)
                        } else {
                            None
                        }
                    });
                } else if line.starts_with("**Parallel Group:**") {
                    let group_str = line
                        .strip_prefix("**Parallel Group:**")
                        .unwrap_or("")
                        .trim();
                    if !group_str.contains("N/A") && !group_str.contains("Sequential") {
                        parallel_group = group_str.parse::<usize>().ok();
                    }
                } else if line.starts_with("**Dependencies:**") {
                    let deps_str = line.strip_prefix("**Dependencies:**").unwrap_or("").trim();
                    metadata.insert("dependencies_raw".to_string(), deps_str.to_string());
                } else if line.starts_with("**Prompt to Delegate:**") {
                    in_prompt_block = true;
                    in_acceptance_block = false;
                } else if line.starts_with("**Acceptance:**") {
                    in_prompt_block = false;
                    in_acceptance_block = true;
                } else if line.starts_with("**") {
                    in_prompt_block = false;
                    in_acceptance_block = false;
                } else if in_prompt_block {
                    if let Some(stripped) = line.strip_prefix("> ") {
                        if !prompt.is_empty() {
                            prompt.push('\n');
                        }
                        prompt.push_str(stripped);
                    } else if line.trim().is_empty() && !prompt.is_empty() {
                        prompt.push('\n');
                    }
                } else if in_acceptance_block {
                    if let Some(stripped) = line.strip_prefix("- ") {
                        acceptance_criteria.push(stripped.trim().to_string());
                    }
                }
            }
        }

        if let Some(role) = specialist {
            let mut turn = Turn::new(turn_num, role, prompt.trim().to_string())
                .with_acceptance(acceptance_criteria)
                .with_parallel_group(parallel_group);

            for (k, v) in metadata {
                turn = turn.with_metadata(k, v);
            }

            Ok(Some(turn))
        } else {
            Ok(None)
        }
    }

    fn extract_turn_number(&self, heading: &str) -> Result<usize, ParseError> {
        let parts: Vec<&str> = heading.split_whitespace().collect();
        if parts.len() >= 2 && parts[0] == "Turn" {
            parts[1]
                .parse::<usize>()
                .map_err(|_| ParseError::InvalidTurnNumber(heading.to_string()))
        } else {
            Err(ParseError::InvalidTurnNumber(heading.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_runbook() {
        let content = r#"# Runbook: Test Epoch

**Epoch Goal:** Build a test feature

## Turn 1 — Systems Agent
**Specialist:** Systems
**Parallel Group:** N/A (Sequential)
**Dependencies:** None

**Prompt to Delegate:**
> This is a test prompt
> that spans multiple lines

**Acceptance:**
- Criterion 1
- Criterion 2
"#;

        let parser = RunbookParser::new(content.to_string());
        let result = parser.parse();
        assert!(result.is_ok());

        let runbook = result.unwrap();
        assert_eq!(runbook.epoch_id, "Test Epoch");
        assert_eq!(runbook.goal, "Build a test feature");
        assert_eq!(runbook.turns.len(), 1);

        let turn = &runbook.turns[0];
        assert_eq!(turn.id, 1);
        assert_eq!(turn.specialist, AgentRole::Systems);
        assert_eq!(
            turn.prompt,
            "This is a test prompt\nthat spans multiple lines"
        );
        assert_eq!(turn.acceptance_criteria.len(), 2);
        assert_eq!(turn.parallel_group, None);
    }

    #[test]
    fn test_parse_parallel_groups() {
        let content = r#"# Runbook: Parallel Test

**Epoch Goal:** Test parallel execution

## Turn 1 — Systems Agent
**Specialist:** Systems
**Parallel Group:** 1
**Dependencies:** None

**Prompt to Delegate:**
> First parallel task

**Acceptance:**
- Done

## Turn 2 — Interface Agent
**Specialist:** Interface
**Parallel Group:** 1
**Dependencies:** None

**Prompt to Delegate:**
> Second parallel task

**Acceptance:**
- Done

## Turn 3 — Router Agent
**Specialist:** Router
**Parallel Group:** 2
**Dependencies:** Turn 1, Turn 2

**Prompt to Delegate:**
> Sequential task after parallel

**Acceptance:**
- Done
"#;

        let parser = RunbookParser::new(content.to_string());
        let result = parser.parse();
        assert!(result.is_ok());

        let runbook = result.unwrap();
        assert_eq!(runbook.turns.len(), 3);

        let turn1 = &runbook.turns[0];
        let turn2 = &runbook.turns[1];
        let turn3 = &runbook.turns[2];

        assert_eq!(turn1.parallel_group, Some(1));
        assert_eq!(turn2.parallel_group, Some(1));
        assert_eq!(turn3.parallel_group, Some(2));

        assert_eq!(turn3.dependencies, vec![1, 2]);
    }

    #[test]
    fn test_missing_epoch_goal() {
        let content = r#"# Runbook: Test Epoch

## Turn 1 — Systems Agent
**Specialist:** Systems
**Parallel Group:** N/A

**Prompt to Delegate:**
> Test

**Acceptance:**
- Done
"#;

        let parser = RunbookParser::new(content.to_string());
        let result = parser.parse();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::MissingField(_)));
    }

    #[test]
    fn test_executable_turns() {
        let content = r#"# Runbook: Execution Test

**Epoch Goal:** Test execution logic

## Turn 1 — Systems Agent
**Specialist:** Systems
**Parallel Group:** N/A

**Prompt to Delegate:**
> First task

**Acceptance:**
- Done

## Turn 2 — Interface Agent
**Specialist:** Interface
**Parallel Group:** N/A

**Prompt to Delegate:**
> Second task

**Acceptance:**
- Done
"#;

        let parser = RunbookParser::new(content.to_string());
        let mut runbook = parser.parse().unwrap();

        let executable = runbook.get_executable_turns();
        assert_eq!(executable.len(), 1);
        assert_eq!(executable[0].id, 1);

        runbook.turns[0].status = super::super::runbook::TurnStatus::Completed;

        let executable = runbook.get_executable_turns();
        assert_eq!(executable.len(), 1);
        assert_eq!(executable[0].id, 2);
    }
}
