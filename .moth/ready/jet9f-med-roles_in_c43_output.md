c43's system and container output should carry the C4 role of every node (and group), so consumers (the pretty drawio theme, f8ew1; ascii; agents) read it from the output instead of deriving it from the node type themselves.

Examples of roles: person, software system, web application, client application, enterprise boundary, system boundary, container kinds (function, api, database, object store, event bus).

Open: attribute name and place in the JSON (node `attributes.role`?), the type/kind -> role table for system and container level, roles for groups (boundaries), ascii output showing the role or not.


----- AI agent updates -------

## Additional scope (from f8ew1)

- The synthetic case `fixtures/layout-cases/hello-world-mermaid/c43.json` (the Hello world mermaid of the Pretty C4 ticket) carries more than c43 generates today. c43's output should grow to what it carries, so the synthetic file can be replaced by real output:
  - a role per node and group (person, software system, database, enterprise boundary, ...);
  - a person node kind (the user of the system) next to frontend/client;
  - a database node kind (a data store as a first-class node kind);
  - an enterprise/system boundary group around the system's nodes;
  - relation descriptions ("says hello", "persists hellos") as relation labels.
- The icon of a node (`user`, `db`, `store`, `bus`) is also to come from the output (node kind) instead of the per-case `icons` hints used by f8ew1 for now (rebob: queue → bus, memory and event-storage → db, frontend and client → user).
- Until then f8ew1 treats a node's role as its type.
