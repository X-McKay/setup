#!/bin/bash 
set -e

asdf_upgrade() {
  echo "asdf_upgrade"
  version=$(asdf list-all "$1" | grep -o "^[0-9.]\+$" | sort -V | tail -1)
  asdf install "$1" "$version"
  asdf global "$1" "$version"
}

# asdf_install_latest() {
#   for lang in "$@"; do
#     asdf plugin-add "$lang"
#     asdf_upgrade "$lang"
#   done
# }

asdf_add_required_plugins() {
  local tool_versions_file="${1}"
  if test -z "${tool_versions_file}"; then
    tool_versions_file="${HOME}/.tool-versions"
  fi 
  plugins=$(awk '{print $1}' < "${tool_versions_file}")
  for plugin in $plugins; do
    if ! asdf list "${plugin}" > /dev/null 2>&1; then
      asdf plugin-add "${plugin}"
    fi
  done
}

asdf_ordered_install() {
  local tool_versions_file="${1}"
  if test -z "${tool_versions_file}"; then
    tool_versions_file="${HOME}/.tool-versions"
  fi 

  #shellcheck disable=SC2207
  plugin_versions=($(cat "${tool_versions_file}"))

  for ((i=0;i<${#plugin_versions[@]};i+=2)); do
    asdf install "${plugin_versions[i]}" "${plugin_versions[i+1]}"; fail=$?
    if test $fail -ne 0; then
      failed="${plugin_versions[i]}"
      break
    fi
  done

  if test -n "${failed}"; then
    return 1
  fi
}


install_all_asdf_plugins() {
  echo '-------------------------------------------------'
  echo "Installing ASDF plugins..."
  echo '- - - - - - - - - - - - - - - - - - - - - - - - -'
  echo "checking for asdf"
  echo `which asdf`
  touch $HOME/.bash_sessions_disable
  echo "adding plugins from .tool-versions"
  asdf_add_required_plugins
  asdf reshim
  echo "installing .tool-versions plugins"
  asdf_ordered_install
  asdf reshim
  # echo "adding additional latest plugins"
  # asdf_install_latest postgres redis terraform
  # asdf reshim
  echo '-------------------------------------------------'
  echo ""
  echo ""
}


create_dirs_if_not_exists () {
  tmpdir="$HOME/bootstrap_tmp"
  mkdir -p $tmpdir
  mkdir -p "$HOME/bin" "$HOME/.ssh"
  set -x
  if [[ `uname -s` == "Darwin" ]]; then
    echo -n ""
  elif [[ `uname -s` =~ "MINGW" ]]; then
    echo "ERROR: Unsupported platform!"
    exit 1
  else
    sudo chown -R "$SUDO_USER":"$SUDO_USER" $tmpdir/
    sudo chmod -R +x $tmpdir/
  fi
  set +x

  cp -a bashrc $tmpdir/
  cp -a env_vars.sh $tmpdir/
  cp -a gitconfig $tmpdir/
  cp -a tool-versions $tmpdir/
  cp -a default-cargo-crates $tmpdir/
  source "$tmpdir/env_vars.sh"
}


copy_configuration_files() {
  echo '-------------------------------------------------'
  echo "Copying configuration files..."
  echo '-------------------------------------------------'
  # Refresh information about underlying system
  # cp -a bashrc $tmpdir/
  # cp -a env_vars.sh $tmpdir/
  # cp -a gitconfig $tmpdir/
  # cp -a tool-versions $tmpdir/
  # source "$tmpdir/env_vars.sh"

  # Copy relevant configuration files
  if [[ `uname -s` == "Darwin" ]]; then
    mv "$tmpdir/bashrc"  "$HOME/.bashrc"
    mv "$tmpdir/gitconfig"  "$HOME/.gitconfig"
    mv "$tmpdir/tool-versions"  "$HOME/.tool-versions"
    mv "$tmpdir/default-cargo-crates"  "$HOME/.default-cargo-crates"
  elif [[ `uname -s` == "Linux" ]]; then
    # Linux
    mv "$tmpdir/bashrc"  "$HOME/.bashrc"
    mv "$tmpdir/gitconfig"  "$HOME/.gitconfig"
    mv "$tmpdir/tool-versions"  "$HOME/.tool-versions"
    mv "$tmpdir/default-cargo-crates"  "$HOME/.default-cargo-crates"
  else
    echo "ERROR: Unsupported platform - '$current_platform'"
    exit 1
  fi
  echo '-------------------------------------------------'
  echo ""
  echo ""
}


prepare_for_install(){
  echo "Preparing for install"
  # Install Homebrew if on Darwin
  if [[ "$my_platform" == "darwin" && -n `which brew 2> /dev/null` ]]; then
    echo "Installing Xcode command line tools..."
    xcode-select --install

    echo "Installing Homebrew..."
    ruby -e "$(curl -fsSL https://raw.github.com/Homebrew/homebrew/go/install)"
  fi

  # Refresh information about underlying system
  # echo "refreshing env vars"
  # source "$tmpdir/env_vars.sh"

  # # Install necessary packages
  # echo "sourcing bash"
  # source "$HOME/.bashrc"
}

git_clone_or_update() {
  if [[ -d "$2" ]]; then
    cd "$2"
    git pull
    cd -
  else
    git clone "$1" "$2"
  fi
}

configure_openssl() { 
  wget https://www.openssl.org/source/openssl-1.1.1g.tar.gz
  tar zxvf openssl-1.1.1g.tar.gz
  cd openssl-1.1.1g
  ./config --prefix=${HOME}/openssl --openssldir=${HOME}/openssl no-ssl2
  cd ..
}


install_asdf() {
  if [[ -z `which asdf 2> /dev/null` ]]; then
    export ASDF_HOME="$HOME/.asdf"
    git_clone_or_update "https://github.com/asdf-vm/asdf.git" "$ASDF_HOME"
    cd "$ASDF_HOME"
    git checkout "$(git describe --abbrev=0 --tags)"
    . asdf.sh
    cd -
  fi
}

unknown_install_method() {
  echo "Not sure how to install the necessary packages"
  echo "INSTALL FAILED"
}

install_packages() {
  echo '-------------------------------------------------'
  echo "Installing packages..."
  echo '-------------------------------------------------'
  # Refresh information about underlying system
  # source "$tmpdir/env_vars.sh"

  # Install necessary packages
  source "$HOME/.bashrc"

  # if [[ "$install_method" == "install" ]]; then
  if [[ "$current_platform" == "darwin" ]]; then
    if [[ "$install_method" == "brew install" ]]; then

      brew doctor

      set -x
      brew install caskroom/cask/brew-cask
      brew cask install virtualbox
      brew cask install vagrant

      # Install ASDF language runtime version manager
      install_asdf
      set +x

    else
      echo "Could not find 'brew' command. Please visit 'http://brew.sh'"
      unknown_install_method && exit 1
    fi
  elif [[ "$current_platform" == "linux" ]]; then
    if [[ "$pkg_fmt" == "deb" ]]; then

      set -x

      # if [[ -n `which add-apt-repository 2> /dev/null` ]]; then
      #   sudo add-apt-repository ppa:openjdk-r/ppa

      #   if [[ -z `ls /etc/apt/sources.list.d/ 2> /dev/null | grep "oracle"` ]]; then
      #     # Virtualbox
      #     codename=`lsb_release -c -s`
      #     wget -q https://www.virtualbox.org/download/oracle_vbox_2016.asc -O- | sudo apt-key add -
      #     sudo add-apt-repository -y "deb https://download.virtualbox.org/virtualbox/debian ${codename} contrib"
      #   fi
      # if [[ -n `which add-apt-repository 2> /dev/null` ]]; then
      #   sudo add-apt-repository ppa:openjdk-r/ppa

      #   if [[ -z `ls /etc/apt/sources.list.d/ 2> /dev/null | grep "oracle"` ]]; then
      #     # Virtualbox
      #     codename=`lsb_release -c -s`
      #     wget -q https://www.virtualbox.org/download/oracle_vbox_2016.asc -O- | sudo apt-key add -
      #     sudo add-apt-repository -y "deb https://download.virtualbox.org/virtualbox/debian ${codename} contrib"
      #   fi

      #   sudo apt-get update
      # fi
      #   sudo apt-get update
      # fi

      sudo apt-get upgrade -y

      sudo apt-get install build-essential -y

      # Install baseline linux packages
      sudo $install_method binutils ca-certificates cmake curl dnsmasq docker-compose docker.io \
                            git wget xclip snap zlib1g-dev openssh-server libedit-dev \
                            python-dev libreadline-dev bzip2 libssl-dev libffi-dev uuid-dev #virtualbox

      # Allow SSH
      sudo ufw allow ssh

      # Configure OpenSSL
      configure_openssl

      # Install ASDF language runtime version manager
      install_asdf
      set +x

    elif [[ "$pkg_fmt" == "rpm" ]]; then


      if [[ -z `ls /etc/yum.repos.d/ 2> /dev/null | grep "virtualbox"` ]]; then
        sudo wget -q https://download.virtualbox.org/virtualbox/rpm/fedora/virtualbox.repo -O /etc/yum.repos.d/virtualbox.repo
      fi

      set -x

      # Install baseline linux packages
      sudo $pkg_mgr update -y
      sudo $install_method @development-tools binutils* cmake curl \
                            dnsmasq docker git lapack-devel memcached \
                            openssl openssl-devel p7zip ripgrep snapd \
                            virt-install virt-manager wget zlib-devel \
                            openssh-server libedit-dev python-dev \
                            libreadline-dev libssl-dev libffi-dev uuid-dev

      # Allow ssh
      sudo ufw allow ssh

      # Configure OpenSSL
      configure_openssl

      # Install ASDF language runtime version manager
      install_asdf
      set +x

    else
      unknown_install_method && exit 1
    fi
  else
    echo "Don't know how to setup for this system"
  fi
  # fi
  echo '-------------------------------------------------'
  echo ""
  echo ""
}

cleanup() {
  echo '-------------------------------------------------'
  echo "Cleaning up files and directories..."
  echo '-------------------------------------------------'
  echo "..."
  # Clean up openssl install
  rm -r openssl-1.1.1g
  rm openssl-1.1.1g.tar.gz
  echo "..."
  # Remove temporary directory
  rm -rf "$tmpdir"
  echo '-------------------------------------------------'
  echo ""
  echo ""
}

run_bootstrap() {
  echo '-------------------------------------------------'
  echo "Starting bootstrap..."
  echo '-------------------------------------------------'
  echo ""
  echo ""
  create_dirs_if_not_exists
  copy_configuration_files
  prepare_for_install
  install_packages
  install_all_asdf_plugins
  cleanup
  
  # Source the new bashrc
  source "$HOME/.bashrc"
  echo '-------------------------------------------------'
  echo "Bootstrap complete."
  echo '-------------------------------------------------'
}

run_bootstrap



