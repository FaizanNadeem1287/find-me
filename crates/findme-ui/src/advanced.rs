//! Advanced search visual query builder state, presets, and query synthesizer

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

    // Extension groups
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
        if self.ext_documents {
            exts.extend(
                ["doc", "docx", "pdf", "txt", "md", "odt"]
                    .iter()
                    .map(|s| s.to_string()),
            );
        }
        if self.ext_code {
            exts.extend(
                ["rs", "c", "cpp", "py", "js", "ts", "go", "java"]
                    .iter()
                    .map(|s| s.to_string()),
            );
        }
        if self.ext_images {
            exts.extend(
                ["png", "jpg", "jpeg", "gif", "svg", "webp"]
                    .iter()
                    .map(|s| s.to_string()),
            );
        }
        if self.ext_audio {
            exts.extend(
                ["mp3", "wav", "flac", "ogg", "m4a"]
                    .iter()
                    .map(|s| s.to_string()),
            );
        }
        if self.ext_video {
            exts.extend(
                ["mp4", "mkv", "avi", "mov", "webm"]
                    .iter()
                    .map(|s| s.to_string()),
            );
        }
        if self.ext_archives {
            exts.extend(
                ["zip", "tar", "gz", "bz2", "xz", "7z"]
                    .iter()
                    .map(|s| s.to_string()),
            );
        }
        if !self.custom_ext.trim().is_empty() {
            for ext in self
                .custom_ext
                .split(|c: char| c == ',' || c == ';' || c.is_whitespace())
            {
                let cleaned = ext.trim().trim_start_matches('.');
                if !cleaned.is_empty() && !exts.iter().any(|e| e.eq_ignore_ascii_case(cleaned)) {
                    exts.push(cleaned.to_ascii_lowercase());
                }
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
