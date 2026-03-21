// Conversation context manager — supports text, tool_calls, and tool result messages.

use std::collections::VecDeque;

use crate::llm::ChatMessage;

pub struct ContextManager {
    system_prompt: String,
    history: VecDeque<ChatMessage>,
    max_messages: usize,
}

impl ContextManager {
    /// Create a new context manager.
    /// `max_messages` is the max number of messages (excluding system) to keep.
    pub fn new(system_prompt: &str, max_messages: usize) -> Self {
        Self {
            system_prompt: system_prompt.to_string(),
            history: VecDeque::new(),
            max_messages,
        }
    }

    /// Update system prompt (e.g. after loading memory).
    pub fn set_system_prompt(&mut self, prompt: String) {
        self.system_prompt = prompt;
    }

    /// Add a message to history, trimming old messages if needed.
    pub fn push(&mut self, msg: ChatMessage) {
        self.history.push_back(msg);
        // Trim from front, but be careful not to break tool_call/tool pairs
        while self.history.len() > self.max_messages {
            self.history.pop_front();
        }
    }

    /// Build the full message array for an LLM request.
    pub fn build_messages(&self) -> Vec<ChatMessage> {
        let mut messages = vec![ChatMessage::text("system", &self.system_prompt)];
        messages.extend(self.history.iter().cloned());
        messages
    }

    /// Build messages with an additional user message appended (not yet in history).
    pub fn build_messages_with(&self, extra: &ChatMessage) -> Vec<ChatMessage> {
        let mut messages = self.build_messages();
        messages.push(extra.clone());
        messages
    }
}
