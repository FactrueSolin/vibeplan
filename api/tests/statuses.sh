#!/bin/sh

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "${SCRIPT_DIR}/lib/test_lib.sh"

PROJECT_ID=''
STATUS_ID=''

test_list_statuses() {
  PROJECT_ID=$(create_project) || return 1
  api_request GET "/projects/${PROJECT_ID}/statuses" || return 1
  assert_status '200' || return 1
  assert_list_envelope || return 1
  assert_json_expr '.data | length > 0' \
    "New project should be initialized with default statuses" || return 1
  return 0
}

test_create_status() {
  suffix=$(random_suffix)
  body=$(cat <<EOF
{"name":"QA ${suffix}","color":"#ef4444"}
EOF
)
  api_request POST "/projects/${PROJECT_ID}/statuses" "${body}" || return 1
  assert_status '201' || return 1
  assert_success_envelope || return 1
  STATUS_ID=$(json_query '.data.id') || return 1
  assert_json_expr ".data.projectId == \"${PROJECT_ID}\"" \
    "Created status projectId did not match fixture project" || return 1
  return 0
}

test_patch_status() {
  body='{"name":"QA Updated","color":"#f97316"}'
  api_request PATCH "/statuses/${STATUS_ID}" "${body}" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1
  assert_json_expr '.data.name == "QA Updated"' \
    "Updated status name was not returned" || return 1
  return 0
}

test_reorder_statuses() {
  extra_status_id=$(create_status "${PROJECT_ID}") || return 1
  list_statuses_for_project "${PROJECT_ID}" || return 1
  ordered_status_ids=$("${JQ_BIN}" -r --arg extra "${extra_status_id}" '
    (.data | map(.id) | map(select(. != $extra))) as $rest
    | [$extra] + $rest
    | map(@json)
    | join(",")
  ' "${LAST_RESPONSE_BODY}") || return 1

  body=$(cat <<EOF
{"orderedStatusIds":[${ordered_status_ids}]}
EOF
)

  api_request POST "/projects/${PROJECT_ID}/statuses/reorder" "${body}" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1

  list_statuses_for_project "${PROJECT_ID}" || return 1
  assert_json_expr ".data[0].id == \"${extra_status_id}\"" \
    "Reordered status was not moved to the first position" || return 1
  return 0
}

test_delete_status_with_active_task_conflict() {
  blocking_status_id=$(create_status "${PROJECT_ID}") || return 1
  task_id=$(create_task "${PROJECT_ID}" "${blocking_status_id}") || return 1
  if [ -z "${task_id}" ]; then
    fail "Failed to create task fixture for status conflict test"
    return 1
  fi

  api_request DELETE "/statuses/${blocking_status_id}" || return 1
  assert_status '409' || return 1
  assert_error_envelope || return 1
  return 0
}

test_create_status_empty_name() {
  body='{"name":"","color":"#ef4444"}'
  api_request POST "/projects/${PROJECT_ID}/statuses" "${body}" || return 1
  assert_status '422' || return 1
  assert_error_envelope || return 1
  return 0
}

test_patch_status_injection_id() {
  body='{"name":"Injected","color":"#ef4444"}'
  api_request PATCH "/statuses/1%27%20OR%20%271%27%3D%271" "${body}" || return 1
  assert_client_error || return 1
  return 0
}

setup_suite 'Statuses API'
run_case 'GET /projects/{projectId}/statuses returns initialized status list' test_list_statuses
run_case 'POST /projects/{projectId}/statuses creates a status' test_create_status
run_case 'PATCH /statuses/{statusId} updates the status' test_patch_status
run_case 'POST /projects/{projectId}/statuses/reorder updates status order' test_reorder_statuses
run_case 'DELETE /statuses/{statusId} returns 409 when status still has active tasks' test_delete_status_with_active_task_conflict
run_case 'POST /projects/{projectId}/statuses rejects empty status name' test_create_status_empty_name
run_case 'PATCH /statuses/{statusId} rejects injection style path input' test_patch_status_injection_id
finish_suite
