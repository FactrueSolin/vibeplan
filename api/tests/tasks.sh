#!/bin/sh

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "${SCRIPT_DIR}/lib/test_lib.sh"

PROJECT_ID=''
STATUS_ID=''
TASK_ID=''

test_list_tasks() {
  PROJECT_ID=$(create_project) || return 1
  STATUS_ID=$(get_first_status_id "${PROJECT_ID}") || return 1
  TASK_ID=$(create_task "${PROJECT_ID}" "${STATUS_ID}") || return 1

  api_request GET "/projects/${PROJECT_ID}/tasks?page=1&pageSize=20&sortBy=position&sortOrder=asc&archived=exclude" || return 1
  assert_status '200' || return 1
  assert_paginated_list_envelope || return 1
  assert_json_expr ".data | any(.id == \"${TASK_ID}\")" \
    "Task list does not contain created task" || return 1
  return 0
}

test_create_task() {
  second_task_id=$(create_task "${PROJECT_ID}" "${STATUS_ID}") || return 1
  api_request GET "/tasks/${second_task_id}" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1
  assert_json_expr ".data.id == \"${second_task_id}\"" \
    "Created task could not be fetched back" || return 1
  return 0
}

test_patch_task() {
  body=$(cat <<EOF
{"title":"Task Updated","description":"Updated description","priority":"high","statusId":"${STATUS_ID}","dueDate":null,"tagIds":[]}
EOF
)
  api_request PATCH "/tasks/${TASK_ID}" "${body}" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1
  assert_json_expr '.data.title == "Task Updated"' \
    "Updated task title was not returned" || return 1
  assert_json_expr '.data.priority == "high"' \
    "Updated task priority was not returned" || return 1
  assert_json_expr '.data.dueDate == null' \
    "Updated task dueDate should be nullable" || return 1
  return 0
}

test_reorder_tasks() {
  another_task_id=$(create_task "${PROJECT_ID}" "${STATUS_ID}") || return 1
  body=$(cat <<EOF
{"movedTaskId":"${another_task_id}","sourceStatusId":"${STATUS_ID}","destinationStatusId":"${STATUS_ID}","orderedTaskIds":["${another_task_id}","${TASK_ID}"]}
EOF
)
  api_request POST "/projects/${PROJECT_ID}/tasks/reorder" "${body}" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1

  api_request GET "/projects/${PROJECT_ID}/tasks?page=1&pageSize=20&sortBy=position&sortOrder=asc&archived=exclude" || return 1
  assert_status '200' || return 1
  assert_paginated_list_envelope || return 1
  assert_json_expr ".data[0].id == \"${another_task_id}\"" \
    "Reordered task was not moved to the first position" || return 1
  return 0
}

test_archive_and_restore_task() {
  api_request POST "/tasks/${TASK_ID}/archive" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1
  assert_json_expr '.data.archivedAt != null' \
    "Archived task should include archivedAt" || return 1

  api_request POST "/tasks/${TASK_ID}/restore" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1
  assert_json_expr '.data.archivedAt == null' \
    "Restored task should clear archivedAt" || return 1
  return 0
}

test_delete_archived_task() {
  archived_task_id=$(create_task "${PROJECT_ID}" "${STATUS_ID}") || return 1
  api_request POST "/tasks/${archived_task_id}/archive" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1

  api_request DELETE "/tasks/${archived_task_id}" || return 1
  assert_status '204' || return 1
  assert_empty_body || return 1
  return 0
}

test_create_task_invalid_date_range() {
  body=$(cat <<EOF
{"statusId":"${STATUS_ID}","title":"Invalid date task","description":"bad input","priority":"medium","startDate":"2026-04-20","dueDate":"2026-04-18","tagIds":[]}
EOF
)
  api_request POST "/projects/${PROJECT_ID}/tasks" "${body}" || return 1
  assert_status '422' || return 1
  assert_error_envelope || return 1
  return 0
}

test_create_task_invalid_priority() {
  body=$(cat <<EOF
{"statusId":"${STATUS_ID}","title":"Invalid priority task","description":"bad input","priority":"critical","startDate":"2026-04-18","dueDate":"2026-04-19","tagIds":[]}
EOF
)
  api_request POST "/projects/${PROJECT_ID}/tasks" "${body}" || return 1
  assert_status '422' || return 1
  assert_error_envelope || return 1
  return 0
}

test_patch_task_injection_status() {
  body=$(cat <<EOF
{"statusId":"1' OR '1'='1","title":"Injected task update","tagIds":[]}
EOF
)
  api_request PATCH "/tasks/${TASK_ID}" "${body}" || return 1
  assert_client_error || return 1
  return 0
}

setup_suite 'Tasks API'
run_case 'GET /projects/{projectId}/tasks returns paginated tasks' test_list_tasks
run_case 'POST /projects/{projectId}/tasks creates a task' test_create_task
run_case 'PATCH /tasks/{taskId} updates mutable task fields' test_patch_task
run_case 'POST /projects/{projectId}/tasks/reorder updates task order' test_reorder_tasks
run_case 'POST /tasks/{taskId}/archive and /restore toggle archived state' test_archive_and_restore_task
run_case 'DELETE /tasks/{taskId} deletes an archived task' test_delete_archived_task
run_case 'POST /projects/{projectId}/tasks rejects invalid date range' test_create_task_invalid_date_range
run_case 'POST /projects/{projectId}/tasks rejects invalid priority enum' test_create_task_invalid_priority
run_case 'PATCH /tasks/{taskId} rejects injection style statusId input' test_patch_task_injection_status
finish_suite
