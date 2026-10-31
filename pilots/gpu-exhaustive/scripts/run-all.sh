#!/bin/sh
# Run every experiment and keep one log per step.
# Usage: scripts/run-all.sh <bin-dir> <out-dir>
# <bin-dir> holds gpu-exhaustive-wgsl and gpu-exhaustive-spirv from scripts/build.sh.
set -u
BIN=${1:?bin dir}
OUT=${2:?out dir}
mkdir -p "$OUT"
W="$BIN/gpu-exhaustive-wgsl"
S="$BIN/gpu-exhaustive-spirv"

step() { # step <name> <cmd...>
  name=$1; shift
  echo "== $name: $*"
  { echo "# cmd: $*"; echo "# load before: $(cat /proc/loadavg)"; "$@" 2>&1; echo "# exit: $?"; echo "# load after: $(cat /proc/loadavg)"; } > "$OUT/$name.log"
  grep -E '^(RESULT (check|diff_summary|mutants_summary|coverage_error|spec_validate_[a-z0-9]+ )|# exit)' "$OUT/$name.log" | cut -c1-160
}

# environment
step info-gpu-wgsl      "$W" info --backend gpu
step info-gpu-spirv     "$S" info --backend gpu
step info-llvmpipe-wgsl "$W" info --backend llvmpipe
step info-cpu           "$W" info --backend cpu
for be in gpu llvmpipe cpu; do
  step "fp-probe-$be-wgsl" "$W" fp-probe --backend $be
  step "fp-probe-$be-spirv" "$S" fp-probe --backend $be
done
step spec-validate "$W" spec-validate

# Experiment 1 + 3, kernel A: on-device check (3 repeats each, in one process)
step a-check-rayon        "$W" check --backend rayon    --kernel bf16 --repeat 3
step a-check-gpu-wgsl     "$W" check --backend gpu      --kernel bf16 --repeat 3
step a-check-gpu-spirv    "$S" check --backend gpu      --kernel bf16 --repeat 3
step a-check-llvmpipe-wgsl  "$W" check --backend llvmpipe --kernel bf16 --repeat 3
step a-check-llvmpipe-spirv "$S" check --backend llvmpipe --kernel bf16 --repeat 3
step a-check-cpu          "$W" check --backend cpu      --kernel bf16 --repeat 3
step a-native-check-cpu   "$W" check --backend cpu      --kernel bf16-native --repeat 2
step a-native-check-gpu   "$W" check --backend gpu      --kernel bf16-native
# kernel A: readback mode, exact comparison with the plain-Rust spec on the host
step a-diff-rayon-spec    "$W" diff --a rayon    --b spec --kernel bf16
step a-diff-gpu-spec      "$W" diff --a gpu      --b spec --kernel bf16
step a-diff-llvmpipe-spec "$W" diff --a llvmpipe --b spec --kernel bf16
step a-diff-cpu-spec      "$W" diff --a cpu      --b spec --kernel bf16
step a-native-diff-cpu-spec "$W" diff --a cpu    --b spec --kernel bf16-native

# Experiment 1 + 3, kernel B (original = round(); mutant 26 = shifter trick as first written)
step b-check-rayon        "$W" check --backend rayon    --kernel expf --repeat 3
step b-check-gpu-wgsl     "$W" check --backend gpu      --kernel expf --repeat 3
step b-check-gpu-spirv    "$S" check --backend gpu      --kernel expf --repeat 3
step b-check-llvmpipe-wgsl  "$W" check --backend llvmpipe --kernel expf --repeat 3
step b-check-llvmpipe-spirv "$S" check --backend llvmpipe --kernel expf --repeat 3
step b-check-cpu          "$W" check --backend cpu      --kernel expf --repeat 3
for be in rayon gpu llvmpipe cpu; do
  step "b-shifter-check-$be" "$W" check --backend $be --kernel expf --mutant 26
done
step b-diff-rayon-spec    "$W" diff --a rayon    --b spec  --kernel expf
step b-diff-gpu-spec      "$W" diff --a gpu      --b spec  --kernel expf
step b-diff-gpu-rayon     "$W" diff --a gpu      --b rayon --kernel expf
step b-diff-llvmpipe-rayon "$W" diff --a llvmpipe --b rayon --kernel expf
step b-diff-cpu-rayon     "$W" diff --a cpu      --b rayon --kernel expf
step b-diff-cpu-spec      "$W" diff --a cpu      --b spec  --kernel expf
step b-diff-gpu-cpu       "$W" diff --a gpu      --b cpu   --kernel expf

# Experiment 2: mutant schemata, exhaustive on every path
step mut-gpu-wgsl       "$W" mutants --backend gpu
step mut-gpu-spirv      "$S" mutants --backend gpu
step mut-llvmpipe-wgsl  "$W" mutants --backend llvmpipe
step mut-cpu            "$W" mutants --backend cpu
step mut-rayon          "$W" mutants --backend rayon
# determinism across processes: rerun the GPU mutant sweep
step mut-gpu-wgsl-rerun "$W" mutants --backend gpu
# seeded 2^24 sample (stride 256) for comparison of cost and verdicts
step mut-gpu-sample     "$W" mutants --backend gpu   --stride 256 --seed 7
step mut-cpu-sample     "$W" mutants --backend cpu   --stride 256 --seed 7
step mut-rayon-sample   "$W" mutants --backend rayon --stride 256 --seed 7
echo done
