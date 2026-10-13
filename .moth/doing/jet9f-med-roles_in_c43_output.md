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

## Specification (decisions of the brainstorm; they supersede the C4 framing and the scope above where they differ)

### Framing: layered architecture-from-code, not strict C4
- c43 is not a C4 tool. It describes the layers of a solution architecture declared in code. The word "C4" in the title and the examples above is historical; the role vocabulary is c43's own. C4 names survive only as optional labels in a theme.
- Layers follow the tree, as today: the **system** view lists `Architecture`s (and the packages that consume them); the **container** view lists the children of an `Architecture` (api containers, functions, stores). `system:<repo>` is the group around the system view's nodes.
- Rejected: matching C4 levels exactly. Under strict C4 an `Architecture` is a deployable container and its children are components, which leaves a store with no container role and makes rebob 13 containers in one system; that fits enterprise architecture, not solution architecture from code. Rejected: the enterprise-boundary vs system-boundary distinction; there is one boundary, the repo group, and "outside" is expressed by `external` (below). A name like "Hello Inc." has no source and is dropped; the group is named after the repo.

### Roles in the output
- Every node and group carries a role in `attributes.role` (the field already exists in the model and is read by the drawio rendering, which falls back to the node type when absent). The ascii output does not show the role beyond what the type already says, so as not to repeat itself (a role is shown only when it differs from the type label).
- Role vocabulary per layer:
  - system view: `person`, `external system` (declared entities, below), `system` (an `Architecture`; the flavour of a final consumer package comes from the existing package classification: `web` for frontend, `client` for client, `service` for backend), and the repo group, whose role is `boundary`;
  - container view: `api`, `function`, `database`, `object store`, `event bus`, from the construct kind (e.g. a JSON store is `database`).
- The role comes from a type/kind table in c43; consumers (drawio, ascii, agents) never derive it.
- Icons of a node (`user`, `db`, `store`, `bus`) come from the role, replacing the per-case `icons` hints. Whole `Architecture`s that behave like a store or bus (rebob: queue, memory, event-storage) stay `system` for now and keep their hint-based icon; deriving a store/bus role for an `Architecture` is deferred (queue holds one store-like construct, memory and event-storage also hold functions and api containers, so a rule would be a guess).
- The synthetic `hello-world-mermaid` fixture is retired: it mixed system context with a lifted store. The target is the real output for the code, the README picture follows the code and is re-rendered from it.

### Declared entities (things the code cannot show)
A person or a system outside the analysed code cannot be derived from imports, so it is declared in the architecture code, in cdk-arch (a separate task in that repo, a prerequisite of the c43 part below):
- `external`: an exported marker `Architecture`. Anything whose scope is `external` is outside the solution; everything else is inside. Containment is the construct tree, so "external" is not a flag. An internal person is simply an entity scoped under a regular `Architecture`.
- `Entity(scope, id, kind)`: a construct with no runtime part; `kind` is `'person' | 'system'` (kept small on purpose; subclasses may come later if the call sites read badly).
- `Entity.uses(target, label?)`: a declared relation to another construct; the label is the relation's description. This is the only place a relation label comes from for now. Relations derived from code keep the default label `uses`; a label override for derived relations is deferred.
- Where it is declared: in the consuming package, in its own `architecture.ts` (e.g. the example `web/src/architecture.ts`: an `Architecture` for the web app plus `new Entity(external, 'user', 'person').uses(webArch, 'says hello')`). The main architecture does not import the web app. A frontend or client package therefore gets a small `architecture.ts` to be addressable; it still gets its type (`frontend`/`client`) from the existing package classification, a declaration saying only that it exists and who uses it.
- Rejected: a web-app construct (a web app with its own `Architecture` is a system in this model, like the other `Architecture`s); declaring derivable edges such as web → api (c43 derives them from imports); an `isEnterpriseBoundary()` flag on `Architecture`; an `external` flag on entities.

### What c43 does with them
- Reads `Entity` declarations like it reads other constructs: node `type` `person` or `system`-entity, role as above, relations from `uses`. The scanner recognises `external` by the identifier imported from the cdk-arch package.
- A person or external system gets no `contains` relation from `system:<repo>`; it sits outside the repo group, which is what "external" means in the picture.
- Check done: constructing an `Architecture` at module level has no import-time side effects (root construct, no parent), so the exported `external` is safe; c43 special-cases it so it never becomes a node.

### Examples checked against the design
- hello world (cdk-arch sample): the user is declared in the web package; web → api is derived. Store is a container-view `database`.
- reveal-interact: presenter → plugin (client), audience → web example (frontend); reveal.js itself could be a declared external system (optional). Its data store is a container-view database.
- rebob: one person → `@bob/frontend`; `sub-bob-bootstrap` has no person (the manager starts it); the llm architectures probably wrap an external provider, to be declared if wanted. queue, memory, event-storage stay `system`.

### Order of work
1. cdk-arch task (neighbouring repo): `external`, `Entity`, `uses`; published release that the consumers (rebob, reveal-interact, the sample) can depend on.
2. This task, c43 side: role table and `attributes.role` at both layers, icons from role, entity reading, repo group role; golden updates; retire the synthetic fixture and replace it by the sample's real output; add the small `architecture.ts` to the sample web package.
3. Consumers adopt (rebob frontend, reveal-interact plugin and web): their own tasks.

### Decisions taken while implementing
- **Role values.** system view: `boundary` (the repo group), `service` (an Architecture), `web` (frontend), `client`, `person`, `external system`, `system` (an entity of kind system declared inside the solution). Container view: `api`, `function`, `database`, `object store`, `event bus`, and `container` for anything of no known kind. A consumer reading an output without roles falls back to the node type for nodes and shows no role for groups.
- **Container roles come from the class name.** cdk-arch has base classes only for the api container and the function. A store or bus is the solution's own class, so the role is chosen by words in its class name (queue, bus, topic, stream → event bus; bucket, blob, object store, file store → object store; store, storage, database, db, graph, table, repository → database), first match winning; functions are always `function`; a class deriving from the api container but named like a store (a JSON store) is a `database`. Rejected: a role from the base kind alone (a JSON store is an api container by inheritance and would be an `api`).
- **Declared entities in the output.** Node type `person` or `externalsystem` (`subsystem` for a system entity inside the solution), uid `person:<id>` / `system-entity:<id>`, attributes `kind` = `entity`. Relation kind (`is`) is the declared label, `uses` when none. An entity is external when its scope is the cdk-arch marker `external` (also under an import alias); external entities have no `contains` from the repo node, internal ones do. The cdk-arch package itself never contributes nodes (its `external` is a library internal). Declarations recognised: `new Entity(scope, id, kind)` as a variable initialiser or as a bare statement, with chained `.uses(target, label?)`, and `variable.uses(target, label?)` statements in the same file.
- **A frontend or client with an Architecture of its own is one node.** An Architecture that nothing is scoped under, in a package that consumes the system's architectures, only names that package: the package stays the `frontend`/`client` node (uid is the package name, role `web`/`client`) and no `backend:` node is made for it; declared relations to that Architecture point at the package node. An Architecture with members stays a backend whatever the package's dependencies.
- **Drawio.** Icons come from the role (person → user, database → db, object store → store, event bus → bus); the per-case hints still override them (rebob keeps its hints for whole Architectures that behave as stores or buses). The repo node is the diagram title as before; when some node sits outside it (a declared person or external system) it is also drawn as a boundary group around what it contains, with its role `boundary` as the group text. Without outsiders the output is unchanged.
- **Ascii.** An entity is a root line (`Person: user [says hello: web-app]`). A role is appended in parentheses only when the type does not already say it: never for the system-level types, `container`, `function` or `api`; so `JsonStore: things (database)`, `TaskQueue: jobs (event bus)`.
- **Fixtures.** The synthetic `hello-world-mermaid` case is gone. `hello-c43-system` is the real output of the sample (`samples/cdk-arch/packages/example`, which now declares its user in `web/src/architecture.ts`), `rebob-*` dumps were regenerated (same nodes, now with roles). The `hello-c43-system` hints are empty: its icons come from roles.
- **Done on the cdk-arch side:** `external`, `Entity`, `Entity.uses` and version 0.8.0 (cdk-arch repo, branch `jet9f-entities`, not published). Consumers (rebob, reveal-interact) adopt in their own tasks; until then they get roles but no persons.

### Open
- Whether a label override for derived relations is ever needed ("persists hellos" on api → store).
- Whether `Architecture`s that are really a store or bus should be declarable as such.
- Deployment-level view (`cmd/deployment.rs`, not wired into the CLI) still emits no roles.
