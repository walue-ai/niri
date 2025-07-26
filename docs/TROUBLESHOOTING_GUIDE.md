# Niri Sorun Giderme Klavuzu

Bu klavuz, niri kullanımında karşılaşılan yaygın sorunları ve çözümlerini içerir.

## İçindekiler

1. [Başlatma Sorunları](#başlatma-sorunları)
2. [Yapılandırma Sorunları](#yapılandırma-sorunları)
3. [Input/Girdi Sorunları](#inputgirdi-sorunları)
4. [Sunshine Streaming Sorunları](#sunshine-streaming-sorunları)
5. [Performans Sorunları](#performans-sorunları)
6. [Sistem Entegrasyonu Sorunları](#sistem-entegrasyonu-sorunları)

## Başlatma Sorunları

### 1. Display Manager'dan Niri Seçildiğinde Login Ekranına Dönüyor

**Belirtiler:**
- GDM/SDDM'den niri seçildiğinde hemen login ekranına geri dönüyor
- Diğer compositor'lar (sway, weston) normal çalışıyor

**Kök Neden:**
Systemd servis dosyasında yanlış binary yolu

**Çözüm:**
```bash
# Servis dosyasını kontrol etme
cat /usr/local/lib/systemd/user/niri.service | grep ExecStart

# Yanlış: ExecStart=/usr/bin/niri --session
# Doğru: ExecStart=/usr/local/bin/niri --session

# Düzeltme
sudo nano /usr/local/lib/systemd/user/niri.service
# ExecStart satırını düzeltin

# Daemon yeniden yükleme
systemctl --user daemon-reload
```

**Doğrulama:**
```bash
# Systemd loglarını kontrol etme
journalctl --user -u niri --since "5 minutes ago"
```

### 2. TTY'den Başlatma Sırasında Siyah Ekran

**Belirtiler:**
- `niri` komutu çalışıyor ama ekran siyah
- Wayland socket oluşturuluyor ama görüntü yok

**Olası Nedenler ve Çözümler:**

#### A. Render Device Sorunu
```bash
# Mevcut DRI cihazlarını listeleme
ls -l /dev/dri/

# Config dosyasında render device belirtme
nano ~/.config/niri/config.kdl

# Eklenecek:
debug {
    render-drm-device "/dev/dri/renderD128"
}
```

#### B. NVIDIA Driver Sorunu
```bash
# Kernel modesetting etkinleştirme
# /etc/default/grub dosyasına ekleyin:
# GRUB_CMDLINE_LINUX="nvidia-drm.modeset=1"

sudo update-grub
sudo reboot
```

### 3. Çoklu Niri Süreçleri

**Belirtiler:**
- `ps aux | grep niri` komutu 10+ süreç gösteriyor
- Sistem yavaşlığı
- Super tuş kombinasyonları çalışmıyor

**Çözüm:**
```bash
# Tüm niri süreçlerini durdurma
systemctl --user stop niri
pkill -f niri

# Systemd servisini devre dışı bırakma (geçici)
systemctl --user disable niri

# Temiz başlatma
systemctl --user enable niri
systemctl --user start niri

# Süreç sayısını kontrol etme
ps aux | grep niri | grep -v grep | wc -l
# Sonuç 1-3 arası olmalı
```

## Yapılandırma Sorunları

### 1. Config Dosyası Yeniden Yüklenmemiş

**Belirtiler:**
- Config değişiklikleri etkili olmuyor
- `niri msg action reload-config` çalışmıyor

**Çözüm:**
```bash
# IPC socket kontrolü
ls -la /run/user/$(id -u)/niri.*

# NIRI_SOCKET environment variable
export NIRI_SOCKET=/run/user/$(id -u)/niri.wayland-1.*.sock

# Config syntax kontrolü
niri validate ~/.config/niri/config.kdl

# Tam yeniden başlatma
systemctl --user restart niri
```

### 2. KDL Syntax Hataları

**Yaygın Hatalar:**
```kdl
// YANLIŞ
binds {
    Mod+T spawn "foot"  // Eksik süslü parantez
}

// DOĞRU
binds {
    Mod+T { spawn "foot"; }
}
```

**Syntax Doğrulama:**
```bash
niri validate ~/.config/niri/config.kdl
```

## Input/Girdi Sorunları

### 1. Super Tuş Kombinasyonları Çalışmıyor

**Belirtiler:**
- Super+T, Super+B gibi kombinasyonlar yanıt vermiyor
- `showkey` ile keycode 125 görülüyor ama niri algılamıyor

**Çözüm A: Modifier Değiştirme**
```kdl
binds {
    // "Mod" yerine "Super" kullanın
    Super+T { spawn "foot"; }
    Super+B { spawn "onboard"; }
    
    // Veya "Mod4" deneyin
    Mod4+T { spawn "foot"; }
}
```

**Çözüm B: XKB Yapılandırması**
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

**Debug:**
```bash
# Input event'lerini izleme
sudo evtest

# Niri input debug
RUST_LOG=niri::input=debug niri
```

### 2. Türkçe Klavye Düzeni Sorunları

**Çözüm:**
```kdl
input {
    keyboard {
        xkb {
            layout "tr"
            variant ""
            options "grp:alt_shift_toggle,compose:ralt"
        }
    }
}
```

## Sunshine Streaming Sorunları

### 1. Sunshine Ekran Göstermiyor Ama Input Çalışıyor

**Kök Neden:**
Sunshine yanlış Wayland session'ında başlatılmış

**Çözüm:**
```bash
# Mevcut Sunshine süreçlerini durdurma
pkill -f sunshine

# Doğru environment ile başlatma
export WAYLAND_DISPLAY=wayland-1
export XDG_SESSION_TYPE=wayland
cd /home/$(whoami)
nohup sunshine > /tmp/sunshine_niri.log 2>&1 &

# Log kontrolü
tail -f /tmp/sunshine_niri.log
```

### 2. Sunshine Port Binding Hatası

**Hata:**
```
Fatal: Couldn't bind RTSP server to port [48010], Address already in use
```

**Çözüm:**
```bash
# Port kullanan süreçleri bulma
sudo netstat -tulpn | grep 48010
sudo lsof -i :48010

# Eski Sunshine süreçlerini temizleme
sudo pkill -9 -f sunshine
sleep 2

# Yeniden başlatma
sunshine
```

### 3. Moonlight Key Mapping Sorunu

**Sorun:**
MacBook'ta Cmd tuşu Linux Super tuşuna map edilmiyor

**Çözümler:**

#### A. Moonlight Ayarları
- Settings → Input → "Optimize game settings" kapatın
- "Capture system keys" etkinleştirin

#### B. Alternatif Test Yöntemleri
```bash
# SSH ile manuel komutlar
ssh user@remote_ip "export WAYLAND_DISPLAY=wayland-1 && niri msg action spawn -- onboard"
ssh user@remote_ip "export WAYLAND_DISPLAY=wayland-1 && niri msg action spawn -- foot"
```

#### C. Fiziksel Klavye
USB klavye bağlayıp Super tuşunu test edin

### 4. H.264 Encoder Bulunamıyor

**Hata:**
```
No H.264 encoder found
```

**Çözüm:**
```bash
# VAAPI desteği kontrol etme
vainfo

# Mesa VAAPI driver kurulumu
sudo apt install mesa-va-drivers

# Intel için
sudo apt install intel-media-va-driver

# NVIDIA için
sudo apt install nvidia-vaapi-driver
```

## Performans Sorunları

### 1. Yüksek CPU Kullanımı

**Tanı:**
```bash
# Niri süreçlerini izleme
top -p $(pgrep niri)

# Tracy profiling (geliştirme)
cargo build --release --features=profile-with-tracy-ondemand
```

**Çözümler:**
- Animasyonları azaltın
- Refresh rate'i düşürün
- Fractional scaling'i kapatın

### 2. Bellek Sızıntısı

**Tanı:**
```bash
# Bellek kullanımını izleme
watch -n 1 'ps aux | grep niri | grep -v grep'
```

**Çözüm:**
```bash
# Niri'yi periyodik yeniden başlatma (geçici)
systemctl --user restart niri
```

## Sistem Entegrasyonu Sorunları

### 1. Portal Servisleri Çalışmıyor

**Belirtiler:**
- Screensharing çalışmıyor
- File picker açılmıyor

**Çözüm:**
```bash
# Portal yapılandırmasını kontrol etme
cat /usr/local/share/xdg-desktop-portal/niri-portals.conf

# Portal servislerini yeniden başlatma
systemctl --user restart xdg-desktop-portal
systemctl --user restart xdg-desktop-portal-gnome
```

### 2. D-Bus Servisleri

**Çözüm:**
```bash
# D-Bus session kontrolü
echo $DBUS_SESSION_BUS_ADDRESS

# Servis durumunu kontrol etme
systemctl --user status dbus
```

### 3. Systemd User Session

**Çözüm:**
```bash
# User session durumu
systemctl --user status

# Graphical session target
systemctl --user status graphical-session.target
```

## Debug ve Log Toplama

### Detaylı Logging

```bash
# Debug seviyesinde log
RUST_LOG=niri=debug niri 2>&1 | tee niri-debug.log

# Belirli modüller için
RUST_LOG=niri::input=debug,niri::layout=debug niri

# Systemd journal
journalctl --user -f -u niri
```

### Sistem Bilgisi Toplama

```bash
# Sistem bilgileri
uname -a
lscpu
lspci | grep -i vga
free -h

# Wayland bilgileri
echo $WAYLAND_DISPLAY
echo $XDG_SESSION_TYPE
ls -la /run/user/$(id -u)/wayland-*

# Niri durumu
niri --version
ps aux | grep niri | grep -v grep
systemctl --user status niri
```

## Acil Durum Kurtarma

### TTY'ye Geçiş

```bash
# Ctrl+Alt+F2 ile TTY2'ye geçin
# Login yapın

# Niri'yi durdurma
systemctl --user stop niri
pkill -f niri

# GDM'yi yeniden başlatma
sudo systemctl restart gdm3
```

### Yapılandırma Sıfırlama

```bash
# Config yedekleme
cp ~/.config/niri/config.kdl ~/.config/niri/config.kdl.broken

# Varsayılan config
cp /path/to/niri/resources/default-config.kdl ~/.config/niri/config.kdl
```

Bu sorun giderme klavuzu, niri kullanımında karşılaşabileceğiniz çoğu sorunu çözmenize yardımcı olacaktır. Sorun devam ederse, niri topluluğundan yardım alabilirsiniz.
