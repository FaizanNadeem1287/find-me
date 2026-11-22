//! Advanced search visual query builder state, presets, and query synthesizer

use std::collections::BTreeSet;

/// Supported file size units for advanced search filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SizeUnit {
    B,
    KB,
    #[default]
    MB,
    GB,
}

impl SizeUnit {
    /// All size units in display order
    pub const ALL: [SizeUnit; 4] = [SizeUnit::B, SizeUnit::KB, SizeUnit::MB, SizeUnit::GB];

    /// Returns the unit symbol as static string
    pub fn as_str(&self) -> &'static str {
        match self {
            SizeUnit::B => "B",
            SizeUnit::KB => "KB",
            SizeUnit::MB => "MB",
            SizeUnit::GB => "GB",
        }
    }
}

impl std::fmt::Display for SizeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Date modification presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DatePreset {
    #[default]
    Any,
    Today,
    Yesterday,
    PastWeek,
    PastMonth,
    PastYear,
}

impl DatePreset {
    /// All available date presets
    pub const ALL: [DatePreset; 6] = [
        DatePreset::Any,
        DatePreset::Today,
        DatePreset::Yesterday,
        DatePreset::PastWeek,
        DatePreset::PastMonth,
        DatePreset::PastYear,
    ];

    /// User-facing display title
    pub fn title(&self) -> &'static str {
        match self {
            DatePreset::Any => "Any Time",
            DatePreset::Today => "Today (dm:today)",
            DatePreset::Yesterday => "Yesterday (dm:yesterday)",
            DatePreset::PastWeek => "Past Week (dm:lastweek)",
            DatePreset::PastMonth => "Past Month (dm:lastmonth)",
            DatePreset::PastYear => "Past Year (dm:lastyear)",
        }
    }

    /// Query syntax snippet
    pub fn query_snippet(&self) -> Option<&'static str> {
        match self {
            DatePreset::Any => None,
            DatePreset::Today => Some("dm:today"),
            DatePreset::Yesterday => Some("dm:yesterday"),
            DatePreset::PastWeek => Some("dm:lastweek"),
            DatePreset::PastMonth => Some("dm:lastmonth"),
            DatePreset::PastYear => Some("dm:lastyear"),
        }
    }
}

impl std::fmt::Display for DatePreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title())
    }
}

/// Category grouping for granular extension selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum ExtensionCategory {
    #[default]
    Audio,
    Code,
    Documents,
    Images,
    Video,
    Archives,
    Data,
}

impl ExtensionCategory {
    pub const ALL: [ExtensionCategory; 7] = [
        ExtensionCategory::Audio,
        ExtensionCategory::Code,
        ExtensionCategory::Documents,
        ExtensionCategory::Images,
        ExtensionCategory::Video,
        ExtensionCategory::Archives,
        ExtensionCategory::Data,
    ];

    pub fn title(&self) -> &'static str {
        match self {
            ExtensionCategory::Audio => "Audio",
            ExtensionCategory::Code => "Code & Config",
            ExtensionCategory::Documents => "Documents",
            ExtensionCategory::Images => "Images",
            ExtensionCategory::Video => "Video",
            ExtensionCategory::Archives => "Archives",
            ExtensionCategory::Data => "Data & DB",
        }
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            ExtensionCategory::Audio => &["ogg", "flac", "mp3", "wav", "m4a", "aac", "opus", "wma"],
            ExtensionCategory::Code => &[
                "rs", "py", "js", "ts", "c", "cpp", "h", "go", "java", "html", "css", "json",
                "toml", "yaml",
            ],
            ExtensionCategory::Documents => {
                &["pdf", "docx", "doc", "txt", "md", "odt", "rtf", "epub"]
            }
            ExtensionCategory::Images => {
                &["png", "jpg", "jpeg", "svg", "webp", "gif", "bmp", "ico"]
            }
            ExtensionCategory::Video => &["mp4", "mkv", "webm", "avi", "mov", "flv", "wmv"],
            ExtensionCategory::Archives => &["zip", "tar", "gz", "7z", "bz2", "xz", "rar"],
            ExtensionCategory::Data => &[
                "sqlite", "sqlite3", "db", "csv", "tsv", "parquet", "xml", "sql",
            ],
        }
    }
}

/// Common search templates for 1-click query synthesis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresetFilter {
    LargeCode,
    RecentDocuments,
    GiganticFiles,
    ArchivesOnly,
}

/// Central state model for the Advanced Search visual form builder
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AdvancedFilterState {
    // Keywords
    pub all_words: String,
    pub exact_phrase: String,
    pub any_words: String,
    pub none_words: String,

    // Granular extension matrix
    pub active_extensions: BTreeSet<String>,
    pub active_ext_category: ExtensionCategory,
    pub custom_ext_input: String,

    // Extension groups (maintained for backwards compatibility)
    pub ext_documents: bool,
    pub ext_code: bool,
    pub ext_images: bool,
    pub ext_audio: bool,
    pub ext_video: bool,
    pub ext_archives: bool,
    pub custom_ext: String,

    // Size constraints
    pub size_min: String,
    pub size_min_unit: SizeUnit,
    pub size_max: String,
    pub size_max_unit: SizeUnit,

    // Date Modified constraint
    pub date_preset: DatePreset,

    // Location & Paths
    pub search_path: String,
    pub exclude_path: String,

    // Flags
    pub match_case: bool,
    pub match_whole_word: bool,
    pub regex_mode: bool,
}

impl AdvancedFilterState {
    /// Toggles a specific file extension (e.g. "ogg", "flac")
    pub fn toggle_extension(&mut self, ext: &str) {
        let clean = ext.trim().trim_start_matches('.').to_ascii_lowercase();
        if clean.is_empty() {
            return;
        }
        if self.active_extensions.contains(&clean) {
            self.active_extensions.remove(&clean);
        } else {
            self.active_extensions.insert(clean);
        }
    }

    /// Checks if an extension is currently active
    pub fn is_extension_active(&self, ext: &str) -> bool {
        let clean = ext.trim().trim_start_matches('.').to_ascii_lowercase();
        self.active_extensions.contains(&clean)
    }

    /// Selects all extensions in the given category
    pub fn select_all_in_category(&mut self, category: ExtensionCategory) {
        for ext in category.extensions() {
            self.active_extensions.insert(ext.to_string());
        }
    }

    /// Clears all extensions in the given category
    pub fn clear_category(&mut self, category: ExtensionCategory) {
        for ext in category.extensions() {
            self.active_extensions.remove(*ext);
        }
    }

    /// Clears all active extensions across all categories
    pub fn clear_all_extensions(&mut self) {
        self.active_extensions.clear();
        self.ext_documents = false;
        self.ext_code = false;
        self.ext_images = false;
        self.ext_audio = false;
        self.ext_video = false;
        self.ext_archives = false;
        self.custom_ext.clear();
        self.custom_ext_input.clear();
    }

    /// Adds a custom extension from input
    pub fn add_custom_extension(&mut self, ext: &str) {
        for part in ext.split(|c: char| c == ',' || c == ';' || c.is_whitespace()) {
            let clean = part.trim().trim_start_matches('.').to_ascii_lowercase();
            if !clean.is_empty() {
                self.active_extensions.insert(clean);
            }
        }
    }
    /// Compiles all active form controls into standard Everything query syntax
    pub fn synthesize_query(&self) -> String {
        let mut tokens: Vec<String> = Vec::new();

        // 1. Flags
        if self.regex_mode {
            tokens.push("regex:".to_string());
        }
        if self.match_case {
            tokens.push("case:".to_string());
        }

        // 2. Keywords
        let all_w = self.all_words.trim();
        if !all_w.is_empty() {
            tokens.push(all_w.to_string());
        }

        let exact = self.exact_phrase.trim();
        if !exact.is_empty() {
            if exact.starts_with('"') && exact.ends_with('"') {
                tokens.push(exact.to_string());
            } else {
                tokens.push(format!("\"{exact}\""));
            }
        }

        let any_w = self.any_words.trim();
        if !any_w.is_empty() {
            let terms: Vec<&str> = any_w.split_whitespace().collect();
            if terms.len() == 1 {
                tokens.push(terms[0].to_string());
            } else if terms.len() > 1 {
                tokens.push(format!("({})", terms.join(" | ")));
            }
        }

        let none_w = self.none_words.trim();
        if !none_w.is_empty() {
            for word in none_w.split_whitespace() {
                if word.starts_with('!') {
                    tokens.push(word.to_string());
                } else {
                    tokens.push(format!("!{word}"));
                }
            }
        }

        // 3. Extensions
        let mut exts: Vec<String> = Vec::new();
        let mut push_ext = |e: &str| {
            let clean = e.trim().trim_start_matches('.').to_ascii_lowercase();
            if !clean.is_empty() && !exts.iter().any(|x| x == &clean) {
                exts.push(clean);
            }
        };

        // Granular active extensions
        for e in &self.active_extensions {
            push_ext(e);
        }

        // Legacy extension group flags
        if self.ext_documents {
            for e in ["doc", "docx", "pdf", "txt", "md", "odt"] {
                push_ext(e);
            }
        }
        if self.ext_code {
            for e in ["rs", "c", "cpp", "py", "js", "ts", "go", "java"] {
                push_ext(e);
            }
        }
        if self.ext_images {
            for e in ["png", "jpg", "jpeg", "gif", "svg", "webp"] {
                push_ext(e);
            }
        }
        if self.ext_audio {
            for e in ["mp3", "wav", "flac", "ogg", "m4a"] {
                push_ext(e);
            }
        }
        if self.ext_video {
            for e in ["mp4", "mkv", "avi", "mov", "webm"] {
                push_ext(e);
            }
        }
        if self.ext_archives {
            for e in ["zip", "tar", "gz", "bz2", "xz", "7z"] {
                push_ext(e);
            }
        }
        if !self.custom_ext.trim().is_empty() {
            for ext in self
                .custom_ext
                .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
            {
                push_ext(ext);
            }
        }
        if !exts.is_empty() {
            tokens.push(format!("ext:{}", exts.join(";")));
        }

        // 4. File Size bounds
        let min_s = self.size_min.trim();
        if !min_s.is_empty() {
            if let Ok(num) = min_s.parse::<u64>() {
                if num > 0 {
                    tokens.push(format!(
                        "size:>={}{}",
                        num,
                        self.size_min_unit.as_str().to_ascii_lowercase()
                    ));
                }
            }
        }

        let max_s = self.size_max.trim();
        if !max_s.is_empty() {
            if let Ok(num) = max_s.parse::<u64>() {
                if num > 0 {
                    tokens.push(format!(
                        "size:<={}{}",
                        num,
                        self.size_max_unit.as_str().to_ascii_lowercase()
                    ));
                }
            }
        }

        // 5. Date Modified
        if let Some(snippet) = self.date_preset.query_snippet() {
            tokens.push(snippet.to_string());
        }

        // 6. Directory restrictions
        let s_path = self.search_path.trim();
        if !s_path.is_empty() {
            if s_path.contains(' ') && !s_path.starts_with('"') {
                tokens.push(format!("path:\"{s_path}\""));
            } else {
                tokens.push(format!("path:{s_path}"));
            }
        }

        let e_path = self.exclude_path.trim();
        if !e_path.is_empty() {
            if e_path.contains(' ') && !e_path.starts_with('"') {
                tokens.push(format!("!path:\"{e_path}\""));
            } else {
                tokens.push(format!("!path:{e_path}"));
            }
        }

        tokens.join(" ")
    }

    /// Applies a preset template to the filter state
    pub fn apply_preset(&mut self, preset: PresetFilter) {
        *self = Self::default();
        match preset {
            PresetFilter::LargeCode => {
                self.ext_code = true;
                self.size_min = "1".to_string();
                self.size_min_unit = SizeUnit::MB;
            }
            PresetFilter::RecentDocuments => {
                self.ext_documents = true;
                self.date_preset = DatePreset::PastWeek;
            }
            PresetFilter::GiganticFiles => {
                self.size_min = "1".to_string();
                self.size_min_unit = SizeUnit::GB;
            }
            PresetFilter::ArchivesOnly => {
                self.ext_archives = true;
            }
        }
    }
}
