#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-https://server.fire.ly}"
RUN_BATCH="${RUN_BATCH:-0}"
DEFAULT_SYNTHEA_FILE="synthea/Alfred550_Boyle917_cd0162c8-4d0d-bcd2-3e73-8d3d5ed99388.json"

if [[ -z "${SYNTHEA_FILE:-}" ]]; then
  if [[ -f "$DEFAULT_SYNTHEA_FILE" ]]; then
    SYNTHEA_FILE="$DEFAULT_SYNTHEA_FILE"
  else
    SYNTHEA_FILE=""
  fi
fi

export CARGO_TARGET_DIR="./target"

timestamp="$(date +"%Y-%m-%d_%H-%M-%S")"
out_dir="out/${timestamp}"
mkdir -p "$out_dir"

summary="$out_dir/summary.txt"

overall=0

set_var() {
  local var="$1"
  local val="$2"
  printf -v "$var" '%s' "$val"
}

get_var() {
  local var="$1"
  printf '%s' "${!var}"
}

record_overall() {
  local rc="$1"
  if [[ "$rc" =~ ^[0-9]+$ ]] && [[ $rc -ne 0 ]] && [[ $overall -eq 0 ]]; then
    overall=$rc
  fi
}

is_expected_failure() {
  case "$1" in
    validate_patient_fail|validate_synthea_firely|validate_synthea_grep)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

run_cmd() {
  local name="$1"
  shift
  local log="$out_dir/${name}.txt"
  set_var "log_${name}" "$log"

  {
    echo "==> $name"
    echo "Command: $*"
    echo "----"
  } > "$log"

  set +e
  "$@" >> "$log" 2>&1
  local rc=$?
  set -e

  set_var "code_${name}" "$rc"
  if [[ $rc -ne 0 ]]; then
    echo "Exit: $rc" >> "$log"
  fi

  if ! is_expected_failure "$name"; then
    record_overall "$rc"
  fi
  return 0
}

skip_cmd() {
  local name="$1"
  local reason="$2"
  local log="$out_dir/${name}.txt"
  set_var "log_${name}" "$log"
  set_var "code_${name}" "skipped"
  {
    echo "==> $name"
    echo "SKIPPED: $reason"
  } > "$log"
}

run_cmd fmt cargo fmt --check
run_cmd clippy cargo clippy -- -D warnings
run_cmd test cargo test
run_cmd validate_patient_pass cargo run --quiet -- validate examples/patient.json --base-url "$BASE_URL"
run_cmd validate_patient_fail cargo run --quiet -- validate examples/patient-bad.json --base-url "$BASE_URL"

if [[ -n "$SYNTHEA_FILE" && -f "$SYNTHEA_FILE" ]]; then
  run_cmd validate_synthea_firely cargo run --quiet -- validate "$SYNTHEA_FILE" --base-url "$BASE_URL"
  run_cmd validate_synthea_grep bash -c "set -o pipefail; cargo run --quiet -- validate '$SYNTHEA_FILE' --base-url '$BASE_URL' | rg -n 'Themes:|Profile resolution|Validate:'; rc=\$?; echo \"PIPELINE_EXIT=\$rc\"; exit \$rc"
else
  skip_cmd validate_synthea_firely "SYNTHEA_FILE not set or file missing"
  skip_cmd validate_synthea_grep "SYNTHEA_FILE not set or file missing"
fi

if [[ "$RUN_BATCH" == "1" ]]; then
  if compgen -G "synthea/*.json" > /dev/null; then
    run_cmd synthea_batch bash -c "set -o pipefail; for f in synthea/*.json; do cargo run --quiet -- validate \"\$f\" --base-url '$BASE_URL'; done"
  else
    skip_cmd synthea_batch "No synthea/*.json files found"
  fi
else
  skip_cmd synthea_batch "RUN_BATCH is not set to 1"
fi

{
  echo "ClinLogix reproducible checks"
  echo "Timestamp: $timestamp"
  echo "Commit: $(git rev-parse HEAD 2>/dev/null || echo unknown)"
  echo "rustc: $(rustc --version 2>/dev/null || echo unknown)"
  echo "cargo: $(cargo --version 2>/dev/null || echo unknown)"
  echo "BASE_URL: $BASE_URL"
  if [[ -n "$SYNTHEA_FILE" ]]; then
    echo "SYNTHEA_FILE: $SYNTHEA_FILE"
  else
    echo "SYNTHEA_FILE: skipped"
  fi
  echo
  echo "Exit codes:"
  for name in fmt clippy test validate_patient_pass validate_patient_fail validate_synthea_firely validate_synthea_grep synthea_batch; do
    echo "$name: $(get_var code_${name})"
  done
  echo
  echo "Logs:"
  for name in fmt clippy test validate_patient_pass validate_patient_fail validate_synthea_firely validate_synthea_grep synthea_batch; do
    echo "$name: $(get_var log_${name})"
  done
} > "$summary"

echo "Output saved to $out_dir"
exit "$overall"
