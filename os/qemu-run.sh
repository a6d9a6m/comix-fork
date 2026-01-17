#!/bin/bash
set -euo pipefail

ELF_FILE="$1"

# 参数定义
os_file="$ELF_FILE"
mem="4G"
smp="${SMP:-1}"  # 从环境变量读取，默认为 1
root_dir="$(cd "$(dirname "$0")/.." && pwd)"
fs_xz="${root_dir}/testsuits-for-oskernel/sdcard-rv.img.xz"
fs="${root_dir}/testsuits-for-oskernel/sdcard-rv.img"
disk="${root_dir}/disk.img"

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

echo "Using testsuite image: ${fs}"

# 3. 运行 QEMU
QEMU_ARGS="-machine virt \
            -kernel $os_file \
            -m $mem \
            -nographic \
            -smp $smp \
            -bios default \
            -no-reboot"

# 串口设备（nographic 下默认使用 stdio）

# RTC 设备 (基于 UTC 时间)
QEMU_ARGS="$QEMU_ARGS -rtc base=utc"

# Virtio Block 设备
QEMU_ARGS="$QEMU_ARGS -drive file=$fs,if=none,format=raw,id=x0"
QEMU_ARGS="$QEMU_ARGS -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0"

# Virtio Network 设备
QEMU_ARGS="$QEMU_ARGS -device virtio-net-device,netdev=net"
QEMU_ARGS="$QEMU_ARGS -netdev user,id=net"

# 可选附加磁盘（与评测一致：存在则挂载）
if [ -f "$disk" ]; then
    QEMU_ARGS="$QEMU_ARGS -drive file=$disk,if=none,format=raw,id=x1"
    QEMU_ARGS="$QEMU_ARGS -device virtio-blk-device,drive=x1,bus=virtio-mmio-bus.1"
fi

# GDB 调试模式
if [ "$2" == "gdb" ]; then
    echo "Starting QEMU in GDB debug mode on port 1234."
    QEMU_ARGS="$QEMU_ARGS -S -gdb tcp::1234"
else
    echo "Starting QEMU in normal run mode."
fi

qemu-system-riscv64 $QEMU_ARGS
