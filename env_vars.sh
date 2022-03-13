#!/bin/bash 
set -e

get_platform() {
    echo '-------------------------------------------------'
    echo "Determining platform..."
    echo '- - - - - - - - - - - - - - - - - - - - - - - - -'

    # Determine Platform
    uname_output=`uname -s`
    if [[ "$uname_output" =~ "Linux" ]]; then
    current_platform="linux"
    echo "Platform: Linux"

    elif [[ "$uname_output" =~ "Darwin" ]]; then
    current_platform="darwin"
    echo "Platform: Mac"

    else
    echo "ERROR: Unsupported platform - '$uname_output'"
    exit 1
    fi
    echo '-------------------------------------------------'
    echo ""
    echo ""
}

get_cpu_info() {
    echo '-------------------------------------------------'
    echo "Determining CPU..."
    echo '- - - - - - - - - - - - - - - - - - - - - - - - -'

    # Determine CPU
    if [[ "$current_platform" == "linux" ]]; then
        if [[ -e "/proc/cpuinfo" ]]; then
            # Linux
            num_cpus=`cat /proc/cpuinfo | grep processor | wc -l`
            cpu_vendor=`cat /proc/cpuinfo | grep "Vendor ID:" | grep -o "\w\+$" | head -1`
        elif [[ -n `which lscpu 2> /dev/null` ]]; then
            # Linux Alternative
            num_cpus=`lscpu | grep -i "CPU(s):" | awk '{print $2}'`
            cpu_vendor=`lscpu | grep "Vendor ID:" | grep -o "\w\+$" | head -1`
        else
            # Currently focusing on Ubuntu
            echo "ERROR: Unable to determine CPU for Linux platform"
            exit 1
        fi
    elif [[ "$current_platform" == "darwin" ]]; then
        # Mac
        num_cpus=`sysctl -n hw.ncpu`
        cpu_vendor=`sysctl -n machdep.cpu.vendor`
    else
        echo "ERROR: Unable to determine CPU information for platform '$current_platform'"
        exit 1
    fi

    cpu_arch=`uname -p` #TODO: FIX OVERLAP
    cpu_arch_family=`uname -p`
    echo "CPU count: $num_cpus"
    echo "CPU vendor: $cpu_vendor"
    echo "CPU architecture: $cpu_arch"
    echo '-------------------------------------------------'
    echo ""
    echo ""
}


setup_build_info() {
    echo '-------------------------------------------------'
    echo "Setting up build info..."
    echo '- - - - - - - - - - - - - - - - - - - - - - - - -'


    # Determine OS and build info
    if [[ "$my_platform" == "linux" ]]; then
        # Linux - checking for lsb_release
        if [[ -z `which lsb_release 2> /dev/null` ]]; then
            if [[ -n `which apt-get 2> /dev/null` ]]; then 
            sudo apt-get install -y lsb
            else
            echo "ERROR: Unknown package manager in use!!"
            exit 1
            fi
        fi

        install_method="install"
        current_major_release=`lsb_release -r 2> /dev/null | awk '{print $2}' | grep -o "[0-9]\+" | head -1`
        current_release_nickname=`lsb_release -c 2> /dev/null | awk '{print $2}'`
        current_pkg_arch="$cpu_arch_family"
        lsb_release_output=`lsb_release -a 2> /dev/null`

        if [[ -n `echo $lsb_release_output | grep -i "debian"` ]]; then
            # Debian (not tested)
            current_distro="debian"
            pkg_fmt="deb"
            my_pkg_mgr="apt-get"
            install_method="apt-get install -y"
            local_install_method="dpkg -i"
            if [[ "$cpu_arch_family" == "x86_64" ]]; then pkg_arch="amd64"; fi
        elif [[ -n `echo $lsb_release_output | grep -i "ubuntu"` ]]; then
            # Ubuntu (primary Linux focus)
            current_distro="ubuntu"
            pkg_fmt="deb"
            pkg_mgr="apt-get"
            install_method="apt-get install -y"
            local_install_method="dpkg -i"
            if [[ "$cpu_arch_family" == "x86_64" ]]; then pkg_arch="amd64"; fi
        else
            echo "Warning: Unsupported Linux distribution, any packages will be compiled from source"
            install_method="build"
            current_distro=`lsb_release -d 2> /dev/null`
        fi

    elif [[ "$current_platform" == "darwin" ]]; then
        # Mac
        current_distro="Mac OSX"
        install_method=""

        if [[ -n `which brew 2> /dev/null` ]]; then # Homebrew
            install_method="brew install"
        else
            install_method="build"
        fi
        current_release=`sw_vers -productVersion | grep -o "[0-9]\+\.[0-9]\+" | head -1`
        current_major_release=(${current_release//./ })
        case "$current_major_release" in
            # Grouping 10 releases as Catalina 
            "10") current_release_nickname="<=Catalina";;
            "11") current_release_nickname="Big Sur";;
            "12") current_release_nickname="Monterey";;
            *)
              echo "Unknown version of OSX detected: $current_major_release"
              current_release_nickname="Unknown"
              ;;
        esac

        pkg_fmt="N/A"
        local_install_method="N/A"


    elif [[ "$current_platform" == "mingw" || "$current_platform" == "cygwin" ]]; then
        # No current plan to support Windows
        echo "Unsupported platform detected: $current_platform"
        exit 1
    fi


    echo "Distro: $current_distro"
    echo "Release: $current_release"
    echo "Major release: $current_major_release"
    echo "Release nickname: $current_release_nickname"
    echo "Package architecture: $current_pkg_arch"
    echo "Package format: $pkg_fmt"
    echo "Package manager: $pkg_mgr"
    echo "Install method: $install_method"
    echo "Local install method: $local_install_method"
    echo '-------------------------------------------------'
    echo ""
    echo ""
}

get_environment (){
    # Run all scripts in order
    get_platform
    get_cpu_info
    setup_build_info
}

get_environment
