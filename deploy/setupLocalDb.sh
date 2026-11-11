#!/bin/bash
# Rebuild the local portal database from scratch: a fresh Postgres container, the
# tms_server schema, and the config data (login provider, tms client, allowed redirects).
# Deletes any existing local portal database.
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
REPO_DIR=$(dirname "${SCRIPT_DIR}")

PG_CONTAINER="tms_portal_postgres"
DB_ROLE="tms"
DB_NAME="tms_db"
DB_PORT="5432"
TMS_SERVER_GIT_BRANCH="main"
ENV_FILE="${HOME}/.config/Code/tms-trust-project/tms_portal/tms_portal_env.sh"
ASSUME_YES=0

function usage() {
  echo "$0 [-e env_file] [-p port] [-b branch] [-y]"

  echo "OPTIONS:"
  echo "     -e --env"
  echo "        File to source for GLOBUS_CLIENT_ID, GLOBUS_CLIENT_SECRET, TMS_PORTAL_BASE_URL, ..."
  echo "        (default ${ENV_FILE})"
  echo
  echo "     -p --port"
  echo "        Postgres port (default ${DB_PORT})"
  echo
  echo "     -b --branch"
  echo "        TMS Server github repository branch or commit for the migrations (default ${TMS_SERVER_GIT_BRANCH})"
  echo
  echo "     -y --yes"
  echo "        Don't ask before deleting the existing local database"
  echo
  exit 1
}

function announce() {
  echo ---==== $@ ====---
}

function fail() {
  echo "Error: $@" >&2
  exit 1
}

function psql_q() {
  docker exec -i ${PG_CONTAINER} psql -U ${DB_ROLE} ${DB_NAME} -v ON_ERROR_STOP=1 -tA "$@"
}

while [[ $# -gt 0 ]]; do
  case $1 in
    -e|--env)
      ENV_FILE="$2"
      shift # past argument
      shift # past value
      ;;
    -p|--port)
      DB_PORT="$2"
      shift # past argument
      shift # past value
      ;;
    -b|--branch)
      TMS_SERVER_GIT_BRANCH="$2"
      shift # past argument
      shift # past value
      ;;
    -y|--yes)
      ASSUME_YES=1
      shift # past argument
      ;;
    -*)
      echo "Unknown option $1"
      usage
      ;;
    *)
      echo "Unknown positional argument $1"
      usage
  esac
done

# the env file is optional when the variables are already exported
if [[ -f "${ENV_FILE}" ]] ; then
  announce "sourcing ${ENV_FILE}"
  # the env script isn't written for set -u
  set +u
  source "${ENV_FILE}"
  set -u
else
  echo "Env file not found: ${ENV_FILE} (using the current environment)"
fi

# check before deleting anything; prints names only, never values
for v in GLOBUS_CLIENT_ID GLOBUS_CLIENT_SECRET ; do
  [[ -n "${!v:-}" ]] || fail "$v is not set"
done
for cmd in docker http jq openssl envsubst uuidgen ; do
  command -v ${cmd} >/dev/null || fail "${cmd} is not installed"
done
# createConfigData.sh ignores an exported BASE_URL, so it's passed with -u
BASE_URL="${TMS_PORTAL_BASE_URL:-${BASE_URL:-http://localhost:8080}}"
BASE_URL="${BASE_URL%/}"

if [[ ${ASSUME_YES} -eq 0 ]] ; then
  read -r -p "This deletes the local portal database (docker compose down -v). Continue? [y/N] " answer
  [[ "${answer}" == [yY]* ]] || fail "cancelled"
fi

cd "${REPO_DIR}"

announce "removing the old database container and volume"
# set to quiet docker compose's warnings about unset variables
PG_PORT=${DB_PORT} PG_ADMIN_PASS=unused docker compose -f docker-compose.yml down -v

announce "starting a fresh postgres"
./setupTmsPortalPg.sh -p ${DB_PORT}

announce "running the tms_server migrations (${TMS_SERVER_GIT_BRANCH})"
rm -f deploy/migrations/*.sql
deploy/migrate.sh -p ${DB_PORT} -b ${TMS_SERVER_GIT_BRANCH}

shopt -s nullglob
MIGRATION_FILES=(deploy/migrations/*.sql)
MIGRATION_FILES=${#MIGRATION_FILES[@]}
shopt -u nullglob
APPLIED=$(psql_q -c 'select count(*) from _sqlx_migrations where success')
[[ ${MIGRATION_FILES} -gt 0 ]] || fail "no migration files were downloaded"
[[ ${APPLIED} -eq ${MIGRATION_FILES} ]] || fail "${APPLIED} of ${MIGRATION_FILES} migrations applied"
psql_q -c 'select version, description from _sqlx_migrations order by version'

announce "loading the config data"
# the generated SQL holds client secrets, so it stays private and is removed on exit
WORK_DIR=$(mktemp -d)
trap 'rm -rf "${WORK_DIR}"' EXIT
deploy/createConfigData.sh -u "${BASE_URL}" -o "${WORK_DIR}/tms_portal_config.sql" >/dev/null
psql_q -q < "${WORK_DIR}/tms_portal_config.sql"

# the Tapis login provider isn't in tms_portal_dev_config.sql; added only when its
# client is filled in (TMS_PORTAL_TAPIS_LOGIN_PROVIDER=tacc_login uses it)
case "${TAPIS_LOGIN_CLIENT_ID:-}${TAPIS_LOGIN_CLIENT_SECRET:-}" in
  ''|*...*)
    echo "TAPIS_LOGIN_CLIENT_ID/TAPIS_LOGIN_CLIENT_SECRET not set: skipping the tacc_login provider"
    ;;
  *)
    announce "adding the tacc_login identity provider"
    # passed as psql variables so the secret isn't expanded into the SQL text
    psql_q -q -v client_id="${TAPIS_LOGIN_CLIENT_ID}" -v client_secret="${TAPIS_LOGIN_CLIENT_SECRET}" <<'SQL'
INSERT INTO identity_providers
        (id, name, oauth2_client_id, oauth2_client_secret, identity_redirect_url,
                oauth2_token_url, oauth2_jwks_url, oidc_user_info_url,
                scope, provider_type, supports_login, supports_resources)
VALUES
        ('tacc_login', 'TACC (Tapis) login', :'client_id', :'client_secret',
                'https://tacc.tapis.io/v3/oauth2/authorize',
                'https://tacc.tapis.io/v3/oauth2/tokens',
                'https://tacc.tapis.io/v3/tokens/.well-known/jwks.json', '',
                'openid profile email', 'tacc_tapis', true, false);
SQL
    ;;
esac

announce "checking the config data"
[[ "$(psql_q -c "select supports_login from identity_providers where id = 'globus'")" == "t" ]] \
  || fail "no globus identity provider with supports_login"
[[ -n "$(psql_q -c "select 1 from allowed_redirects where client_id = 'tms' and uri = '${BASE_URL}/'")" ]] \
  || fail "${BASE_URL}/ is not an allowed redirect"
psql_q -c 'select id, supports_login from identity_providers order by id'
psql_q -c 'select client_id, uri from allowed_redirects order by client_id, uri'

announce "done: database ready on port ${DB_PORT}"
