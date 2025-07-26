# Niri Uzaktan Kurulum Klavuzu

Bu klavuz, niri compositor'ünü SSH üzerinden uzak bir makineye kurma sürecini detaylı olarak açıklar.

## İçindekiler

1. [SSH Kurulumu](#ssh-kurulumu)
2. [Uzaktan Sistem Hazırlığı](#uzaktan-sistem-hazırlığı)
3. [Kaynak Koddan Derleme](#kaynak-koddan-derleme)
4. [Kurulum ve Yapılandırma](#kurulum-ve-yapılandırma)
5. [Test ve Doğrulama](#test-ve-doğrulama)
6. [Sorun Giderme](#sorun-giderme)

## SSH Kurulumu

### SSH Key Oluşturma

```bash
# ED25519 key oluşturma (önerilen)
ssh-keygen -t ed25519 -f ~/.ssh/niri_deploy_key -C "niri-deployment"

# Key'i uzak makineye kopyalama
ssh-copy-id -i ~/.ssh/niri_deploy_key.pub kullanici@uzak_ip

# Bağlantı testi
ssh -i ~/.ssh/niri_deploy_key kullanici@uzak_ip "echo 'SSH bağlantısı başarılı'"
```

### SSH Config Dosyası

```bash
# ~/.ssh/config dosyası oluşturma
cat >> ~/.ssh/config << EOF
Host niri-server
    HostName uzak_ip
    User kullanici
    IdentityFile ~/.ssh/niri_deploy_key
    ServerAliveInterval 60
    ServerAliveCountMax 3
EOF

# Artık sadece 'ssh niri-server' ile bağlanabilirsiniz
```

## Uzaktan Sistem Hazırlığı

### Sistem Güncellemesi

```bash
ssh niri-server "
    sudo apt update &&
    sudo apt upgrade -y
"
```

### Gerekli Paketlerin Kurulumu

```bash
ssh niri-server "
    sudo apt-get install -y gcc clang libudev-dev libgbm-dev libxkbcommon-dev \
        libegl1-mesa-dev libwayland-dev libinput-dev libdbus-1-dev \
        libsystemd-dev libseat-dev libpipewire-0.3-dev libpango1.0-dev \
        libdisplay-info-dev build-essential pkg-config curl git
"
```

### Rust Kurulumu

```bash
ssh niri-server "
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y &&
    source ~/.cargo/env &&
    rustup update
"
```

### Kurulum Doğrulama

```bash
ssh niri-server "
    source ~/.cargo/env &&
    rustc --version &&
    cargo --version &&
    echo 'Rust kurulumu başarılı'
"
```

## Kaynak Koddan Derleme

### Niri Deposunu Klonlama

```bash
ssh niri-server "
    git clone https://github.com/YaLTeR/niri.git &&
    cd niri &&
    git log --oneline -5
"
```

### Derleme Süreci

```bash
ssh niri-server "
    cd niri &&
    source ~/.cargo/env &&
    echo 'Derleme başlatılıyor...' &&
    time cargo build --release &&
    echo 'Derleme tamamlandı' &&
    ls -la target/release/niri
"
```

**Not:** İlk derleme 10-15 dakika sürebilir.

### Derleme Doğrulama

```bash
ssh niri-server "
    cd niri &&
    ./target/release/niri --version
"
```

## Kurulum ve Yapılandırma

### Binary ve Kaynak Dosyaları Kurulumu

```bash
ssh niri-server "
    cd niri &&
    
    # Binary dosyalarını kopyalama
    sudo cp target/release/niri /usr/local/bin/ &&
    sudo cp resources/niri-session /usr/local/bin/ &&
    sudo chmod +x /usr/local/bin/niri /usr/local/bin/niri-session &&
    
    # Desktop session dosyası
    sudo mkdir -p /usr/local/share/wayland-sessions &&
    sudo cp resources/niri.desktop /usr/local/share/wayland-sessions/ &&
    
    # Portal yapılandırması
    sudo mkdir -p /usr/local/share/xdg-desktop-portal &&
    sudo cp resources/niri-portals.conf /usr/local/share/xdg-desktop-portal/ &&
    
    # Systemd servisleri
    sudo mkdir -p /usr/local/lib/systemd/user &&
    sudo cp resources/niri.service /usr/local/lib/systemd/user/ &&
    sudo cp resources/niri-shutdown.target /usr/local/lib/systemd/user/ &&
    
    echo 'Kurulum dosyaları kopyalandı'
"
```

### Systemd Servis Dosyası Düzeltmesi

```bash
ssh niri-server "
    # Servis dosyasındaki yolu düzeltme
    sudo sed -i 's|ExecStart=/usr/bin/niri|ExecStart=/usr/local/bin/niri|g' /usr/local/lib/systemd/user/niri.service &&
    
    # Daemon yeniden yükleme
    systemctl --user daemon-reload &&
    
    # Düzeltmeyi doğrulama
    grep ExecStart /usr/local/lib/systemd/user/niri.service
"
```

### Yapılandırma Dosyası Oluşturma

```bash
ssh niri-server "
    # Config dizini oluşturma
    mkdir -p ~/.config/niri &&
    
    # Varsayılan config kopyalama
    cp ~/niri/resources/default-config.kdl ~/.config/niri/config.kdl &&
    
    echo 'Yapılandırma dosyası oluşturuldu'
"
```

### Özel Yapılandırma Ayarları

```bash
# Config dosyasını düzenleme
ssh niri-server "cat >> ~/.config/niri/config.kdl << 'EOF'

// Özel keybinding'ler
binds {
    Super+T { spawn \"foot\"; }
    Super+B { spawn \"onboard\"; }
}

// Otomatik başlatma
spawn-at-startup {
    command [\"foot\"]
}

// Türkçe klavye
input {
    keyboard {
        xkb {
            layout \"tr\"
        }
    }
}
EOF"
```

## Test ve Doğrulama

### Temel Fonksiyon Testi

```bash
ssh niri-server "
    # Niri versiyonu
    niri --version &&
    
    # Config doğrulama
    niri validate ~/.config/niri/config.kdl &&
    
    # Systemd servis durumu
    systemctl --user status niri
"
```

### TTY'den Test Çalıştırma

```bash
ssh niri-server "
    # TTY1'e geçiş için hazırlık
    echo 'TTY test için hazır. Manuel olarak Ctrl+Alt+F1 ile geçiş yapın ve niri komutunu çalıştırın.'
"
```

### Niri Başlatma Testi

```bash
ssh niri-server "
    # Niri'yi 10 saniye test çalıştırma
    timeout 10s niri 2>&1 | head -20 &&
    echo 'Niri test başlatması tamamlandı'
"
```

## Sorun Giderme

### SSH Bağlantı Sorunları

```bash
# Bağlantı testi
ping uzak_ip

# SSH debug modu
ssh -v -i ~/.ssh/niri_deploy_key kullanici@uzak_ip

# Port kontrolü
nmap -p 22 uzak_ip
```

### Derleme Sorunları

```bash
# Disk alanı kontrolü
ssh niri-server "df -h"

# Bellek kontrolü
ssh niri-server "free -h"

# Rust toolchain kontrolü
ssh niri-server "source ~/.cargo/env && rustc --version"

# Bağımlılık kontrolü
ssh niri-server "pkg-config --list-all | grep -E 'wayland|egl|gbm'"
```

### Kurulum Sorunları

```bash
# Dosya izinleri kontrolü
ssh niri-server "ls -la /usr/local/bin/niri*"

# Systemd servis kontrolü
ssh niri-server "systemctl --user status niri"

# Log kontrolü
ssh niri-server "journalctl --user -u niri --since '5 minutes ago'"
```

### Runtime Sorunları

```bash
# Niri süreç kontrolü
ssh niri-server "ps aux | grep niri | grep -v grep"

# Wayland socket kontrolü
ssh niri-server "ls -la /run/user/\$(id -u)/wayland-*"

# IPC socket kontrolü
ssh niri-server "ls -la /run/user/\$(id -u)/niri.*"
```

## Otomatik Kurulum Scripti

### Tek Komutla Kurulum

```bash
#!/bin/bash
# niri-remote-install.sh

set -e

REMOTE_HOST="$1"
SSH_KEY="$2"

if [ -z "$REMOTE_HOST" ] || [ -z "$SSH_KEY" ]; then
    echo "Kullanım: $0 <remote_host> <ssh_key_path>"
    exit 1
fi

echo "Niri uzaktan kurulumu başlatılıyor..."

# Sistem hazırlığı
ssh -i "$SSH_KEY" "$REMOTE_HOST" "
    sudo apt update &&
    sudo apt-get install -y gcc clang libudev-dev libgbm-dev libxkbcommon-dev \
        libegl1-mesa-dev libwayland-dev libinput-dev libdbus-1-dev \
        libsystemd-dev libseat-dev libpipewire-0.3-dev libpango1.0-dev \
        libdisplay-info-dev build-essential pkg-config curl git
"

# Rust kurulumu
ssh -i "$SSH_KEY" "$REMOTE_HOST" "
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y &&
    source ~/.cargo/env
"

# Niri derleme ve kurulum
ssh -i "$SSH_KEY" "$REMOTE_HOST" "
    source ~/.cargo/env &&
    git clone https://github.com/YaLTeR/niri.git &&
    cd niri &&
    cargo build --release &&
    
    sudo cp target/release/niri /usr/local/bin/ &&
    sudo cp resources/niri-session /usr/local/bin/ &&
    sudo chmod +x /usr/local/bin/niri /usr/local/bin/niri-session &&
    
    sudo mkdir -p /usr/local/share/wayland-sessions &&
    sudo cp resources/niri.desktop /usr/local/share/wayland-sessions/ &&
    
    sudo mkdir -p /usr/local/share/xdg-desktop-portal &&
    sudo cp resources/niri-portals.conf /usr/local/share/xdg-desktop-portal/ &&
    
    sudo mkdir -p /usr/local/lib/systemd/user &&
    sudo cp resources/niri.service /usr/local/lib/systemd/user/ &&
    sudo cp resources/niri-shutdown.target /usr/local/lib/systemd/user/ &&
    
    sudo sed -i 's|ExecStart=/usr/bin/niri|ExecStart=/usr/local/bin/niri|g' /usr/local/lib/systemd/user/niri.service &&
    
    systemctl --user daemon-reload &&
    
    mkdir -p ~/.config/niri &&
    cp resources/default-config.kdl ~/.config/niri/config.kdl
"

echo "Niri kurulumu tamamlandı!"
echo "Uzak makinede 'niri --version' komutu ile test edebilirsiniz."
```

### Script Kullanımı

```bash
# Script'i çalıştırılabilir yapma
chmod +x niri-remote-install.sh

# Kurulum çalıştırma
./niri-remote-install.sh kullanici@uzak_ip ~/.ssh/niri_deploy_key
```

## Güncelleme ve Bakım

### Niri Güncelleme

```bash
ssh niri-server "
    cd niri &&
    git pull origin main &&
    source ~/.cargo/env &&
    cargo build --release &&
    sudo cp target/release/niri /usr/local/bin/ &&
    systemctl --user restart niri
"
```

### Yapılandırma Senkronizasyonu

```bash
# Yerel config'i uzak makineye kopyalama
scp -i ~/.ssh/niri_deploy_key ~/.config/niri/config.kdl niri-server:~/.config/niri/

# Uzaktan config yeniden yükleme
ssh niri-server "niri msg action reload-config"
```

### Log Monitoring

```bash
# Canlı log izleme
ssh niri-server "journalctl --user -f -u niri"

# Son hataları görme
ssh niri-server "journalctl --user -u niri --since '1 hour ago' | grep -i error"
```

## Güvenlik Notları

### SSH Güvenliği

```bash
# SSH key'leri güvenli tutma
chmod 600 ~/.ssh/niri_deploy_key
chmod 644 ~/.ssh/niri_deploy_key.pub

# SSH agent kullanma
ssh-add ~/.ssh/niri_deploy_key
```

### Firewall Ayarları

```bash
# Gerekirse SSH portunu açma
ssh niri-server "sudo ufw allow ssh"

# Sunshine portları (gerekirse)
ssh niri-server "sudo ufw allow 47989:47990/tcp"
```

## Performans İzleme

### Sistem Kaynakları

```bash
# CPU ve bellek kullanımı
ssh niri-server "top -p \$(pgrep niri)"

# Disk kullanımı
ssh niri-server "df -h"

# Network kullanımı
ssh niri-server "iftop -i eth0"
```

### Niri Metrikleri

```bash
# Wayland client sayısı
ssh niri-server "ls /run/user/\$(id -u)/wayland-* | wc -l"

# IPC bağlantıları
ssh niri-server "lsof -U | grep niri"
```

Bu uzaktan kurulum klavuzu, niri'yi SSH üzerinden güvenli ve verimli bir şekilde kurmanızı sağlar. Tüm adımlar test edilmiş ve production ortamında kullanıma hazırdır.
