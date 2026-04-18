#!/bin/sh

API_BASE_URL=${API_BASE_URL:-http://localhost:3001/api/v1}
CURL_BIN=${CURL_BIN:-curl}
JQ_BIN=${JQ_BIN:-jq}

SUITE_NAME=""
CASE_TOTAL=0
CASE_FAILED=0
TMP_DIR=""
LAST_RESPONSE_STATUS=""
LAST_RESPONSE_BODY=""
LAST_RESPONSE_CONTENT_TYPE=""
CLEANUP_PROJECT_IDS=""

setup_suite() {
  SUITE_NAME=$1
  CASE_TOTAL=0
  CASE_FAILED=0
  TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/plan-api-tests.XXXXXX") || exit 1
  LAST_RESPONSE_STATUS=""
  LAST_RESPONSE_BODY=""
  LAST_RESPONSE_CONTENT_TYPE=""
  CLEANUP_PROJECT_IDS=""
  require_command "$CURL_BIN"
  require_command "$JQ_BIN"
  trap cleanup_suite EXIT HUP INT TERM
  printf '\n== %s ==\n' "$SUITE_NAME"
}

cleanup_suite() {
  cleanup_projects
  if [ -n "${TMP_DIR}" ] && [ -d "${TMP_DIR}" ]; then
    rm -rf "${TMP_DIR}"
  fi
}

cleanup_projects() {
  if [ -z "${CLEANUP_PROJECT_IDS}" ]; then
    return 0
  fi

  printf '%s\n' "${CLEANUP_PROJECT_IDS}" | while IFS= read -r project_id; do
    if [ -z "${project_id}" ]; then
      continue
    fi

    "${CURL_BIN}" -sS \
      -X DELETE \
      -H 'Accept: application/json' \
      -o /dev/null \
      "${API_BASE_URL}/projects/${project_id}" >/dev/null 2>&1 || true
  done
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || {
    printf 'Missing required command: %s\n' "$1" >&2
    exit 1
  }
}

run_case() {
  case_name=$1
  case_func=$2

  CASE_TOTAL=$((CASE_TOTAL + 1))
  printf '[CASE] %s ... ' "$case_name"

  if "${case_func}"; then
    printf 'PASS\n'
    return 0
  fi

  CASE_FAILED=$((CASE_FAILED + 1))
  printf 'FAIL\n'
  return 1
}

finish_suite() {
  passed=$((CASE_TOTAL - CASE_FAILED))
  printf 'Summary: %s passed, %s failed, %s total\n' \
    "$passed" "$CASE_FAILED" "$CASE_TOTAL"

  if [ "${CASE_FAILED}" -ne 0 ]; then
    return 1
  fi

  return 0
}

new_tmp_file() {
  mktemp "${TMP_DIR}/response.XXXXXX"
}

fail() {
  message=$1
  printf '\n  %s\n' "$message" >&2

  if [ -n "${LAST_RESPONSE_STATUS}" ]; then
    printf '  HTTP status: %s\n' "${LAST_RESPONSE_STATUS}" >&2
  fi

  if [ -n "${LAST_RESPONSE_CONTENT_TYPE}" ]; then
    printf '  Content-Type: %s\n' "${LAST_RESPONSE_CONTENT_TYPE}" >&2
  fi

  if [ -n "${LAST_RESPONSE_BODY}" ] && [ -f "${LAST_RESPONSE_BODY}" ]; then
    printf '  Response body:\n' >&2
    sed 's/^/    /' "${LAST_RESPONSE_BODY}" >&2
  fi

  return 1
}

api_request() {
  method=$1
  path=$2
  body=${3-}
  content_type=${4-application/json}
  output_file=$(new_tmp_file) || return 1

  if [ -n "${body}" ]; then
    curl_result=$(
      "${CURL_BIN}" -sS \
        -X "${method}" \
        -H 'Accept: application/json' \
        -H "Content-Type: ${content_type}" \
        --data "${body}" \
        -o "${output_file}" \
        -w '%{http_code}|%{content_type}' \
        "${API_BASE_URL}${path}"
    ) || {
      LAST_RESPONSE_BODY=${output_file}
      fail "curl request failed for ${method} ${path}"
      return 1
    }
  else
    curl_result=$(
      "${CURL_BIN}" -sS \
        -X "${method}" \
        -H 'Accept: application/json' \
        -o "${output_file}" \
        -w '%{http_code}|%{content_type}' \
        "${API_BASE_URL}${path}"
    ) || {
      LAST_RESPONSE_BODY=${output_file}
      fail "curl request failed for ${method} ${path}"
      return 1
    }
  fi

  LAST_RESPONSE_STATUS=$(printf '%s' "${curl_result}" | cut -d '|' -f 1)
  LAST_RESPONSE_CONTENT_TYPE=$(printf '%s' "${curl_result}" | cut -d '|' -f 2-)
  LAST_RESPONSE_BODY=${output_file}
  return 0
}

assert_status() {
  expected=$1
  if [ "${LAST_RESPONSE_STATUS}" != "${expected}" ]; then
    fail "Expected HTTP ${expected}, got ${LAST_RESPONSE_STATUS}"
    return 1
  fi
  return 0
}

assert_status_in() {
  allowed=$1

  for code in ${allowed}; do
    if [ "${LAST_RESPONSE_STATUS}" = "${code}" ]; then
      return 0
    fi
  done

  fail "Expected HTTP status in [${allowed}], got ${LAST_RESPONSE_STATUS}"
  return 1
}

assert_json_expr() {
  expr=$1
  message=$2

  if ! "${JQ_BIN}" -e "${expr}" "${LAST_RESPONSE_BODY}" >/dev/null 2>&1; then
    fail "${message}"
    return 1
  fi

  return 0
}

json_query() {
  expr=$1
  "${JQ_BIN}" -er "${expr}" "${LAST_RESPONSE_BODY}"
}

assert_success_envelope() {
  assert_json_expr '.data != null' "Response is missing top-level data" || return 1
  assert_json_expr '.meta.requestId | type == "string" and length > 0' \
    "Response meta.requestId is missing" || return 1
  return 0
}

assert_list_envelope() {
  assert_json_expr '.data | type == "array"' "Response data is not an array" || return 1
  assert_json_expr '.meta.requestId | type == "string" and length > 0' \
    "List response meta.requestId is missing" || return 1
  return 0
}

assert_paginated_list_envelope() {
  assert_list_envelope || return 1
  assert_json_expr '.meta.page | type == "number"' "List response meta.page is missing" || return 1
  assert_json_expr '.meta.pageSize | type == "number"' "List response meta.pageSize is missing" || return 1
  assert_json_expr '.meta.total | type == "number"' "List response meta.total is missing" || return 1
  return 0
}

assert_error_envelope() {
  assert_json_expr '.error.code | type == "string" and length > 0' \
    "Error response code is missing" || return 1
  assert_json_expr '.error.message | type == "string" and length > 0' \
    "Error response message is missing" || return 1
  assert_json_expr '.error.requestId | type == "string" and length > 0' \
    "Error response requestId is missing" || return 1
  return 0
}

assert_client_error() {
  assert_status_in '400 404 409 422' || return 1
  assert_error_envelope || return 1
  return 0
}

assert_empty_body() {
  if [ -s "${LAST_RESPONSE_BODY}" ]; then
    fail "Expected empty response body"
    return 1
  fi
  return 0
}

register_project_cleanup() {
  project_id=$1
  CLEANUP_PROJECT_IDS="${CLEANUP_PROJECT_IDS}
${project_id}"
}

random_suffix() {
  temp_name=$(mktemp "${TMP_DIR}/id.XXXXXX") || return 1
  suffix=$(basename "${temp_name}")
  rm -f "${temp_name}"
  printf '%s' "${suffix}"
}

create_project() {
  suffix=$(random_suffix)
  body=$(cat <<EOF
{"name":"Plan Test ${suffix}","slug":"plan-test-${suffix}","description":"Contract test project","color":"#2563eb"}
EOF
)

  api_request POST '/projects' "${body}" || return 1
  assert_status '201' || return 1
  assert_success_envelope || return 1
  project_id=$(json_query '.data.id') || return 1
  register_project_cleanup "${project_id}"
  printf '%s\n' "${project_id}"
}

list_statuses_for_project() {
  project_id=$1
  api_request GET "/projects/${project_id}/statuses" || return 1
  assert_status '200' || return 1
  assert_list_envelope || return 1
  return 0
}

get_first_status_id() {
  project_id=$1
  list_statuses_for_project "${project_id}" || return 1
  json_query '.data[0].id'
}

create_status() {
  project_id=$1
  suffix=$(random_suffix)
  body=$(cat <<EOF
{"name":"Review ${suffix}","color":"#f59e0b"}
EOF
)

  api_request POST "/projects/${project_id}/statuses" "${body}" || return 1
  assert_status '201' || return 1
  assert_success_envelope || return 1
  json_query '.data.id'
}

create_task() {
  project_id=$1
  status_id=$2
  suffix=$(random_suffix)
  body=$(cat <<EOF
{"statusId":"${status_id}","title":"Task ${suffix}","description":"Contract test task","priority":"medium","startDate":"2026-04-18","dueDate":"2026-04-19","tagIds":[]}
EOF
)

  api_request POST "/projects/${project_id}/tasks" "${body}" || return 1
  assert_status '201' || return 1
  assert_success_envelope || return 1
  json_query '.data.id'
}

create_tag() {
  project_id=$1
  suffix=$(random_suffix)
  body=$(cat <<EOF
{"name":"Backend ${suffix}","color":"#10b981"}
EOF
)

  api_request POST "/projects/${project_id}/tags" "${body}" || return 1
  assert_status '201' || return 1
  assert_success_envelope || return 1
  json_query '.data.id'
}

create_comment() {
  task_id=$1
  suffix=$(random_suffix)
  body=$(cat <<EOF
{"authorName":"tester","content":"Comment ${suffix}"}
EOF
)

  api_request POST "/tasks/${task_id}/comments" "${body}" || return 1
  assert_status '201' || return 1
  assert_success_envelope || return 1
  json_query '.data.id'
}
