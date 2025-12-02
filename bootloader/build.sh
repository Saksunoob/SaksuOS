cargo build -Zbuild-std --target x86_64-unknown-uefi || exit 1
cp target/x86_64-unknown-uefi/debug/bootloader.efi .