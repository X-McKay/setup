//! `neovim` component - Neovim editor.
//!
//! Install delegates to the legacy installer, which also seeds a default
//! config under `~/.config/nvim`.
//!
//! Uninstall: unsupported for now because the install path can mix package
//! manager state with user config that should not be deleted automatically.

use anyhow::{Context, Result};
use std::fs;

use super::util::{ensure_bin_dir, run_command, run_sudo};
use super::Component;

pub struct Neovim;

impl Component for Neovim {
    fn id(&self) -> &str {
        "neovim"
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("nvim").is_ok())
    }

    fn install(&self) -> Result<()> {
        install_neovim()
    }
}

fn install_neovim() -> Result<()> {
    if which::which("nvim").is_err() {
        if run_sudo("apt", &["install", "-y", "neovim"]).is_err() {
            let bin_dir = ensure_bin_dir()?;
            run_command(
                "sh",
                &[
                    "-c",
                    &format!(
                        "curl -Lo {}/nvim https://github.com/neovim/neovim/releases/latest/download/nvim.appimage && chmod +x {}/nvim",
                        bin_dir.display(),
                        bin_dir.display()
                    ),
                ],
            )?;
        }
    }

    let home = dirs::home_dir().context("Could not find home directory")?;
    let nvim_config = home.join(".config").join("nvim");
    fs::create_dir_all(&nvim_config)?;

    let init_lua = nvim_config.join("init.lua");
    if !init_lua.exists() {
        let config = r#"-- Sensible Neovim defaults
vim.opt.number = true
vim.opt.relativenumber = true
vim.opt.mouse = 'a'
vim.opt.ignorecase = true
vim.opt.smartcase = true
vim.opt.hlsearch = false
vim.opt.wrap = false
vim.opt.breakindent = true
vim.opt.tabstop = 4
vim.opt.shiftwidth = 4
vim.opt.expandtab = true
vim.opt.termguicolors = true
vim.opt.signcolumn = 'yes'
vim.opt.updatetime = 250
vim.opt.timeoutlen = 300
vim.opt.splitright = true
vim.opt.splitbelow = true
vim.opt.inccommand = 'split'
vim.opt.cursorline = true
vim.opt.scrolloff = 10
vim.opt.clipboard = 'unnamedplus'
vim.opt.undofile = true

vim.g.mapleader = ' '
vim.g.maplocalleader = ' '

vim.keymap.set('n', '<Esc>', '<cmd>nohlsearch<CR>')
vim.keymap.set('n', '<leader>w', '<cmd>w<CR>', { desc = 'Save' })
vim.keymap.set('n', '<leader>q', '<cmd>q<CR>', { desc = 'Quit' })
vim.keymap.set('n', '<C-h>', '<C-w>h')
vim.keymap.set('n', '<C-j>', '<C-w>j')
vim.keymap.set('n', '<C-k>', '<C-w>k')
vim.keymap.set('n', '<C-l>', '<C-w>l')
"#;
        fs::write(&init_lua, config)?;
    }

    Ok(())
}
