//! Camada de Infraestrutura — adaptadores concretos.

mod assistant_settings;
mod github_pat_store;
mod blame;
mod blame_parser;
mod branch_diff;
mod branch_order;
mod branch_origin;
mod branches;
mod audit_log;
mod conflict;
mod credential;
mod github_pr;
mod git2_reader;
mod git_cli;
pub mod llm;
mod llm_credentials;
mod repo_query;
mod reword;
mod repo_watcher;
mod ssh_keys;
mod stashes;
mod status_parser;
mod tags;
mod upstream;
mod validation;
mod worktree_file;

pub use ssh_keys::{read_ssh_public_key, test_github_ssh};

pub use branch_diff::{get_branch_file_diff, list_branch_diff};
pub use branch_order::order_refs_by_recent_checkout;
pub use branches::{list_local_branches, list_remote_branches};
pub use stashes::{list_stashes, stash_reference};
pub use tags::list_tags;
pub use upstream::{
    fetch_all_remote_branch_refs, preview_fetch_all_remote_branch_refs,
};

pub use credential::{
    detect_credential_status, enable_github_use_http_path, ensure_gcm_configured, logout_github_account, store_github_pat,
    trigger_github_login,
};
pub use conflict::{get_conflict_file, resolve_conflict_content, resolve_conflict_side};
pub use assistant_settings::{
    load_settings as load_assistant_settings, save_settings as save_assistant_settings,
};
pub use audit_log::{
    append_entry as append_audit_entry, list_entries as list_audit_entries, now_timestamp,
    purge_old_logs,
};
pub use llm_credentials::{
    clear_llm_api_key, get_llm_api_key, has_llm_api_key, store_llm_api_key,
};
pub use github_pr::{clear_branch_pr_cache, get_branch_pr_status};
pub use git2_reader::{is_git_repo, repo_info, Git2Reader};
pub use git_cli::{
    defensive_config_args, network_operation_timeout, run_streaming_git, run_unbound_git,
    SafeGitCli,
};
pub use repo_query::{
    commit_summary, head_commit_id, is_merge_commit, primary_remote, resolve_commit_id,
};
pub use reword::execute_reword;
pub use repo_watcher::RepoWatcher;
pub use validation::{
    repo_name_from_url, validate_clone_branch, validate_clone_depth, validate_clone_destination,
    validate_compare_ref, validate_folder_name, validate_git_object_id, validate_remote_name,
    validate_remote_url, validate_repo_relative_path, validate_tag_name,
};
pub use worktree_file::{
    absolute_worktree_path, open_git_bash, open_worktree_path, reveal_worktree_path,
    save_worktree_file, worktree_file_exists,
};

#[cfg(test)]
mod tests {
    use super::git_cli::defensive_base_args;

    #[test]
    fn defensive_base_via_git_cli() {
        let args = defensive_base_args("C:/repo");
        assert!(args.contains(&"gc.auto=0".to_string()));
    }
}
