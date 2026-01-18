#!/bin/bash
set -euo pipefail

# LoongArch64 QEMU 运行脚本（对齐评测启动方式）
KERNEL=$1
MODE=${2:-run}

# 参数定义（对齐评测指令）
mem="4G"
smp="1"
root_dir="$(cd "$(dirname "$0")/.." && pwd)"
fs_xz="${root_dir}/sdcard-la.img.xz"
fs="${root_dir}/sdcard-la.img"
disk="${root_dir}/disk-la.img"

# 评测盘镜像（EXT4，无分区表）
if [ ! -f "$fs" ]; then
    if [ -f "$fs_xz" ]; then
        if ! command -v xz >/dev/null 2>&1; then
            echo "Error: xz not found; cannot decompress ${fs_xz}" >&2
            exit 1
        fi
        echo "Decompressing ${fs_xz}..."
        tmp_fs="${fs}.tmp"
        xz -dc "$fs_xz" > "$tmp_fs"
        mv "$tmp_fs" "$fs"
    else
        echo "Error: ${fs_xz} not found!" >&2
        exit 1
    fi
fi

QEMU_ARGS=(
    -machine virt
    -kernel "$KERNEL"
    -m "$mem"
    -nographic
    -smp "$smp"
    -no-reboot
)

# Virtio Block 设备（评测盘）
QEMU_ARGS+=(-drive file="$fs",if=none,format=raw,id=x0)
QEMU_ARGS+=(-device virtio-blk-pci,drive=x0)

# Virtio Network 设备
QEMU_ARGS+=(-device virtio-net-pci,netdev=net0)
QEMU_ARGS+=(-netdev user,id=net0,hostfwd=tcp::5555-:5555,hostfwd=udp::5555-:5555)

# RTC 设备 (基于 UTC 时间)
QEMU_ARGS+=(-rtc base=utc)

# 附加磁盘 (disk-la.img)
if [ -f "$disk" ]; then
    QEMU_ARGS+=(-drive file="$disk",if=none,format=raw,id=x1)
    QEMU_ARGS+=(-device virtio-blk-pci,drive=x1)
fi

case $MODE in
    run)
        qemu-system-loongarch64 "${QEMU_ARGS[@]}"
        ;;
    gdb)
        QEMU_ARGS+=(-s -S)
        qemu-system-loongarch64 "${QEMU_ARGS[@]}"
        ;;
    *)
        echo "Usage: $0 <kernel> [run|gdb]"
        exit 1
        ;;
esac
