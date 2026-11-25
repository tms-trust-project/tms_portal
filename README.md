# Documentation
|  |  |
| :---         | :--- |
| API docs     | https://tms-trust-project.github.io/tms-live-docs |
| docs source  | https://github.com/tapis-project/tms-live-docs    |


# Build Instructions

## Prerequistes
You'll need a rust compiler and docker.  You'll need npm if you want to build the UI.

## Rust Build
```
cargo clean
cargo build ( or cargo build --release )
```

## Setup the database
```
./setupTmsPortalPg.sh (or if you want use a custom port ./setupTmsPortalPg.sh -p <portnum>)
```

## Build the UI (only if you want to run the UI locally)
The UI's login link carries the portal url the login returns to, which is compiled into the
build from `REDIRECT_URI` (the build stops without it).  It must be one of the `tms` client's
allowed redirects (`BASE_URL` and `BASE_URL/` in the sample data).  For a local portal:
```
cd client
pnpm install
REDIRECT_URI=http://localhost:8080/ pnpm build
```
or, without node/pnpm installed, build it in docker (`REDIRECT_URI` defaults to
`http://localhost:8080/`):
```
./setupClient.sh
REDIRECT_URI=https://<portal host>/ ./setupClient.sh
```

The portal serves the UI from `dist/` in its working directory.  If you're running from the
top level of the git repo, create a symlink:
`ln -s ./client/dist/ ./dist`

The portal's docker image builds the UI itself; pass the url as a build arg
(`./buildTmsPortal.sh` defaults it to `http://localhost:8080/`):
```
REDIRECT_URI=https://<portal host>/ ./buildTmsPortal.sh
```

## Create the schema
run the following command to create the schema from the tms_server project.  It will download the 
sqlx migration files directly from that project, and apply them.
```
deploy/migrate.sh 
```

deploy/migrate.sh -? will give all of the command line options.  There are default values for everything,
so you only need those parameters if you want to change something - for example the port.

The schema is created the first time that you run the portal.
```
export TMS_PORTAL_DB_PASSWORD=tms_password
export TMS_PORTAL_DB_HOST=localhost
```
If you changed the port, you may need this also:
```
export TMS_PORTAL_DB_PORT=<tms_db_port>
```

## Populate db with sample data
Copy `deploy/vars.sh`, fill in the installation-specific values, and use the
configuration script to generate the seed SQL.
```
deploy/createConfigData.sh -v deploy/vars.sh
```
The script creates `output.sql`; it does not modify the database. Review the
file, then apply it explicitly when ready:
```
docker exec -i tms_portal_postgres \
  psql -v ON_ERROR_STOP=1 -U tms -d tms_db < output.sql
```
Run `deploy/createConfigData.sh -?` to see the generation and key options.

The `tms_portal_dev_config.sql` file is a template. The initial values will be
different for each installation.

## Configure the approved institutions
Every login goes through an approved institution: Globus sends the user straight to the
institution's own login page (`redirect_idp_domain`) and requires an identity from its domain
(`session_required_single_domain`), and the portal checks the identity again when Globus
redirects back.  The institutions are read at startup from a JSON file; the server won't
start without it.
```
export TMS_PORTAL_INSTITUTIONS_CONFIG_FILE=$PWD/deploy/institutions.sample.json
```
Each entry has an `id` (what the client sends as `institution`), a `name`, `enabled`, and
`idp_config.globus` with the `redirect_idp_domain` registered with Globus (e.g. `utexas.edu`)
and the `idp_logout_url` of the institution's own IdP sign-out page (empty if it has none).
The k8s deployments mount the same file from their ConfigMap
(`deploy/k8s/*/tms-api/config-map.yml`).

The Globus identity provider's scope should include
`urn:globus:auth:scope:auth.globus.org:view_identity_set` (the sample data does), so a user
can log in with an institution identity linked to a Globus account whose primary identity is
from somewhere else.  For an existing database:
```
update identity_providers
   set scope = 'openid profile email urn:globus:auth:scope:auth.globus.org:view_identity_set'
 where id = 'globus';
```

## Run
```
cargo run
```
Point the browser at http://localhost:8080/

Now the portal should be fully functional.  Log Out ends the Globus session and, when the
institution has an `idp_logout_url`, Globus's sign-out page links on to the institution's
own sign-out, so the next login asks for credentials again.  For tacc resource provider, you
can go to tacc.tapis.io, and from the chrome dev tools, you can clear the cookies to force a
new auth check.

