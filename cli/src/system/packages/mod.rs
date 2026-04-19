//! Package installation and system setup functions.
//!
//! This module provides functions to install development tools,
//! configure services, and manage system updates.
//!
//! DEPRECATED: these functions are wrapped by trait impls in
//! `cli/src/components/*.rs`. This module is deleted in Phase 10
//! after `install.rs` is fully rewritten onto the manifest.

mod apt;
mod security;
mod services;
mod tools;
mod updates;
mod utils;

// Re-export all public functions to maintain backward compatibility.
// Commands continue to use: packages::install_docker(), packages::update_system(), etc.

// APT packages
pub use apt::{install_apt_packages, install_extra_tools};

// Individual tools
pub use tools::{
    install_bottom, install_chromium, install_claude_code, install_discord, install_docker,
    install_gh, install_ghostty, install_glow, install_hyperfine, install_jq, install_just,
    install_lazygit, install_mise, install_neovim, install_obsidian, install_spotify,
    install_tldr, install_tpm, install_vlc, install_yq,
};

// Services (monitoring, backup)
pub use services::{install_backup, install_monitoring};

// Security (SSH, GPG)
pub use security::{setup_gpg, setup_ssh_keys};

// Updates
pub use updates::{sync_dotfiles, update_mise, update_rust_tools, update_system};
