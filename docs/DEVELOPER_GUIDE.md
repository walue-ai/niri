# Niri Geliştirici Klavuzu

Bu klavuz, niri compositor'ünü geliştirmek, özelleştirmek ve katkıda bulunmak isteyenler için hazırlanmıştır.

## İçindekiler

1. [Geliştirme Ortamı Kurulumu](#geliştirme-ortamı-kurulumu)
2. [Proje Yapısı](#proje-yapısı)
3. [Derleme ve Test](#derleme-ve-test)
4. [Debug ve Profiling](#debug-ve-profiling)
5. [Katkıda Bulunma](#katkıda-bulunma)
6. [Uzaktan Geliştirme](#uzaktan-geliştirme)

## Geliştirme Ortamı Kurulumu

### Gerekli Araçlar

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add clippy rustfmt

# Geliştirme araçları
sudo apt install -y git build-essential pkg-config clang

# Niri bağımlılıkları
sudo apt-get install -y gcc clang libudev-dev libgbm-dev libxkbcommon-dev \
    libegl1-mesa-dev libwayland-dev libinput-dev libdbus-1-dev \
    libsystemd-dev libseat-dev libpipewire-0.3-dev libpango1.0-dev \
    libdisplay-info-dev

# Opsiyonel: Tracy profiler
# https://github.com/wolfpld/tracy/releases
```

### IDE Kurulumu

#### VS Code
```bash
# Rust extension'ları
code --install-extension rust-lang.rust-analyzer
code --install-extension vadimcn.vscode-lldb
```

#### Vim/Neovim
```bash
# rust.vim ve coc-rust-analyzer
```

### Proje Klonlama

```bash
git clone https://github.com/YaLTeR/niri.git
cd niri

# Fork'unuzu klonlama (katkıda bulunmak için)
git clone https://github.com/KULLANICI_ADI/niri.git
cd niri
git remote add upstream https://github.com/YaLTeR/niri.git
```

## Proje Yapısı

### Ana Dizinler

```
niri/
├── src/                    # Ana kaynak kod
│   ├── main.rs            # Giriş noktası
│   ├── niri.rs            # Core state management (6300+ satır)
│   ├── layout/            # Layout sistemi
│   ├── input/             # Girdi işleme
│   ├── animation/         # Animasyon sistemi
│   ├── render_helpers/    # Rendering yardımcıları
│   ├── protocols/         # Wayland protokolleri
│   └── handlers/          # Event handlers
├── niri-config/           # Yapılandırma sistemi
├── niri-ipc/             # IPC (Inter-Process Communication)
├── niri-visual-tests/     # Görsel testler
├── resources/             # Kurulum dosyaları
└── wiki/                  # Dokümantasyon
```

### Önemli Dosyalar

#### `src/main.rs`
- Uygulama giriş noktası
- Komut satırı argümanları işleme
- Environment setup
- Event loop başlatma

#### `src/niri.rs`
- Ana compositor durumu (400+ satır struct)
- Wayland event işleme
- Input processing
- Output management

#### `src/layout/mod.rs`
- Scrollable tiling layout algoritması
- Monitor management
- Workspace logic
- Interactive move/resize

#### `src/input/mod.rs`
- Klavye, fare, dokunmatik girdi
- Keybinding işleme
- Gesture recognition

## Derleme ve Test

### Derleme Seçenekleri

```bash
# Debug build (geliştirme için)
cargo build

# Release build (performans için)
cargo build --release

# Belirli özelliklerle derleme
cargo build --release --no-default-features --features dinit,dbus,xdp-gnome-screencast

# Profiling ile derleme
cargo build --release --features=profile-with-tracy-ondemand
```

### Test Çalıştırma

```bash
# Unit testler
cargo test

# Integration testler
cargo test --test integration

# Belirli test
cargo test test_layout

# Görsel testler
cd niri-visual-tests
cargo run
```

### Linting ve Formatting

```bash
# Clippy (linter)
cargo clippy

# Formatting
cargo fmt

# Typo kontrolü
typos

# Pre-commit hook kurulumu
pre-commit install
```

## Debug ve Profiling

### Debug Logging

```bash
# Debug seviyesinde çalıştırma
RUST_LOG=niri=debug cargo run

# Belirli modüller
RUST_LOG=niri::input=debug,niri::layout=debug cargo run

# Trace seviyesi (sadece debug build'de)
RUST_LOG=niri=trace cargo run
```

### Nested Window Mode

```bash
# Mevcut desktop session içinde test
cargo run

# Belirli boyutta pencere
cargo run -- --size 1920x1080
```

### Tracy Profiling

```bash
# Tracy ile derleme
cargo build --release --features=profile-with-tracy-ondemand

# Tracy server başlatma
./tracy

# Niri'yi profiling ile çalıştırma
./target/release/niri
```

### GDB/LLDB Debugging

```bash
# GDB ile debug
cargo build
gdb ./target/debug/niri

# LLDB ile debug (VS Code)
# launch.json yapılandırması gerekli
```

## Katkıda Bulunma

### Git Workflow

```bash
# Upstream'den güncellemeler
git fetch upstream
git checkout main
git merge upstream/main

# Feature branch oluşturma
git checkout -b feature/yeni-ozellik

# Değişiklikleri commit etme
git add .
git commit -m "feat: yeni özellik eklendi"

# Push ve PR
git push origin feature/yeni-ozellik
# GitHub'da Pull Request oluşturun
```

### Commit Mesajları

```bash
# Conventional commits kullanın
feat: yeni özellik eklendi
fix: hata düzeltildi
docs: dokümantasyon güncellendi
style: kod formatı düzeltildi
refactor: kod yeniden düzenlendi
test: test eklendi
chore: build/config değişiklikleri
```

### Code Review Süreci

1. **PR Oluşturma**
   - Açıklayıcı başlık ve açıklama
   - İlgili issue'lara referans
   - Screenshot/video (UI değişiklikleri için)

2. **CI Kontrolleri**
   - Clippy warnings
   - Test geçişi
   - Formatting kontrolü

3. **Review Süreci**
   - Maintainer review
   - Değişiklik talepleri
   - Approval ve merge

### Test Yazma

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_calculation() {
        let layout = Layout::new();
        // Test implementation
        assert_eq!(layout.width(), 100);
    }
}
```

## Uzaktan Geliştirme

### SSH ile Geliştirme

```bash
# SSH key kurulumu
ssh-keygen -t ed25519 -f ~/.ssh/niri_dev_key
ssh-copy-id -i ~/.ssh/niri_dev_key.pub user@remote_ip

# Uzaktan derleme
ssh -i ~/.ssh/niri_dev_key user@remote_ip "
    cd niri &&
    git pull &&
    cargo build --release
"
```

### Remote Testing

```bash
# Uzaktan test çalıştırma
ssh -i ~/.ssh/niri_dev_key user@remote_ip "
    cd niri &&
    RUST_LOG=debug cargo test
"

# Binary transfer
scp -i ~/.ssh/niri_dev_key target/release/niri user@remote_ip:/tmp/
ssh -i ~/.ssh/niri_dev_key user@remote_ip "
    sudo cp /tmp/niri /usr/local/bin/ &&
    systemctl --user restart niri
"
```

### Development Environment Sync

```bash
# Rsync ile kod senkronizasyonu
rsync -avz --exclude target/ --exclude .git/ \
    -e "ssh -i ~/.ssh/niri_dev_key" \
    ./ user@remote_ip:~/niri-dev/

# Watch mode ile otomatik sync
watchman-make -p '**/*.rs' -t sync-remote
```

## Özel Geliştirme Konuları

### Layout Algoritması Geliştirme

```rust
// src/layout/mod.rs içinde
impl Layout {
    pub fn calculate_positions(&mut self) {
        // Scrollable tiling algoritması
        for (i, column) in self.columns.iter_mut().enumerate() {
            column.x = i as f64 * self.column_width;
            // ...
        }
    }
}
```

### Input Handler Ekleme

```rust
// src/input/mod.rs içinde
impl InputHandler {
    pub fn handle_custom_gesture(&mut self, gesture: CustomGesture) {
        match gesture {
            CustomGesture::ThreeFingerSwipe => {
                // Custom gesture implementation
            }
        }
    }
}
```

### Animation Sistemi

```rust
// src/animation/mod.rs içinde
pub struct CustomAnimation {
    duration: Duration,
    easing: EasingFunction,
}

impl Animation for CustomAnimation {
    fn update(&mut self, dt: Duration) -> bool {
        // Animation logic
        true // Continue animation
    }
}
```

### Wayland Protocol Ekleme

```rust
// src/protocols/mod.rs içinde
pub mod custom_protocol {
    use wayland_server::protocol::*;
    
    // Protocol implementation
}
```

## Performance Optimization

### Profiling Workflow

1. **Tracy ile profiling**
   ```bash
   cargo build --release --features=profile-with-tracy-ondemand
   ./target/release/niri &
   tracy
   ```

2. **Hotspot analizi**
   - Function call frequency
   - Memory allocation patterns
   - GPU usage

3. **Optimization**
   - Algorithm improvements
   - Memory pool usage
   - Batch operations

### Memory Management

```rust
// Object pooling örneği
pub struct ObjectPool<T> {
    objects: Vec<T>,
    available: Vec<usize>,
}

impl<T> ObjectPool<T> {
    pub fn get(&mut self) -> Option<&mut T> {
        self.available.pop()
            .map(|idx| &mut self.objects[idx])
    }
}
```

Bu geliştirici klavuzu, niri compositor'üne katkıda bulunmak ve özelleştirmek için gereken tüm bilgileri içerir. Sorularınız için niri topluluğuna katılabilirsiniz.
