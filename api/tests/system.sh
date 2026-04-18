#!/bin/sh

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
. "${SCRIPT_DIR}/lib/test_lib.sh"

test_openapi_json() {
  api_request GET '/openapi.json' || return 1
  assert_status '200' || return 1
  assert_json_expr '.openapi | startswith("3.1")' \
    "OpenAPI version should start with 3.1" || return 1
  assert_json_expr '.info.title == "Plan API"' \
    "OpenAPI info.title should be Plan API" || return 1
  assert_json_expr '.info.version == "v1"' \
    "OpenAPI info.version should be v1" || return 1
  assert_json_expr '.paths | type == "object" and length > 0' \
    "OpenAPI paths should not be empty" || return 1
  return 0
}

setup_suite 'System API'
run_case 'GET /openapi.json returns OpenAPI 3.1 document' test_openapi_json
finish_suite
