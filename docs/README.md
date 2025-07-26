# Niri Türkçe Dokümantasyon

Bu dizin, niri Wayland compositor'ü için Türkçe dokümantasyon içerir. Bu dokümantasyon, gerçek bir uzaktan kurulum ve yapılandırma deneyiminden elde edilen bilgilere dayanmaktadır.

## Dokümantasyon İçeriği

### 📚 Kullanıcı Klavuzları

- **[Türkçe Kurulum Klavuzu](TURKISH_INSTALLATION_GUIDE.md)** - Niri'yi kaynak koddan derleme, kurulum ve temel yapılandırma
- **[Sorun Giderme Klavuzu](TROUBLESHOOTING_GUIDE.md)** - Yaygın sorunlar ve çözümleri
- **[Sunshine Entegrasyon Klavuzu](SUNSHINE_INTEGRATION_GUIDE.md)** - Niri ile Sunshine streaming entegrasyonu

### 🛠️ Geliştirici Klavuzları

- **[Geliştirici Klavuzu](DEVELOPER_GUIDE.md)** - Niri geliştirme, katkıda bulunma ve özelleştirme
- **[Uzaktan Kurulum Klavuzu](REMOTE_INSTALLATION_GUIDE.md)** - SSH ile uzaktan kurulum ve yönetim

## Hızlı Başlangıç

### Temel Kurulum

```bash
# Sistem bağımlılıklarını kurma
sudo apt-get install -y gcc clang libudev-dev libgbm-dev libxkbcommon-dev \
    libegl1-mesa-dev libwayland-dev libinput-dev libdbus-1-dev \
    libsystemd-dev libseat-dev libpipewire-0.3-dev libpango1.0-dev \
    libdisplay-info-dev

# Rust kurulumu
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Niri derleme ve kurulum
git clone https://github.com/YaLTeR/niri.git
cd niri
cargo build --release
sudo cp target/release/niri /usr/local/bin/
```

Detaylı kurulum talimatları için [Türkçe Kurulum Klavuzu](TURKISH_INSTALLATION_GUIDE.md)'na bakın.

### Temel Kullanım

- **Super+T**: Terminal açma (foot)
- **Super+B**: Ekran klavyesi açma (onboard)
- **Super+D**: Uygulama başlatıcı (fuzzel)
- **Super+Q**: Pencereyi kapatma

## Önemli Notlar

### Yapılandırma Dosyası

Niri yapılandırma dosyası: `~/.config/niri/config.kdl`

```kdl
binds {
    Super+T { spawn "foot"; }
    Super+B { spawn "onboard"; }
}

spawn-at-startup {
    command ["foot"]
}
```

### Systemd Servis Sorunu

Display manager'dan niri seçildiğinde login ekranına dönüyorsa:

```bash
# Servis dosyasındaki yolu düzeltme
sudo sed -i 's|ExecStart=/usr/bin/niri|ExecStart=/usr/local/bin/niri|g' /usr/local/lib/systemd/user/niri.service
systemctl --user daemon-reload
```

### Sunshine Streaming

Niri session'ında Sunshine başlatma:

```bash
export WAYLAND_DISPLAY=wayland-1
export XDG_SESSION_TYPE=wayland
sunshine > /tmp/sunshine_niri.log 2>&1 &
```

## Deneyim Temelli Çözümler

Bu dokümantasyon, aşağıdaki gerçek sorunların çözümlerini içerir:

- ✅ **Çoklu niri süreçleri** - Systemd servis yapılandırma hatası
- ✅ **Super tuş algılanmıyor** - KDL config syntax ve modifier mapping
- ✅ **Sunshine siyah ekran** - Wayland environment variable'ları
- ✅ **Display manager başlatma sorunu** - Binary yolu düzeltmesi
- ✅ **Moonlight key mapping** - MacBook Cmd tuşu sorunu
- ✅ **SSH uzaktan kurulum** - Tam otomatik kurulum süreci

## Katkıda Bulunma

Bu dokümantasyonu geliştirmek için:

1. Sorunları ve çözümleri test edin
2. Eksik bilgileri ekleyin
3. Türkçe çevirileri iyileştirin
4. Yeni kullanım senaryoları ekleyin

## Destek

- **Ana Proje**: https://github.com/YaLTeR/niri
- **Matrix Kanalı**: https://matrix.to/#/#niri:matrix.org
- **Wiki**: https://github.com/YaLTeR/niri/wiki

## Lisans

Bu dokümantasyon, niri projesinin lisansı altında dağıtılır. Detaylar için ana proje deposuna bakın.

---

*Bu dokümantasyon, gerçek bir uzaktan kurulum deneyiminden elde edilen bilgilerle hazırlanmıştır ve production ortamında test edilmiştir.*
