#!/bin/sh

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "${SCRIPT_DIR}/lib/test_lib.sh"

PROJECT_ID=''
STATUS_ID=''
TASK_ID=''

test_get_board_snapshot() {
  PROJECT_ID=$(create_project) || return 1
  STATUS_ID=$(get_first_status_id "${PROJECT_ID}") || return 1
  TASK_ID=$(create_task "${PROJECT_ID}" "${STATUS_ID}") || return 1

  api_request GET "/projects/${PROJECT_ID}/board?includeArchived=false" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1
  assert_json_expr ".data.project.id == \"${PROJECT_ID}\"" \
    "Board snapshot project id did not match fixture project" || return 1
  assert_json_expr '.data.statuses | type == "array" and length > 0' \
    "Board snapshot is missing statuses" || return 1
  assert_json_expr ".data.tasks | any(.id == \"${TASK_ID}\")" \
    "Board snapshot is missing created task" || return 1
  assert_json_expr '.data.tags | type == "array"' \
    "Board snapshot tags should be an array" || return 1
  assert_json_expr '.data.taskTags | type == "array"' \
    "Board snapshot taskTags should be an array" || return 1
  assert_json_expr '.data.summary.activeTaskCount | type == "number"' \
    "Board snapshot summary.activeTaskCount is missing" || return 1
  return 0
}

test_get_board_missing_project() {
  api_request GET '/projects/00000000-0000-7000-8000-000000000404/board?includeArchived=false' || return 1
  assert_status '404' || return 1
  assert_error_envelope || return 1
  return 0
}

test_get_board_injection_query() {
  api_request GET "/projects/${PROJECT_ID}/board?includeArchived=%27%20OR%201%3D1%20--" || return 1
  assert_client_error || return 1
  return 0
}

setup_suite 'Board API'
run_case 'GET /projects/{projectId}/board returns normalized board snapshot' test_get_board_snapshot
run_case 'GET /projects/{projectId}/board returns 404 for unknown project' test_get_board_missing_project
run_case 'GET /projects/{projectId}/board rejects malformed includeArchived injection input' test_get_board_injection_query
finish_suite
