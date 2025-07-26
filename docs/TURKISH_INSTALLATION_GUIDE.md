# Niri Kurulum ve Yapılandırma Klavuzu

Bu klavuz, niri Wayland compositor'ünü kaynak koddan derleme, kurulum ve yapılandırma sürecini detaylı olarak açıklar.

## İçindekiler

1. [Sistem Gereksinimleri](#sistem-gereksinimleri)
2. [Kaynak Koddan Derleme](#kaynak-koddan-derleme)
3. [Kurulum](#kurulum)
4. [Yapılandırma](#yapılandırma)
5. [Sorun Giderme](#sorun-giderme)
6. [Sunshine Entegrasyonu](#sunshine-entegrasyonu)
7. [SSH ile Uzaktan Kurulum](#ssh-ile-uzaktan-kurulum)

## Sistem Gereksinimleri

### Ubuntu 22.04/24.04 için Gerekli Paketler

```bash
sudo apt-get update
sudo apt-get install -y gcc clang libudev-dev libgbm-dev libxkbcommon-dev \
    libegl1-mesa-dev libwayland-dev libinput-dev libdbus-1-dev \
    libsystemd-dev libseat-dev libpipewire-0.3-dev libpango1.0-dev \
    libdisplay-info-dev build-essential pkg-config
```

### Rust Kurulumu

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup update
```

## Kaynak Koddan Derleme

### 1. Niri Deposunu Klonlama

```bash
git clone https://github.com/YaLTeR/niri.git
cd niri
```

### 2. Derleme

```bash
# Release modunda derleme (önerilen)
cargo build --release

# Derleme sürecini kontrol etme
ls -la target/release/niri
```

**Önemli:** `--all-features` ile derleme yapmayın! Bazı özellikler sadece geliştirme amaçlıdır.

### 3. Derleme Süresi

- İlk derleme: ~10-15 dakika (bağımlılıklar dahil)
- Sonraki derlemeler: ~2-5 dakika

## Kurulum

### Manuel Kurulum (Önerilen)

```bash
# Binary dosyalarını kopyalama
sudo cp target/release/niri /usr/local/bin/
sudo cp resources/niri-session /usr/local/bin/
sudo chmod +x /usr/local/bin/niri /usr/local/bin/niri-session

# Desktop session dosyası
sudo cp resources/niri.desktop /usr/local/share/wayland-sessions/

# Portal yapılandırması
sudo cp resources/niri-portals.conf /usr/local/share/xdg-desktop-portal/

# Systemd servisleri
sudo cp resources/niri.service /usr/local/lib/systemd/user/
sudo cp resources/niri-shutdown.target /usr/local/lib/systemd/user/

# Systemd daemon yeniden yükleme
systemctl --user daemon-reload
```

### Kurulum Doğrulama

```bash
# Niri versiyonunu kontrol etme
niri --version

# Systemd servis dosyasını kontrol etme
systemctl --user status niri
```

## Yapılandırma

### Temel Yapılandırma Dosyası

Niri yapılandırma dosyası: `~/.config/niri/config.kdl`

```bash
# Yapılandırma dizinini oluşturma
mkdir -p ~/.config/niri

# Varsayılan yapılandırmayı kopyalama
cp resources/default-config.kdl ~/.config/niri/config.kdl
```

### Önemli Yapılandırma Değişiklikleri

#### 1. Super Tuş Kombinasyonları

```kdl
binds {
    // Terminal açma (foot kullanarak)
    Mod+T { spawn "foot"; }
    
    // Ekran klavyesi açma
    Mod+B { spawn "onboard"; }
    
    // Uygulama başlatıcı
    Mod+D { spawn "fuzzel"; }
}
```

#### 2. Otomatik Başlatma

```kdl
spawn-at-startup {
    command ["foot"]
}
```

#### 3. Girdi Cihazları

```kdl
input {
    keyboard {
        xkb {
            layout "tr"
            variant ""
        }
    }
    
    touchpad {
        tap true
        natural-scroll true
    }
}
```

### Yapılandırma Yeniden Yükleme

```bash
# Canlı yapılandırma yeniden yükleme
niri msg action reload-config

# Tam yeniden başlatma (gerekirse)
systemctl --user restart niri
```

## Sorun Giderme

### 1. Display Manager'dan Başlatma Sorunu

**Sorun:** Niri seçildiğinde hemen login ekranına dönüyor.

**Çözüm:** Systemd servis dosyasındaki yol hatası

```bash
# Servis dosyasını düzenleme
sudo nano /usr/local/lib/systemd/user/niri.service

# ExecStart satırını kontrol etme
ExecStart=/usr/local/bin/niri --session

# Daemon yeniden yükleme
systemctl --user daemon-reload
```

### 2. Çoklu Niri Süreçleri

**Sorun:** Birden fazla niri süreci çalışıyor.

**Çözüm:**
```bash
# Tüm niri süreçlerini durdurma
systemctl --user stop niri
pkill -f niri

# Temiz başlatma
systemctl --user start niri
```

### 3. Super Tuş Algılanmıyor

**Sorun:** Super tuş kombinasyonları çalışmıyor.

**Çözüm:** Config dosyasında modifier kontrolü

```kdl
binds {
    // "Mod" yerine "Super" kullanma
    Super+T { spawn "foot"; }
    Super+B { spawn "onboard"; }
}
```

### 4. Türkçe Klavye Düzeni

```kdl
input {
    keyboard {
        xkb {
            layout "tr"
            options "grp:alt_shift_toggle"
        }
    }
}
```

## Sunshine Entegrasyonu

### Sunshine Kurulumu

```bash
# Sunshine'ı kurun (dağıtımınıza göre)
# Ubuntu için:
wget https://github.com/LizardByte/Sunshine/releases/latest/download/sunshine-ubuntu-22.04-amd64.deb
sudo dpkg -i sunshine-ubuntu-22.04-amd64.deb
```

### Niri Session'ında Sunshine Başlatma

```bash
# Doğru Wayland environment ile başlatma
export WAYLAND_DISPLAY=wayland-1
export XDG_SESSION_TYPE=wayland
sunshine > /tmp/sunshine_niri.log 2>&1 &
```

### Sunshine Yapılandırması

1. Web arayüzü: `https://localhost:47990`
2. Input cihazlarının doğru algılandığını kontrol edin
3. Video encoder ayarlarını kontrol edin (h264_vaapi önerilen)

### Moonlight Key Mapping Sorunu

**Sorun:** MacBook'ta Cmd tuşu Linux Super tuşuna map edilmiyor.

**Çözümler:**
1. Moonlight ayarlarında "Capture system keys" etkinleştirin
2. "Optimize game settings" kapatın
3. Fiziksel USB klavye kullanın
4. SSH ile manuel komutlar:
   ```bash
   niri msg action spawn -- onboard
   niri msg action spawn -- foot
   ```

## SSH ile Uzaktan Kurulum

### SSH Key Kurulumu

```bash
# Yerel makinede SSH key oluşturma
ssh-keygen -t ed25519 -f ~/.ssh/niri_deploy_key

# Public key'i uzak makineye kopyalama
ssh-copy-id -i ~/.ssh/niri_deploy_key.pub user@remote_ip

# Bağlantı testi
ssh -i ~/.ssh/niri_deploy_key user@remote_ip "echo 'SSH connection successful'"
```

### Uzaktan Derleme ve Kurulum

```bash
# Uzak makinede niri klonlama ve derleme
ssh -i ~/.ssh/niri_deploy_key user@remote_ip "
    git clone https://github.com/YaLTeR/niri.git &&
    cd niri &&
    cargo build --release
"

# Kurulum dosyalarını kopyalama
ssh -i ~/.ssh/niri_deploy_key user@remote_ip "
    cd niri &&
    sudo cp target/release/niri /usr/local/bin/ &&
    sudo cp resources/niri-session /usr/local/bin/ &&
    sudo cp resources/niri.desktop /usr/local/share/wayland-sessions/ &&
    sudo cp resources/niri-portals.conf /usr/local/share/xdg-desktop-portal/ &&
    sudo cp resources/niri.service /usr/local/lib/systemd/user/ &&
    sudo cp resources/niri-shutdown.target /usr/local/lib/systemd/user/ &&
    systemctl --user daemon-reload
"
```

## Güncelleme

### Niri Güncelleme

```bash
cd niri
git pull origin main
cargo build --release

# Yeni binary'yi kurma
sudo cp target/release/niri /usr/local/bin/

# Niri'yi yeniden başlatma
systemctl --user restart niri
```

### Yapılandırma Yedekleme

```bash
# Yapılandırma yedekleme
cp ~/.config/niri/config.kdl ~/.config/niri/config.kdl.backup

# Yedekten geri yükleme
cp ~/.config/niri/config.kdl.backup ~/.config/niri/config.kdl
```

## Yararlı Komutlar

```bash
# Niri durumunu kontrol etme
systemctl --user status niri

# Niri loglarını görme
journalctl --user -f -u niri

# IPC komutları
niri msg action reload-config
niri msg action spawn -- foot
niri msg action spawn -- onboard

# Wayland socket kontrolü
ls -la /run/user/$(id -u)/wayland-*

# Niri süreçlerini listeleme
ps aux | grep niri | grep -v grep
```

## Destek ve Yardım

- **Matrix Kanalı:** https://matrix.to/#/#niri:matrix.org
- **GitHub Issues:** https://github.com/YaLTeR/niri/issues
- **Wiki:** https://github.com/YaLTeR/niri/wiki

Bu klavuz, niri'yi başarıyla kurup yapılandırmanız için gereken tüm adımları içerir. Herhangi bir sorunla karşılaştığınızda, sorun giderme bölümünü kontrol edin veya topluluk desteği alın.
