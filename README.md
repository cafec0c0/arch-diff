# arch-diff

Show new package versions for arch linux in a parsable format.

## Output
```
rogue@militech ~/r/arch-diff (main)>  ~/.cargo/bin/arch-diff 
:: Fetching package database for core
:: Fetching package database for extra
:: Comparing package versions
:: 120 packages to upgrade
core/audit                  4.0.2-3             -> 4.0.3-1
core/cryptsetup             2.7.5-1             -> 2.7.5-2
core/device-mapper          2.03.29-1           -> 2.03.30-1
core/libxcrypt              4.4.37-1            -> 4.4.38-1
core/linux                  6.12.8.arch1-1      -> 6.12.9.arch1-1
core/linux-firmware         20241210.b00a7f7e-1 -> 20250109.7673dffd-1
core/linux-firmware-whence  20241210.b00a7f7e-1 -> 20250109.7673dffd-1
core/linux-headers          6.12.8.arch1-1      -> 6.12.9.arch1-1
core/ppp                    2.5.1-1             -> 2.5.2-1
core/sqlite                 3.47.2-1            -> 3.48.0-2
core/systemd                257.2-1             -> 257.2-2
core/systemd-libs           257.2-1             -> 257.2-2
core/systemd-sysvcompat     257.2-1             -> 257.2-2
core/util-linux             2.40.3-1            -> 2.40.4-1
core/util-linux-libs        2.40.3-1            -> 2.40.4-1
extra/at-spi2-core          2.54.0-2            -> 2.54.1-1
extra/attica                6.9.0-1             -> 6.10.0-1
extra/baloo                 6.9.0-1             -> 6.10.0-1
extra/bcc                   0.32.0-2            -> 0.32.0-3
extra/bcc-tools             0.32.0-2            -> 0.32.0-3
extra/blas                  3.12.0-5            -> 3.12.1-2
...
```

## Building
```
cargo build --release
```

## Installing
```
cargo install --path .
```

## Usage
```
arch-diff
```
