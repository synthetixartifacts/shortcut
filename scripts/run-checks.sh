#!/usr/bin/env bash

set -euo pipefail

profile="${CHECK_PROFILE:-full}"
profile="$(printf '%s' "$profile" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')"

case "$profile" in
	full)
		tasks=(svelte-check cargo-check clippy)
		;;
	fast)
		tasks=(svelte-check cargo-check)
		;;
	frontend)
		tasks=(svelte-check)
		;;
	rust)
		tasks=(cargo-check)
		;;
	clippy)
		tasks=(clippy)
		;;
	skip)
		echo "=== Validation skipped (CHECK_PROFILE=skip) ==="
		exit 0
		;;
	*)
		echo "Unsupported CHECK_PROFILE: $profile" >&2
		echo "Supported values: full, fast, frontend, rust, clippy, skip" >&2
		exit 1
		;;
esac

ensure_node_modules() {
	if [ ! -f node_modules/.package-lock.json ]; then
		npm ci --no-audit --no-fund --prefer-offline || npm install --no-audit --no-fund
	fi
}

run_task() {
	local task="$1"
	local step="$2"
	local total="$3"

	case "$task" in
		svelte-check)
			echo "=== [$step/$total] npm run check (svelte-check) ==="
			ensure_node_modules
			npm run check
			;;
		cargo-check)
			echo "=== [$step/$total] cargo check (Rust) ==="
			(
				cd src-tauri
				cargo check
			)
			;;
		clippy)
			echo "=== [$step/$total] cargo clippy -- -D warnings (Rust lints) ==="
			(
				cd src-tauri
				cargo clippy --all-targets -- -D warnings
			)
			;;
	esac
}

total_steps="${#tasks[@]}"

echo "=== Validation profile: $profile ==="

for index in "${!tasks[@]}"; do
	step=$((index + 1))
	run_task "${tasks[$index]}" "$step" "$total_steps"

	if [ "$step" -lt "$total_steps" ]; then
		echo
	fi
done

echo
echo "=== All requested checks passed ==="
