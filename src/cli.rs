use clap::{Parser as ClapParser, Subcommand};
use std::path::PathBuf;

#[derive(ClapParser, Debug)]
#[command(author, version, about = "AI-friendly, fully local, semantic code search tool for large codebases", long_about = None)]
pub struct Args {
    /// Search pattern (used when no subcommand is provided)
    #[arg(value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// Files or directories to search (used when no subcommand is provided)
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Skip AST parsing and just output unique files
    #[arg(short, long = "files-only")]
    pub files_only: bool,

    /// Custom patterns to ignore (in addition to .gitignore and common patterns)
    #[arg(short, long)]
    pub ignore: Vec<String>,

    /// Exclude files whose names match query words (filename matching is enabled by default)
    #[arg(short = 'n', long = "exclude-filenames")]
    pub exclude_filenames: bool,

    /// Reranking method to use for search results
    #[arg(short = 'r', long = "reranker", default_value = "hybrid", value_parser = ["hybrid", "hybrid2", "bm25", "tfidf"])]
    pub reranker: String,

    /// Use frequency-based search with stemming and stopword removal (enabled by default)
    #[arg(short = 's', long = "frequency", default_value = "true")]
    pub frequency_search: bool,

    /// Use exact matching without stemming or stopword removal
    #[arg(long = "exact")]
    pub exact: bool,

    /// Maximum number of results to return
    #[arg(long = "max-results")]
    pub max_results: Option<usize>,

    /// Maximum total bytes of code content to return
    #[arg(long = "max-bytes")]
    pub max_bytes: Option<usize>,

    /// Maximum total tokens in code content to return (for AI usage)
    #[arg(long = "max-tokens")]
    pub max_tokens: Option<usize>,

    /// Allow test files and test code blocks in search results
    #[arg(long = "allow-tests")]
    pub allow_tests: bool,

    /// Disable merging of adjacent code blocks after ranking (merging enabled by default)
    #[arg(long = "no-merge", default_value = "false")]
    pub no_merge: bool,

    /// Maximum number of lines between code blocks to consider them adjacent for merging (default: 5)
    #[arg(long = "merge-threshold")]
    pub merge_threshold: Option<usize>,

    /// Output only file names and line numbers without full content
    #[arg(long = "dry-run")]
    pub dry_run: bool,

    /// Output format (default: color)
    #[arg(short = 'o', long = "format", default_value = "color", value_parser = ["terminal", "markdown", "plain", "json", "color"])]
    pub format: String,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Search code using patterns
    Search {
        /// Search pattern
        #[arg(value_name = "PATTERN")]
        pattern: String,

        /// Files or directories to search
        #[arg(value_name = "PATH", default_value = ".")]
        paths: Vec<PathBuf>,

        /// Skip AST parsing and just output unique files
        #[arg(short, long = "files-only")]
        files_only: bool,

        /// Custom patterns to ignore (in addition to .gitignore and common patterns)
        #[arg(short, long)]
        ignore: Vec<String>,

        /// Exclude files whose names match query words (filename matching is enabled by default)
        #[arg(short = 'n', long = "exclude-filenames")]
        exclude_filenames: bool,

        /// Reranking method to use for search results
        #[arg(short = 'r', long = "reranker", default_value = "hybrid", value_parser = ["hybrid", "hybrid2", "bm25", "tfidf"])]
        reranker: String,

        /// Use frequency-based search with stemming and stopword removal (enabled by default)
        #[arg(short = 's', long = "frequency", default_value = "true")]
        frequency_search: bool,

        /// Use exact matching without stemming or stopword removal
        #[arg(long = "exact")]
        exact: bool,

        /// Maximum number of results to return
        #[arg(long = "max-results")]
        max_results: Option<usize>,

        /// Maximum total bytes of code content to return
        #[arg(long = "max-bytes")]
        max_bytes: Option<usize>,

        /// Maximum total tokens in code content to return (for AI usage)
        #[arg(long = "max-tokens")]
        max_tokens: Option<usize>,

        /// Allow test files and test code blocks in search results
        #[arg(long = "allow-tests")]
        allow_tests: bool,

        /// Disable merging of adjacent code blocks after ranking (merging enabled by default)
        #[arg(long = "no-merge", default_value = "false")]
        no_merge: bool,

        /// Maximum number of lines between code blocks to consider them adjacent for merging (default: 5)
        #[arg(long = "merge-threshold")]
        merge_threshold: Option<usize>,

        /// Output only file names and line numbers without full content
        #[arg(long = "dry-run")]
        dry_run: bool,

        /// Output format (default: color)
        #[arg(short = 'o', long = "format", default_value = "color", value_parser = ["terminal", "markdown", "plain", "json", "color"])]
        format: String,
    },

    /// Extract code blocks from files
    ///
    /// This command extracts code blocks from files based on file paths and optional line numbers.
    /// When a line number is specified (e.g., file.rs:10), the command uses tree-sitter to find
    /// the closest suitable parent node (function, struct, class, etc.) for that line.
    Extract {
        /// Files to extract from (can include line numbers with colon, e.g., file.rs:10)
        #[arg(value_name = "FILES")]
        files: Vec<String>,

        /// Allow test files and test code blocks in results
        #[arg(long = "allow-tests")]
        allow_tests: bool,

        /// Number of context lines to include before and after the extracted block
        #[arg(short = 'c', long = "context", default_value = "0")]
        context_lines: usize,

        /// Output format (default: color)
        #[arg(short = 'o', long = "format", default_value = "color", value_parser = ["markdown", "plain", "json", "color"])]
        format: String,
    },

    /// Use AI chat to interact with codebase
    Chat,
}
