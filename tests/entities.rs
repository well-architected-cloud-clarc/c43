use std::fs;
use std::path::Path;
use tempfile::tempdir;
use c43::ascii;
use c43::cmd::{container, system};
use c43::model::{C4Document, Node};

fn package(root: &Path, name: &str, deps: &[&str], files: &[(&str, &str)]) {
    let dir = root.join("packages").join(name.replace('/', "-"));
    fs::create_dir_all(dir.join("src")).unwrap();
    let deps = deps.iter().map(|d| format!("\"{d}\": \"*\"")).collect::<Vec<_>>().join(", ");
    fs::write(dir.join("package.json"), format!(r#"{{"name": "{name}", "version": "1.0.0", "dependencies": {{{deps}}}}}"#)).unwrap();
    files.iter().for_each(|(file, source)| fs::write(dir.join("src").join(file), source).unwrap());
}

fn node<'a>(doc: &'a C4Document, uid: &str) -> &'a Node {
    doc.nodes.iter().find(|n| n.uid == uid).unwrap_or_else(|| panic!("no node {uid}: {:?}", doc.nodes.iter().map(|n| &n.uid).collect::<Vec<_>>()))
}

fn role<'a>(doc: &'a C4Document, uid: &str) -> Option<&'a str> {
    node(doc, uid).attributes.role.as_deref()
}

fn has(doc: &C4Document, start: &str, is: &str, end: &str) -> bool {
    doc.relations.iter().any(|r| r.start == start && r.is == is && r.end == end)
}

const API: &str = r#"
    import { Architecture, ApiContainer, Function, Construct } from '@arinoto/cdk-arch';
    export const arch = new Architecture('svc');
    class JsonStore extends ApiContainer {}
    class TaskQueue extends Construct {}
    export const store = new JsonStore(arch, 'things', {});
    export const queue = new TaskQueue(arch, 'jobs');
    export const hello = new Function(arch, 'hello', async () => 'hi');
    export const api = new ApiContainer(arch, 'api', { hello: { path: 'GET /hello', handler: hello } });
"#;

/// a backend, and a web package (react) that uses it and declares its own Architecture
fn workspace(root: &Path, web_architecture: &str) {
    package(root, "svc-arch", &[], &[("arch.ts", API)]);
    package(root, "web-app", &["react"], &[
        ("App.tsx", "import { api } from 'svc-arch'; export const App = () => api;"),
        ("architecture.ts", web_architecture),
    ]);
}

#[test]
fn a_declared_person_is_outside_the_system_and_uses_the_web_node_with_its_label() {
    let dir = tempdir().unwrap();
    workspace(dir.path(), r#"
        import { Architecture, Entity, external } from '@arinoto/cdk-arch';
        export const webArch = new Architecture('web');
        export const user = new Entity(external, 'user', 'person').uses(webArch, 'says hello');
    "#);
    let doc = system::run(dir.path());

    let person = node(&doc, "person:user");
    assert_eq!((person.node_type.as_str(), role(&doc, "person:user")), ("person", Some("person")));
    assert!(has(&doc, "person:user", "says hello", "web-app"));
    // outside: nothing contains it
    assert!(!doc.relations.iter().any(|r| r.is == "contains" && r.end == "person:user"));
    // the web Architecture is the frontend node, not a backend of its own
    assert_eq!((node(&doc, "web-app").node_type.as_str(), role(&doc, "web-app")), ("frontend", Some("web")));
    assert!(!doc.nodes.iter().any(|n| n.uid == "backend:web"));
    assert!(has(&doc, "web-app", "uses", "backend:svc"));
    assert_eq!(role(&doc, "backend:svc"), Some("service"));
    assert_eq!(role(&doc, &doc.nodes[0].uid), Some("boundary"));
}

#[test]
fn uses_called_on_a_variable_defaults_to_the_label_uses() {
    let dir = tempdir().unwrap();
    workspace(dir.path(), r#"
        import { Architecture, Entity, external } from '@arinoto/cdk-arch';
        export const webArch = new Architecture('web');
        const visitor = new Entity(external, 'visitor', 'person');
        visitor.uses(webArch);
    "#);
    assert!(has(&system::run(dir.path()), "person:visitor", "uses", "web-app"));
}

#[test]
fn an_entity_may_use_a_backend_architecture_or_something_inside_it() {
    let dir = tempdir().unwrap();
    workspace(dir.path(), r#"
        import { Architecture, Entity, external } from '@arinoto/cdk-arch';
        import { arch, api } from 'svc-arch';
        export const webArch = new Architecture('web');
        new Entity(external, 'bot', 'person').uses(api, 'polls');
        new Entity(external, 'cron', 'system').uses(arch);
    "#);
    let doc = system::run(dir.path());
    assert!(has(&doc, "person:bot", "polls", "backend:svc"));
    assert!(has(&doc, "system-entity:cron", "uses", "backend:svc"));
}

#[test]
fn an_external_system_has_its_own_type_and_role() {
    let dir = tempdir().unwrap();
    workspace(dir.path(), r#"
        import { Architecture, Entity, external } from '@arinoto/cdk-arch';
        import { arch } from 'svc-arch';
        export const webArch = new Architecture('web');
        new Entity(external, 'mailer', 'system');
    "#);
    let doc = system::run(dir.path());
    assert_eq!((node(&doc, "system-entity:mailer").node_type.as_str(), role(&doc, "system-entity:mailer")), ("externalsystem", Some("external system")));
}

#[test]
fn an_entity_under_a_regular_architecture_is_part_of_the_system() {
    let dir = tempdir().unwrap();
    workspace(dir.path(), r#"
        import { Architecture, Entity } from '@arinoto/cdk-arch';
        export const webArch = new Architecture('web');
        new Entity(webArch, 'admin', 'person').uses(webArch, 'manages');
    "#);
    let doc = system::run(dir.path());
    assert!(has(&doc, &doc.nodes[0].uid, "contains", "person:admin"));
    assert!(has(&doc, "person:admin", "manages", "web-app"));
}

#[test]
fn external_is_recognised_under_an_import_alias() {
    let dir = tempdir().unwrap();
    workspace(dir.path(), r#"
        import { Architecture, Entity, external as outside } from '@arinoto/cdk-arch';
        export const webArch = new Architecture('web');
        new Entity(outside, 'visitor', 'person').uses(webArch);
    "#);
    let doc = system::run(dir.path());
    assert!(!doc.relations.iter().any(|r| r.is == "contains" && r.end == "person:visitor"));
}

#[test]
fn the_marker_architecture_in_the_cdk_arch_package_is_not_a_node() {
    let dir = tempdir().unwrap();
    workspace(dir.path(), r#"
        import { Architecture, Entity, external } from '@arinoto/cdk-arch';
        export const webArch = new Architecture('web');
        new Entity(external, 'user', 'person').uses(webArch);
    "#);
    package(dir.path(), "@arinoto/cdk-arch", &[], &[("entity.ts", "import { Architecture } from './architecture'; export const external = new Architecture('external');")]);
    let doc = system::run(dir.path());
    assert!(!doc.nodes.iter().any(|n| n.uid == "backend:external"));
}

#[test]
fn an_architecture_with_members_stays_a_backend_even_in_a_package_with_web_deps() {
    let dir = tempdir().unwrap();
    package(dir.path(), "svc-arch", &["react"], &[("arch.ts", API)]);
    let doc = system::run(dir.path());
    assert_eq!(node(&doc, "backend:svc").node_type, "backend");
}

#[test]
fn container_roles_follow_the_construct_kind_and_class_name() {
    let dir = tempdir().unwrap();
    package(dir.path(), "svc-arch", &[], &[("arch.ts", API)]);
    let doc = container::run(dir.path());
    assert_eq!(role(&doc, "svc/things"), Some("database"));
    assert_eq!(role(&doc, "svc/jobs"), Some("event bus"));
    assert_eq!(role(&doc, "svc/api"), Some("api"));
}

#[test]
fn ascii_shows_a_role_only_when_the_type_does_not_say_it() {
    let dir = tempdir().unwrap();
    workspace(dir.path(), r#"
        import { Architecture, Entity, external } from '@arinoto/cdk-arch';
        export const webArch = new Architecture('web');
        new Entity(external, 'user', 'person').uses(webArch, 'says hello');
    "#);
    let out = ascii::render(&container::run(dir.path()));
    assert!(out.contains("Person: user [says hello: web-app]"), "{out}");
    assert!(out.contains("jsonstore: things (database)") || out.contains("JsonStore: things (database)"), "{out}");
    assert!(out.contains("taskqueue: jobs (event bus)") || out.contains("TaskQueue: jobs (event bus)"), "{out}");
    assert!(!out.contains("(api)") && !out.contains("(service)") && !out.contains("(web)"), "{out}");
}
