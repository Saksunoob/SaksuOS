rm disk.img
dd if=/dev/zero of=disk.img bs=1M count=1024
mkfs.vfat -F 32 disk.img
# Copy bootloader to uefi load location
mmd -i disk.img ::/EFI
mmd -i disk.img ::/EFI/BOOT
mcopy -i disk.img bootloader/bootloader.efi ::/EFI/BOOT/BOOTX64.EFI
mcopy -i disk.img kernel/kernel ::/KERNEL

qemu-system-x86_64 -bios ovmf -drive format=raw,file=disk.img