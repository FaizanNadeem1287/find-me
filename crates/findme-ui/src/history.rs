//! Search history, saved query bookmarks, and execution metrics

use serde::{Deserialize, Serialize};

/// A recorded search query execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Unique sequential entry ID
    pub id: u64,
    /// Executed query syntax string
    pub query: String,
    /// Unix timestamp when the search was executed
    pub timestamp_secs: u64,
    /// Number of matching files found
    pub match_count: usize,
    /// Duration of query execution in milliseconds
    pub duration_ms: u64,
    /// Whether this query is starred/bookmarked for quick access
    pub is_bookmarked: bool,
}

impl HistoryEntry {
    /// Formatted human-readable relative execution timestamp
    pub fn formatted_relative_time(&self, now_secs: u64) -> String {
        if now_secs <= self.timestamp_secs {
            return "Just now".to_string();
        }

        let elapsed = now_secs.saturating_sub(self.timestamp_secs);
        if elapsed < 60 {
            format!("{elapsed}s ago")
        } else if elapsed < 3600 {
            format!("{}m ago", elapsed / 60)
        } else if elapsed < 86400 {
            format!("{}h ago", elapsed / 3600)
        } else {
            format!("{}d ago", elapsed / 86400)
        }
    }

    /// Formatted match count with label
    pub fn formatted_matches(&self) -> String {
        let count_str = crate::state::format_number(self.match_count as u64);
        format!("{count_str} matches")
    }
}

/// Search history state management
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SearchHistoryState {
    pub entries: Vec<HistoryEntry>,
    pub filter_bookmarked_only: bool,
    pub search_filter: String,
    pub next_id: u64,
}

impl SearchHistoryState {
    /// Pre-populates search history with helpful standard examples
    pub fn with_default_history() -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(1_700_000_000);

        let entries = vec![
            HistoryEntry {
                id: 1,
                query: "ext:rs size:>100kb".to_string(),
                timestamp_secs: now.saturating_sub(300),
                match_count: 48,
                duration_ms: 2,
                is_bookmarked: true,
            },
            HistoryEntry {
                id: 2,
                query: "path:~/Documents dm:today".to_string(),
                timestamp_secs: now.saturating_sub(1800),
                match_count: 14,
                duration_ms: 1,
                is_bookmarked: true,
            },
            HistoryEntry {
                id: 3,
                query: "Cargo.toml".to_string(),
                timestamp_secs: now.saturating_sub(7200),
                match_count: 125,
                duration_ms: 3,
                is_bookmarked: false,
            },
            HistoryEntry {
                id: 4,
                query: "regex:^lib.*\\.so$".to_string(),
                timestamp_secs: now.saturating_sub(86400),
                match_count: 890,
                duration_ms: 4,
                is_bookmarked: false,
            },
        ];

        Self {
            entries,
            filter_bookmarked_only: false,
            search_filter: String::new(),
            next_id: 5,
        }
    }

    /// Records a new search execution into the history log
    pub fn record_query(
        &mut self,
        query: &str,
        match_count: usize,
        duration_ms: u64,
        now_secs: u64,
    ) {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return;
        }

        // If query was already executed before, preserve bookmark state and move to top
        if let Some(pos) = self.entries.iter().position(|e| e.query == trimmed) {
            let mut existing = self.entries.remove(pos);
            existing.timestamp_secs = now_secs;
            existing.match_count = match_count;
            existing.duration_ms = duration_ms;
            self.entries.insert(0, existing);
            return;
        }

        let entry = HistoryEntry {
            id: self.next_id,
            query: trimmed.to_string(),
            timestamp_secs: now_secs,
            match_count,
            duration_ms,
            is_bookmarked: false,
        };
        self.next_id += 1;

        self.entries.insert(0, entry);

        // Cap history at 200 items
        if self.entries.len() > 200 {
            self.entries.truncate(200);
        }
    }

    /// Toggles bookmark star for a given history entry
    pub fn toggle_bookmark(&mut self, id: u64) -> bool {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.is_bookmarked = !entry.is_bookmarked;
            return true;
        }
        false
    }

    /// Removes a specific history entry
    pub fn remove_entry(&mut self, id: u64) -> bool {
        let original_len = self.entries.len();
        self.entries.retain(|e| e.id != id);
        self.entries.len() < original_len
    }

    /// Clears all history entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns references to entries filtered by bookmark state and search query
    pub fn filtered_entries(&self) -> Vec<&HistoryEntry> {
        let search = self.search_filter.trim().to_ascii_lowercase();

        self.entries
            .iter()
            .filter(|e| {
                if self.filter_bookmarked_only && !e.is_bookmarked {
                    return false;
                }
                if !search.is_empty() && !e.query.to_ascii_lowercase().contains(&search) {
                    return false;
                }
                true
            })
            .collect()
    }
}
