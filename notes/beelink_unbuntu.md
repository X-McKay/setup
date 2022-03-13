# Install base packages if minimum build

```bash
sudo apt-get install build-essential -y
sudo apt install zlib
sudo apt install zlib1g-dev
```

## OPEN SSL

```
wget https://www.openssl.org/source/openssl-1.1.1g.tar.gz
tar zxvf openssl-1.1.1g.tar.gz
cd openssl-1.1.1g
./config --prefix=${HOME}/openssl --openssldir=${HOME}/openssl no-ssl2
```

# install ssh server
```bash
sudo apt update
sudo apt install openssh-server
```

# allow ssh
```bash
sudo ufw allow ssh
```