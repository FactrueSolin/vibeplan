#!/bin/sh

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "${SCRIPT_DIR}/lib/test_lib.sh"

PROJECT_ID=''
STATUS_ID=''
TASK_ID=''
TAG_ID=''

test_list_tags() {
  PROJECT_ID=$(create_project) || return 1
  STATUS_ID=$(get_first_status_id "${PROJECT_ID}") || return 1
  TASK_ID=$(create_task "${PROJECT_ID}" "${STATUS_ID}") || return 1

  api_request GET "/projects/${PROJECT_ID}/tags" || return 1
  assert_status '200' || return 1
  assert_list_envelope || return 1
  return 0
}

test_create_tag() {
  TAG_ID=$(create_tag "${PROJECT_ID}") || return 1
  api_request GET "/projects/${PROJECT_ID}/tags" || return 1
  assert_status '200' || return 1
  assert_list_envelope || return 1
  assert_json_expr ".data | any(.id == \"${TAG_ID}\")" \
    "Created tag was not returned by list endpoint" || return 1
  return 0
}

test_patch_tag() {
  body='{"name":"Backend Updated","color":"#059669"}'
  api_request PATCH "/tags/${TAG_ID}" "${body}" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1
  assert_json_expr '.data.name == "Backend Updated"' \
    "Updated tag name was not returned" || return 1
  return 0
}

test_bind_tag_to_task() {
  api_request PUT "/tasks/${TASK_ID}/tags/${TAG_ID}" || return 1
  assert_status_in '200 204' || return 1
  if [ "${LAST_RESPONSE_STATUS}" = '200' ]; then
    assert_success_envelope || return 1
  fi

  api_request GET "/tasks/${TASK_ID}" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1
  assert_json_expr ".data.tagIds | index(\"${TAG_ID}\") != null" \
    "Bound tag id was not returned by task detail" || return 1
  return 0
}

test_unbind_tag_from_task() {
  api_request DELETE "/tasks/${TASK_ID}/tags/${TAG_ID}" || return 1
  assert_status_in '200 204' || return 1
  if [ "${LAST_RESPONSE_STATUS}" = '200' ]; then
    assert_success_envelope || return 1
  fi

  api_request GET "/tasks/${TASK_ID}" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1
  assert_json_expr ".data.tagIds | index(\"${TAG_ID}\") == null" \
    "Unbound tag id should not remain on task detail" || return 1
  return 0
}

test_delete_tag() {
  tag_id=$(create_tag "${PROJECT_ID}") || return 1
  api_request DELETE "/tags/${tag_id}" || return 1
  assert_status '204' || return 1
  assert_empty_body || return 1
  return 0
}

test_create_tag_empty_name() {
  body='{"name":"","color":"#10b981"}'
  api_request POST "/projects/${PROJECT_ID}/tags" "${body}" || return 1
  assert_status '422' || return 1
  assert_error_envelope || return 1
  return 0
}

test_bind_tag_injection_path() {
  api_request PUT "/tasks/${TASK_ID}/tags/1%27%20OR%20%271%27%3D%271" || return 1
  assert_client_error || return 1
  return 0
}

setup_suite 'Tags API'
run_case 'GET /projects/{projectId}/tags returns tag list' test_list_tags
run_case 'POST /projects/{projectId}/tags creates a tag' test_create_tag
run_case 'PATCH /tags/{tagId} updates tag fields' test_patch_tag
run_case 'PUT /tasks/{taskId}/tags/{tagId} binds a tag to a task' test_bind_tag_to_task
run_case 'DELETE /tasks/{taskId}/tags/{tagId} unbinds a tag from a task' test_unbind_tag_from_task
run_case 'DELETE /tags/{tagId} deletes an existing tag' test_delete_tag
run_case 'POST /projects/{projectId}/tags rejects empty tag name' test_create_tag_empty_name
run_case 'PUT /tasks/{taskId}/tags/{tagId} rejects injection style path input' test_bind_tag_injection_path
finish_suite
