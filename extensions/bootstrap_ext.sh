

ubuntu_install_extras() {
    # exa terminal toy
    wget -c http://old-releases.ubuntu.com/ubuntu/pool/universe/r/rust-exa/exa_0.9.0-4_amd64.deb
    sudo apt-get install ./exa_0.9.0-4_amd64.deb
}
    


darwin_install_extras() {
    brew install exa
}



setup_tailscale() {
    # Tailscale for VPN
    curl -fsSL https://tailscale.com/install.sh | sh
    sudo tailscale up
}