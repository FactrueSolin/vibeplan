#!/bin/sh

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "${SCRIPT_DIR}/lib/test_lib.sh"

PROJECT_ID=''

test_list_projects() {
  api_request GET '/projects?page=1&pageSize=20' || return 1
  assert_status '200' || return 1
  assert_paginated_list_envelope || return 1
  return 0
}

test_create_project() {
  suffix=$(random_suffix)
  body=$(cat <<EOF
{"name":"Project ${suffix}","slug":"project-${suffix}","description":"Created by test","color":"#0f766e"}
EOF
)

  api_request POST '/projects' "${body}" || return 1
  assert_status '201' || return 1
  assert_success_envelope || return 1
  assert_json_expr '.data.name | startswith("Project ")' \
    "Created project name was not returned" || return 1
  assert_json_expr '.data.slug | startswith("project-")' \
    "Created project slug was not returned" || return 1
  PROJECT_ID=$(json_query '.data.id') || return 1
  register_project_cleanup "${PROJECT_ID}"
  return 0
}

test_get_project() {
  api_request GET "/projects/${PROJECT_ID}" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1
  assert_json_expr ".data.id == \"${PROJECT_ID}\"" \
    "Fetched project id did not match created project" || return 1
  return 0
}

test_patch_project() {
  body='{"name":"Project Updated","description":"Updated description","color":"#1d4ed8"}'
  api_request PATCH "/projects/${PROJECT_ID}" "${body}" || return 1
  assert_status '200' || return 1
  assert_success_envelope || return 1
  assert_json_expr '.data.name == "Project Updated"' \
    "Updated project name was not returned" || return 1
  assert_json_expr '.data.description == "Updated description"' \
    "Updated project description was not returned" || return 1
  return 0
}

test_create_project_empty_name() {
  body='{"name":"","slug":"invalid-project","description":"bad input","color":"#1d4ed8"}'
  api_request POST '/projects' "${body}" || return 1
  assert_status '422' || return 1
  assert_error_envelope || return 1
  return 0
}

test_create_project_sql_injection_slug() {
  body=$(cat <<EOF
{"name":"Injected Project","slug":"bad-project'; DROP TABLE tasks; --","description":"bad input","color":"#1d4ed8"}
EOF
)
  api_request POST '/projects' "${body}" || return 1
  assert_client_error || return 1
  return 0
}

test_get_missing_project() {
  api_request GET '/projects/00000000-0000-7000-8000-000000000404' || return 1
  assert_status '404' || return 1
  assert_error_envelope || return 1
  return 0
}

test_delete_project() {
  project_id=$(create_project) || return 1
  api_request DELETE "/projects/${project_id}" || return 1
  assert_status '204' || return 1
  assert_empty_body || return 1
  return 0
}

setup_suite 'Projects API'
run_case 'GET /projects returns paginated project list' test_list_projects
run_case 'POST /projects creates a project' test_create_project
run_case 'GET /projects/{projectId} returns the created project' test_get_project
run_case 'PATCH /projects/{projectId} updates mutable project fields' test_patch_project
run_case 'POST /projects rejects empty project name' test_create_project_empty_name
run_case 'POST /projects rejects SQL injection style slug input' test_create_project_sql_injection_slug
run_case 'GET /projects/{projectId} returns 404 for unknown project' test_get_missing_project
run_case 'DELETE /projects/{projectId} deletes an existing project' test_delete_project
finish_suite
