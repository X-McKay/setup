# .bashrc

# Source global definitions
if [[ `uname -s` != "Darwin" ]]; then
  if [ -f /etc/bashrc ]; then
    . /etc/bashrc
  fi
fi

if [ -f "$HOME/.profile" ]; then
  . "$HOME/.profile"
fi

# Enable Bash Completion from Homebrew
if [[ -n `which brew 2> /dev/null` ]]; then
  if [ -f "$(brew --prefix)/etc/bash_completion" ]; then
    . "$(brew --prefix)/etc/bash_completion"
  fi
fi

setup_darwin(){
  # Enable Bash Colors in OSX
  if [[ `uname -s` == "Darwin" ]]; then
    export CLICOLOR=1
    export LS_COLORS='rs=0:di=01;34:ln=01;36:mh=00:pi=40;33:so=01;35:do=01;35:bd=40;33;01:cd=40;33;01:or=40;31;01:mi=01;05;37;41:su=37;41:sg=30;43:ca=30;41:tw=30;42:ow=34;42:st=37;44:ex=01;32:*.tar=01;31:*.tgz=01;31:*.arj=01;31:*.taz=01;31:*.lzh=01;31:*.lzma=01;31:*.tlz=01;31:*.txz=01;31:*.zip=01;31:*.z=01;31:*.Z=01;31:*.dz=01;31:*.gz=01;31:*.lz=01;31:*.xz=01;31:*.bz2=01;31:*.tbz=01;31:*.tbz2=01;31:*.bz=01;31:*.tz=01;31:*.deb=01;31:*.rpm=01;31:*.jar=01;31:*.war=01;31:*.ear=01;31:*.sar=01;31:*.rar=01;31:*.ace=01;31:*.zoo=01;31:*.cpio=01;31:*.7z=01;31:*.rz=01;31:*.jpg=01;35:*.jpeg=01;35:*.gif=01;35:*.bmp=01;35:*.pbm=01;35:*.pgm=01;35:*.ppm=01;35:*.tga=01;35:*.xbm=01;35:*.xpm=01;35:*.tif=01;35:*.tiff=01;35:*.png=01;35:*.svg=01;35:*.svgz=01;35:*.mng=01;35:*.pcx=01;35:*.mov=01;35:*.mpg=01;35:*.mpeg=01;35:*.m2v=01;35:*.mkv=01;35:*.ogm=01;35:*.mp4=01;35:*.m4v=01;35:*.mp4v=01;35:*.vob=01;35:*.qt=01;35:*.nuv=01;35:*.wmv=01;35:*.asf=01;35:*.rm=01;35:*.rmvb=01;35:*.flc=01;35:*.avi=01;35:*.fli=01;35:*.flv=01;35:*.gl=01;35:*.dl=01;35:*.xcf=01;35:*.xwd=01;35:*.yuv=01;35:*.cgm=01;35:*.emf=01;35:*.axv=01;35:*.anx=01;35:*.ogv=01;35:*.ogx=01;35:*.aac=01;36:*.au=01;36:*.flac=01;36:*.mid=01;36:*.midi=01;36:*.mka=01;36:*.mp3=01;36:*.mpc=01;36:*.ogg=01;36:*.ra=01;36:*.wav=01;36:*.axa=01;36:*.oga=01;36:*.spx=01;36:*.xspf=01;36:'

    defaults write NSGlobalDomain NSAutomaticWindowAnimationsEnabled -bool false
  
  # Export SSL_CERT_FILE on Darwin
    if [[-z "$SSL_CERT_FILE" ]]; then
      cert_file="/usr/local/opt/curl-ca-bundle/share/ca-bundle.crt"
      if test -e "$cert_file"; then
        export SSL_CERT_FILE="$cert_file"
      fi
    fi
  fi

  else
    echo "Not a Mac, incorrectly triggered setup_darwin()"
    exit 1
  fi

}

setup_linux(){
  # Clipboard integration
  if [[ `uname -s` == "Linux" ]]; then
    alias pbcopy='xclip -selection c'

  else
    echo "Not Linux, incorrectly triggered setup_linux()"
    exit 1
  fi

}


# Setup ASDF for managing versions
setup_asdf() {

  if [[ -z `which asdf 2> /dev/null` ]]; then
    export ASDF_HOME="$HOME/.asdf"
  elif which brew &> /dev/null; then
    export ASDF_HOME=$(brew --prefix asdf)
  else
    tmp_asdf_home="$(which asdf | xargs dirname)/.."
    if which realpath &> /dev/null; then
      export ASDF_HOME=`realpath "$tmp_asdf_home"`
    elif which grealpath &> /dev/null; then
      export ASDF_HOME=`grealpath "$tmp_asdf_home"`
    fi
  fi

  if [[ -d "$ASDF_HOME" ]];then
    . "$ASDF_HOME/asdf.sh"
    if test -f "$ASDF_HOME/completions/asdf.bash"; then
      . "$ASDF_HOME/completions/asdf.bash"
    elif test -f "$ASDF_HOME/etc/bash_completion.d/asdf.bash"; then
      . "$ASDF_HOME/etc/bash_completion.d/asdf.bash"
    fi
  fi
}

setup_ssh_agent () {
  # SSH-Agent for not needing to input passwords every time
  if which ssh-agent > /dev/null; then
    if test -f "$HOME/.ssh/id_rsa"; then
      if [[ -n "$SSH_AGENT_PID" ]]; then
        ssh_agent_pid=`ps x | awk '{ print $1 }' | grep "^${SSH_AGENT_PID}$"`
        if [[ "$ssh_agent_pid" != "$SSH_AGENT_PID" ]]; then
          unset SSH_AGENT_PID
          unset SSH_AUTH_SOCK
        fi
      fi

      if [[ -z "$SSH_AGENT_PID" ]]; then
        eval "$(ssh-agent -s)"
        ssh-add "$HOME/.ssh/id_rsa"
      fi
    fi
  fi

}


# General settings
export BASH_SILENCE_DEPRECATION_WARNING=1
export LANG="en_US.UTF-8"

# Handy shortcuts
alias clr="clear"
alias flush="sudo killall -HUP mDNSResponder"
alias ip="curl icanhazip.com"
alias ll="exa -al"
alias ldir="ls -al | grep ^d"
alias o="open ."
alias ut="uptime"
alias ls="exa"
alias ..="cd .."
alias ...="cd ../../../"
alias ....="cd ../../../../"
alias resrc="source ~/.bashrc"




