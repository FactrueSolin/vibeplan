#!/bin/sh

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
CURL_BIN=${CURL_BIN:-curl}

total=0
failed=0

printf 'Plan API test runner\n'
printf 'API_BASE_URL=%s\n' "${API_BASE_URL:-http://localhost:3001/api/v1}"

if ! command -v "${CURL_BIN}" >/dev/null 2>&1; then
  printf 'Missing required command: %s\n' "${CURL_BIN}" >&2
  exit 1
fi

preflight_status=$(
  "${CURL_BIN}" -sS \
    -o /dev/null \
    -w '%{http_code}' \
    "${API_BASE_URL:-http://localhost:3001/api/v1}/openapi.json"
) || preflight_status='curl_error'

if [ "${preflight_status}" != '200' ]; then
  printf 'Preflight failed: GET %s/openapi.json returned %s\n' \
    "${API_BASE_URL:-http://localhost:3001/api/v1}" \
    "${preflight_status}" >&2
  printf 'Start the backend or override API_BASE_URL before running tests.\n' >&2
  exit 1
fi

for script_name in system.sh projects.sh board.sh statuses.sh tasks.sh comments.sh tags.sh; do
  total=$((total + 1))
  printf '\n[RUN] %s\n' "${script_name}"

  if sh "${SCRIPT_DIR}/${script_name}"; then
    printf '[OK] %s\n' "${script_name}"
  else
    printf '[FAIL] %s\n' "${script_name}"
    failed=$((failed + 1))
  fi
done

if [ "${failed}" -ne 0 ]; then
  printf '\nResult: %s suite(s) failed\n' "${failed}"
  exit 1
fi

printf '\nResult: all %s suite(s) passed\n' "${total}"
