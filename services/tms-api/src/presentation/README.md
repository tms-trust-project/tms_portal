# Presentation Layer

The Presentation Layer is the outermost layer of this application responsible for showing information to the user and interpreting user commands.

It contains web interface and HTTP-related concerns such as request and response contracts (dtos), routes/endpoints definitons, route handlers, web-framework related, and interface versioning.

## Directory Structure

The Presentation layer uses a nested directory structure to categorize the code. The directory structure obeys the following pattern:

- Level 0: protocol
- Level 1: interface version (v1, v2)
- Level 2: high-level features (delegations, login, resources, etc)
- Level 3: handlers/request dtos/response DTOS, routers, etc
