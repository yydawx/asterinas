#!/bin/sh

# SPDX-License-Identifier: MPL-2.0
# Run all standalone benchmarks and save results.

BENCH="/benchmark/common/bench_runner.sh"

echo "=== Mounting persistent disk ==="
mkdir -p /ext2
if mount -t ext2 /dev/vda /ext2 2>/dev/null; then
    RESULT_FILE="/ext2/bench_results.txt"
    echo "Disk mounted at /ext2"
else
    RESULT_FILE="/tmp/bench_results.txt"
    echo "WARNING: No disk, results will be lost on shutdown!"
fi

echo "=== Benchmark Results ===" | tee "$RESULT_FILE"
echo "" | tee -a "$RESULT_FILE"

run() {
    local name="$1"
    echo "*** Running: $name ***" | tee -a "$RESULT_FILE"
    sh "$BENCH" "$name" asterinas 2>&1 | tee -a "$RESULT_FILE"
    echo "" | tee -a "$RESULT_FILE"
}

# Disk I/O first (memory-hungry, needs clean memory)
run fio/ext2_seq_read_bw
run fio/ext2_seq_write_bw

# CPU
run sysbench/cpu_lat
run sysbench/thread_lat

# IPC / context switch
run lmbench/process_ctx_lat
run lmbench/pipe_lat
run lmbench/process_fork_lat
run lmbench/process_exec_lat

# Memory
run lmbench/mem_read_bw
run lmbench/mem_copy_bw
run lmbench/mem_mmap_bw
run lmbench/mem_mmap_lat

# VFS
run lmbench/vfs_stat_lat
run lmbench/vfs_read_lat
run lmbench/vfs_write_lat

# Scheduler
run hackbench/group8_smp1
run schbench/smp1

echo "=== All benchmarks done ===" | tee -a "$RESULT_FILE"
echo "Results saved to $RESULT_FILE"

# Sync to ensure data is written before shutdown
if [ "$RESULT_FILE" = "/ext2/bench_results.txt" ]; then
    sync
    echo "Disk synced."
fi
