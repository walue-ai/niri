# Niri ile Sunshine Streaming Entegrasyonu

Bu klavuz, niri Wayland compositor'ü ile Sunshine streaming yazılımının entegrasyonunu ve sorun giderme süreçlerini açıklar.

## İçindekiler

1. [Sunshine Kurulumu](#sunshine-kurulumu)
2. [Niri Session Entegrasyonu](#niri-session-entegrasyonu)
3. [Yapılandırma](#yapılandırma)
4. [Sorun Giderme](#sorun-giderme)
5. [Moonlight Client Ayarları](#moonlight-client-ayarları)
6. [Performans Optimizasyonu](#performans-optimizasyonu)

## Sunshine Kurulumu

### Ubuntu/Debian için Kurulum

```bash
# En son sürümü indirme
wget https://github.com/LizardByte/Sunshine/releases/latest/download/sunshine-ubuntu-22.04-amd64.deb

# Kurulum
sudo dpkg -i sunshine-ubuntu-22.04-amd64.deb

# Bağımlılık sorunları varsa
sudo apt-get install -f
```

### Fedora/RHEL için Kurulum

```bash
# RPM paketini indirme
wget https://github.com/LizardByte/Sunshine/releases/latest/download/sunshine-fedora-39-amd64.rpm

# Kurulum
sudo rpm -i sunshine-fedora-39-amd64.rpm
```

### Kurulum Doğrulama

```bash
# Sunshine versiyonu
sunshine --version

# Systemd servis durumu
systemctl --user status sunshine
```

## Niri Session Entegrasyonu

### Wayland Environment Ayarları

Sunshine'ın niri session'ında düzgün çalışması için doğru environment variable'lar gereklidir:

```bash
# Niri session'ında Sunshine başlatma
export WAYLAND_DISPLAY=wayland-1
export XDG_SESSION_TYPE=wayland
export XDG_CURRENT_DESKTOP=niri
```

### Manuel Başlatma

```bash
# Niri terminal'inden Sunshine başlatma
cd ~
export WAYLAND_DISPLAY=wayland-1
export XDG_SESSION_TYPE=wayland
nohup sunshine > /tmp/sunshine_niri.log 2>&1 &
```

### Otomatik Başlatma

Niri config dosyasına (`~/.config/niri/config.kdl`) ekleyin:

```kdl
spawn-at-startup {
    command ["sh", "-c", "export WAYLAND_DISPLAY=wayland-1 && export XDG_SESSION_TYPE=wayland && sunshine > /tmp/sunshine_niri.log 2>&1 &"]
}
```

## Yapılandırma

### Sunshine Web Arayüzü

1. Tarayıcıda `https://localhost:47990` adresine gidin
2. İlk kurulum sihirbazını tamamlayın
3. Kullanıcı adı ve şifre belirleyin

### Temel Ayarlar

#### Video Encoder Ayarları

```json
{
    "encoder": "h264_vaapi",
    "adapter_name": "/dev/dri/renderD128",
    "framerate": 60,
    "bitrate": 20000
}
```

#### Audio Ayarları

```json
{
    "audio_sink": "auto",
    "virtual_sink": true
}
```

#### Input Ayarları

```json
{
    "gamepad": "auto",
    "mouse": "enabled",
    "keyboard": "enabled"
}
```

### Niri Özel Ayarları

#### Display Capture

Sunshine'ın niri ekranını yakalayabilmesi için:

```bash
# Wayland socket'in doğru olduğunu kontrol edin
echo $WAYLAND_DISPLAY
# Çıktı: wayland-1 olmalı

# XDG portal'ların çalıştığını kontrol edin
systemctl --user status xdg-desktop-portal
systemctl --user status xdg-desktop-portal-gnome
```

## Sorun Giderme

### 1. Sunshine Ekran Göstermiyor

**Belirtiler:**
- Moonlight bağlanıyor ama siyah ekran
- Input çalışıyor ama görüntü yok

**Çözüm:**
```bash
# Mevcut Sunshine süreçlerini durdurma
pkill -f sunshine

# Doğru environment ile yeniden başlatma
export WAYLAND_DISPLAY=wayland-1
export XDG_SESSION_TYPE=wayland
cd ~
sunshine > /tmp/sunshine_debug.log 2>&1 &

# Log kontrolü
tail -f /tmp/sunshine_debug.log
```

### 2. Input Kontrolü Çalışmıyor

**Belirtiler:**
- Ekran görünüyor ama mouse/keyboard çalışmıyor
- Moonlight'ta input lag var

**Çözüm:**
```bash
# Input cihazlarının algılandığını kontrol etme
ls -la /dev/input/

# Sunshine'ı root yetkisiyle çalıştırma (geçici çözüm)
sudo sunshine

# Udev kuralları ekleme (kalıcı çözüm)
sudo usermod -a -G input $USER
```

### 3. Port Binding Hatası

**Hata:**
```
Fatal: Couldn't bind RTSP server to port [48010], Address already in use
```

**Çözüm:**
```bash
# Port kullanan süreçleri bulma
sudo netstat -tulpn | grep 48010
sudo lsof -i :48010

# Eski süreçleri temizleme
sudo pkill -9 -f sunshine
sleep 2

# Yeniden başlatma
sunshine
```

### 4. H.264 Encoder Bulunamıyor

**Hata:**
```
No H.264 encoder found
```

**Çözüm:**
```bash
# VAAPI desteği kontrol etme
vainfo

# Intel için driver kurulumu
sudo apt install intel-media-va-driver mesa-va-drivers

# NVIDIA için
sudo apt install nvidia-vaapi-driver

# AMD için
sudo apt install mesa-va-drivers
```

### 5. Audio Sync Sorunu

**Çözüm:**
```bash
# PipeWire durumu kontrol etme
systemctl --user status pipewire

# Audio sink'leri listeleme
pactl list sinks

# Sunshine config'de audio sink belirtme
# "audio_sink": "alsa_output.pci-0000_00_1f.3.analog-stereo"
```

## Moonlight Client Ayarları

### MacBook'ta Key Mapping

**Sorun:** MacBook Cmd tuşu Linux Super tuşuna map edilmiyor

**Çözümler:**

#### Moonlight Ayarları
1. Settings → Input
2. "Optimize game settings" kapatın
3. "Capture system keys" etkinleştirin
4. "Mouse acceleration" kapatın

#### Alternatif Çözümler
```bash
# SSH ile manuel komutlar
ssh user@niri_server "export WAYLAND_DISPLAY=wayland-1 && niri msg action spawn -- onboard"
ssh user@niri_server "export WAYLAND_DISPLAY=wayland-1 && niri msg action spawn -- foot"

# Fiziksel USB klavye kullanma
# Bluetooth klavye pairing
```

### Network Optimizasyonu

#### Moonlight Ayarları
- Resolution: 1920x1080 (başlangıç için)
- FPS: 60
- Bitrate: 20 Mbps (ağ hızına göre ayarlayın)
- Hardware decoding: Etkinleştirin

#### Ağ Testi
```bash
# Ping testi
ping -c 10 niri_server_ip

# Bandwidth testi
iperf3 -c niri_server_ip

# Packet loss kontrolü
mtr niri_server_ip
```

## Performans Optimizasyonu

### Sunshine Ayarları

#### Düşük Latency için
```json
{
    "min_threads": 2,
    "hevc_mode": 0,
    "av1_mode": 0,
    "fps": 60,
    "resolutions": [
        {"width": 1920, "height": 1080}
    ]
}
```

#### Yüksek Kalite için
```json
{
    "encoder": "h264_nvenc",
    "bitrate": 50000,
    "fps": 120,
    "hevc_mode": 2
}
```

### Sistem Optimizasyonu

#### CPU Governor
```bash
# Performance mode
sudo cpupower frequency-set -g performance

# Kontrol
cpupower frequency-info
```

#### GPU Ayarları
```bash
# Intel GPU frequency
sudo intel_gpu_frequency -s 1200

# NVIDIA GPU
nvidia-smi -pm 1
nvidia-smi -ac 4004,1911
```

### Niri Optimizasyonu

#### Config Ayarları
```kdl
// Animasyonları azaltma
animations {
    slowdown 0.5
    
    window-open {
        duration-ms 150
    }
    
    window-close {
        duration-ms 150
    }
}

// VSync ayarları
environment {
    NIRI_DISABLE_VSYNC "1"
}
```

## Monitoring ve Debug

### Log Analizi

```bash
# Sunshine logları
tail -f /tmp/sunshine_niri.log

# Niri logları
journalctl --user -f -u niri

# System logları
journalctl -f | grep -i sunshine
```

### Performance Metrikleri

```bash
# GPU kullanımı
nvidia-smi -l 1

# CPU kullanımı
htop -p $(pgrep sunshine)

# Network trafiği
iftop -i eth0

# Disk I/O
iotop -p $(pgrep sunshine)
```

### Debug Komutları

```bash
# Wayland debug
WAYLAND_DEBUG=1 sunshine

# Sunshine debug
SUNSHINE_LOG_LEVEL=debug sunshine

# Niri input debug
RUST_LOG=niri::input=debug niri
```

## Güvenlik

### Firewall Ayarları

```bash
# Sunshine portlarını açma
sudo ufw allow 47989:47990/tcp
sudo ufw allow 48010/tcp

# Sadece belirli IP'den erişim
sudo ufw allow from 192.168.1.100 to any port 47989:47990
```

### SSL Sertifikası

```bash
# Self-signed sertifika oluşturma
openssl req -x509 -newkey rsa:4096 -keyout sunshine.key -out sunshine.crt -days 365 -nodes

# Sunshine config'de sertifika yolu belirtme
```

Bu entegrasyon klavuzu, niri ile Sunshine'ı başarıyla kullanmanız için gereken tüm bilgileri içerir. Sorunlarla karşılaştığınızda, adım adım sorun giderme bölümünü takip edin.
