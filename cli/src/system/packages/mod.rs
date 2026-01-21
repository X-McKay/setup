//! Package installation and system setup functions.
//!
//! This module provides functions to install development tools,
//! configure services, and manage system updates.

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
    install_bottom, install_docker, install_gh, install_glow, install_hyperfine,
    install_jq, install_just, install_lazygit, install_mise, install_neovim,
    install_tldr, install_tpm, install_yq,
};

// Services (monitoring, backup)
pub use services::{install_backup, install_monitoring};

// Security (SSH, GPG)
pub use security::{setup_gpg, setup_ssh_keys};

// Updates
pub use updates::{sync_dotfiles, update_mise, update_rust_tools, update_system};
