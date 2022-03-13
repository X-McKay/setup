#!/bin/bash 
set -e

asdf_upgrade() {
  version=$(asdf list-all "$1" | grep -o "^[0-9.]\+$" | sort -V | tail -1)
  asdf install "$1" "$version"
  asdf global "$1" "$version"
}

asdf_install_latest() {
  for lang in "$@"; do
    asdf plugin-add "$lang"
    asdf_upgrade "$lang"
  done
}

install_all_asdf_plugins() {
  echo '-------------------------------------------------'
  echo "Installing ASDF plugins..."
  echo '- - - - - - - - - - - - - - - - - - - - - - - - -'
  asdf_install_latest postgres redis terraform
  echo '-------------------------------------------------'
  echo ""
  echo ""
}

create_dirs_if_not_exists () {
  tmpdir="$HOME/bootstrap_tmp"
  mkdir -p $tmpdir
  mkdir -p "$HOME/bin" "$HOME/.ssh"
  if [[ `uname -s` == "Darwin" ]]; then
    echo -n ""
  elif [[ `uname -s` =~ "MINGW" ]]; then
    echo "ERROR: Unsupported platform!"
    exit 1
  else
    chown "$SUDO_USER":"$SUDO_USER" $tmpdir/*
  fi
  set +x
}

copy_configuration_files() {
  echo '-------------------------------------------------'
  echo "Copying configuration files..."
  echo '- - - - - - - - - - - - - - - - - - - - - - - - -'
  # Refresh information about underlying system
  source "$tmpdir/env_vars.sh"

  # Copy relevant configuration files
  if [[ `uname -s` == "Darwin" ]]; then
    mv "$tmpdir/bashrc"  "$HOME/.bashrc"
    mv "$tmpdir/gitconfig"  "$HOME/.gitconfig"
    mv "$tmpdir/tool-versions"  "$HOME/.tool-versions"
  elif 
    # Linux
    mv "$tmpdir/bashrc"  "$HOME/.bashrc"
    mv "$tmpdir/gitconfig"  "$HOME/.gitconfig"
    mv "$tmpdir/tool-versions"  "$HOME/.tool-versions"
  else [[ `uname -s` =~ "MINGW" ]]; then
    echo "ERROR: Unsupported platform - '$current_platform'"
    exit 1
  fi
  echo '-------------------------------------------------'
  echo ""
  echo ""
}

prepare_for_install(){
  # Install Homebrew if on Darwin
  if [[ "$my_platform" == "darwin" && -n `which brew 2> /dev/null` ]]; then
    echo "Installing Xcode command line tools..."
    xcode-select --install

    echo "Installing Homebrew..."
    ruby -e "$(curl -fsSL https://raw.github.com/Homebrew/homebrew/go/install)"
  fi

  # Refresh information about underlying system
  source "$tmpdir/env_vars.sh"

  # Install necessary packages
  source "$HOME/.bashrc"
}

unknown_install_method() {
  echo "Not sure how to install the necessary packages"
  echo "INSTALL FAILED"
}

install_packages() {
  echo '-------------------------------------------------'
  echo "Installing packages..."
  echo '- - - - - - - - - - - - - - - - - - - - - - - - -'
  # Refresh information about underlying system
  source "$tmpdir/env_vars.sh"

  # Install necessary packages
  source "$HOME/.bashrc"
  if [[ "$install_method" == "install" ]]; then
    if [[ "$current_platform" == "darwin" ]]; then
      if [[ "$install_method" == "brew install" ]]; then

        brew doctor

        set -x
        brew install caskroom/cask/brew-cask
        brew cask install virtualbox
        brew cask install vagrant

        install_all_asdf_plugins
        set +x

      else
        echo "Couldn't find 'brew' command. It's highly recommended that you use 'http://brew.sh'"
        unknown_install_method && exit 1
      fi
    elif [[ "$current_platform" == "linux" ]]; then
      if [[ "$pkg_fmt" == "deb" ]]; then

        set -x
        if [[ -n `which add-apt-repository 2> /dev/null` ]]; then
          sudo add-apt-repository ppa:openjdk-r/ppa

          if [[ -z `ls /etc/apt/sources.list.d/ 2> /dev/null | grep "oracle"` ]]; then
            # Virtualbox
            codename=`lsb_release -c -s`
            wget -q https://www.virtualbox.org/download/oracle_vbox_2016.asc -O- | sudo apt-key add -
            sudo add-apt-repository -y "deb https://download.virtualbox.org/virtualbox/debian ${codename} contrib"
          fi

          sudo apt-get update
        fi

        sudo apt-get upgrade -y

        sudo $install_method binutils ca-certificates cmake curl dnsmasq docker-compose docker.io \
                             git wget xclip kvm snap virtualbox-5.2

        if [[ -n `which snap 2> /dev/null` ]]; then
          snap install rg
        fi

        sudo usermod -a -G libvirtd $(whoami)
        newgrp libvirtd

        set +x

      elif [[ "$pkg_fmt" == "rpm" ]]; then


        if [[ -z `ls /etc/yum.repos.d/ 2> /dev/null | grep "virtualbox"` ]]; then
          sudo wget -q https://download.virtualbox.org/virtualbox/rpm/fedora/virtualbox.repo -O /etc/yum.repos.d/virtualbox.repo
        fi

        set -x

        sudo $pkg_mgr update -y
        sudo $install_method @development-tools binutils* cmake curl \
                             dnsmasq docker git lapack-devel memcached \
                             openssl openssl-devel p7zip ripgrep snapd \
                             virt-install virt-manager wget zlib-devel


        if [[ -n `which snap 2> /dev/null` ]]; then
          # sudo ln -s /var/lib/snapd/snap /snap
          snap install slack --classic
        fi

        sudo usermod -a -G libvirt $(whoami)
        newgrp libvirtd

        set +x

      else
        unknown_install_method && exit 1
      fi
    else
      echo "Don't know how to setup for this system"
    fi
  fi
  echo '-------------------------------------------------'
  echo ""
  echo ""
}

run_bootstrap() {
  echo '-------------------------------------------------'
  echo "Starting bootstrap..."
  create_dirs_if_not_exists
  copy_configuration_files
  prepare_for_install
  install_all_asdf_plugins
  install_packages
  # Remove temporary directory
  rm -rf "$tmpdir"

  # Source the new bashrc
  source "$HOME/.bashrc"
  echo '-------------------------------------------------'
  echo "Bootstrap complete."
  echo '-------------------------------------------------'
}

run_bootstrap



