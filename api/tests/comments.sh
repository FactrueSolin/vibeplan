#!/bin/sh

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "${SCRIPT_DIR}/lib/test_lib.sh"

PROJECT_ID=''
STATUS_ID=''
TASK_ID=''
COMMENT_ID=''

test_list_comments() {
  PROJECT_ID=$(create_project) || return 1
  STATUS_ID=$(get_first_status_id "${PROJECT_ID}") || return 1
  TASK_ID=$(create_task "${PROJECT_ID}" "${STATUS_ID}") || return 1
  COMMENT_ID=$(create_comment "${TASK_ID}") || return 1

  api_request GET "/tasks/${TASK_ID}/comments" || return 1
  assert_status '200' || return 1
  assert_list_envelope || return 1
  assert_json_expr ".data | any(.id == \"${COMMENT_ID}\")" \
    "Comment list does not contain created comment" || return 1
  return 0
}

test_create_comment() {
  another_comment_id=$(create_comment "${TASK_ID}") || return 1
  api_request GET "/tasks/${TASK_ID}/comments" || return 1
  assert_status '200' || return 1
  assert_list_envelope || return 1
  assert_json_expr ".data | any(.id == \"${another_comment_id}\")" \
    "Created comment was not returned by list endpoint" || return 1
  return 0
}

test_patch_comment() {
  body='{"content":"Comment updated by test"}'
  api_request PATCH "/comments/${COMMENT_ID}" "${body}" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1
  assert_json_expr '.data.content == "Comment updated by test"' \
    "Updated comment content was not returned" || return 1
  return 0
}

test_delete_comment() {
  comment_id=$(create_comment "${TASK_ID}") || return 1
  api_request DELETE "/comments/${comment_id}" || return 1
  assert_status '204' || return 1
  assert_empty_body || return 1
  return 0
}

test_create_comment_empty_content() {
  body='{"authorName":"tester","content":""}'
  api_request POST "/tasks/${TASK_ID}/comments" "${body}" || return 1
  assert_status '422' || return 1
  assert_error_envelope || return 1
  return 0
}

test_patch_comment_injection_id() {
  body='{"content":"Injected comment update"}'
  api_request PATCH "/comments/1%27%20OR%20%271%27%3D%271" "${body}" || return 1
  assert_client_error || return 1
  return 0
}

setup_suite 'Comments API'
run_case 'GET /tasks/{taskId}/comments returns comments list' test_list_comments
run_case 'POST /tasks/{taskId}/comments creates a comment' test_create_comment
run_case 'PATCH /comments/{commentId} updates comment content' test_patch_comment
run_case 'DELETE /comments/{commentId} deletes comment' test_delete_comment
run_case 'POST /tasks/{taskId}/comments rejects empty content' test_create_comment_empty_content
run_case 'PATCH /comments/{commentId} rejects injection style path input' test_patch_comment_injection_id
finish_suite
