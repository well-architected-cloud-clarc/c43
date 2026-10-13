use c43::drawio::graph::{build, doc_from_json};
use serde_json::{json, Value};

fn read(path: &str) -> String {
    std::fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

fn rebob() -> c43::drawio::graph::Built {
    build(&doc_from_json(&serde_json::from_str(&read("fixtures/layout-cases/rebob-container/c43.json")).unwrap()))
}

#[test]
fn rebob_container_dump_builds_groups_nodes_and_edges() {
    // what contains something is a group; the empty backend sub-bob is a plain node
    let b = rebob();
    assert_eq!(b.graph.title.as_deref(), Some("rebob"));
    assert_eq!(b.graph.groups.len(), 10);
    assert!(b.graph.groups.iter().all(|g| g.id.starts_with("backend:")));
    let sub_bob = b.graph.nodes.iter().find(|n| n.id == "backend:sub-bob").unwrap();
    assert_eq!((sub_bob.label.as_deref(), sub_bob.group.as_deref()), (Some("sub-bob"), None));
    assert!(!b.graph.edges.is_empty());
    assert!(b.warnings.is_empty(), "{:?}", b.warnings);
}

#[test]
fn system_view_backends_are_nodes() {
    let doc = doc_from_json(&json!({
        "nodes": [node("system:s", "system"), node("backend:b", "backend"), node("web", "frontend")],
        "relations": [rel("system:s", "contains", "backend:b"), rel("system:s", "contains", "web"), rel("web", "uses", "backend:b")]
    }));
    let b = build(&doc);
    assert!(b.graph.groups.is_empty());
    assert_eq!(b.graph.nodes.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(), ["backend:b", "web"]);
}

#[test]
fn hints_size_nodes_for_the_pretty_text_and_put_group_text_at_the_bottom() {
    let sizes = rebob().hints.sizes;
    // the groups carry a role, so their text is two lines
    assert_eq!(sizes.title_height, 2.0);
    assert!(sizes.title_bottom);
    // room for the hanging icon: no node under 5 units
    assert!(sizes.node.values().all(|s| *s >= 5.0), "{:?}", sizes.node);
}

#[test]
fn a_long_role_widens_its_node() {
    let with_role = |role: &str| json!({"uid": "n", "name": "n", "type": "x", "attributes": {"role": role}});
    let side = |role: &str| build(&doc_from_json(&json!({"nodes": [with_role(role)], "relations": []}))).hints.sizes.node["n"];
    assert!(side("unbreakablerolethatisfarlongerthananodeside") > side("api"));
}

#[test]
fn roles_are_read_from_the_output_for_nodes_and_groups() {
    let b = rebob();
    assert_eq!(b.meta["queue/task-queue"].role.as_deref(), Some("event bus"));
    assert_eq!(b.meta["backend:queue"].role.as_deref(), Some("service"));
}

#[test]
fn an_output_without_roles_falls_back_to_the_type_for_nodes_and_none_for_groups() {
    let doc = doc_from_json(&json!({
        "nodes": [node("system:s", "system"), node("backend:b", "backend"), node("b/api", "apicontainer")],
        "relations": [rel("system:s", "contains", "backend:b"), rel("backend:b", "contains", "b/api")]
    }));
    let b = build(&doc);
    assert_eq!(b.meta["b/api"].role.as_deref(), Some("apicontainer"));
    assert_eq!(b.meta["backend:b"].role, None);
}

#[test]
fn icons_follow_the_role() {
    let with_role = |uid: &str, role: &str| json!({"uid": uid, "name": uid, "type": "x", "attributes": {"role": role}});
    let doc = doc_from_json(&json!({
        "nodes": [with_role("p", "person"), with_role("d", "database"), with_role("o", "object store"), with_role("q", "event bus"), with_role("s", "service")],
        "relations": []
    }));
    let icons: Vec<_> = ["p", "d", "o", "q", "s"].iter().map(|id| build(&doc).meta[*id].icon).collect();
    use c43::drawio::pretty::Icon::*;
    assert_eq!(icons, [Some(User), Some(Db), Some(Store), Some(Bus), None]);
}

#[test]
fn the_system_is_a_boundary_group_only_when_something_sits_outside_it() {
    let nodes = |outsider: bool| {
        let mut nodes = vec![node("system:s", "system"), node("backend:b", "backend")];
        if outsider {
            nodes.push(node("person:u", "person"));
        }
        nodes
    };
    let relations = json!([rel("system:s", "contains", "backend:b")]);
    let alone = build(&doc_from_json(&json!({"nodes": nodes(false), "relations": relations})));
    assert!(alone.graph.groups.is_empty());
    assert_eq!(alone.graph.title.as_deref(), Some("system:s"));

    let with_person = build(&doc_from_json(&json!({"nodes": nodes(true), "relations": relations})));
    assert_eq!(with_person.graph.groups.iter().map(|g| g.id.as_str()).collect::<Vec<_>>(), ["system:s"]);
    let member = |id: &str| with_person.graph.nodes.iter().find(|n| n.id == id).unwrap().group.clone();
    assert_eq!(member("backend:b").as_deref(), Some("system:s"));
    assert_eq!(member("person:u"), None);
    // the title stays
    assert_eq!(with_person.graph.title.as_deref(), Some("system:s"));
}

fn node(uid: &str, ty: &str) -> Value {
    json!({"uid": uid, "name": uid, "type": ty, "attributes": {}})
}

fn rel(s: &str, is: &str, e: &str) -> Value {
    json!({"start": s, "is": is, "end": e})
}

#[test]
fn rejected_relations_become_warnings() {
    let doc = doc_from_json(&json!({
        "nodes": [node("system:s", "system"), node("g", "x"), node("h", "x"), node("a", "x"), node("b", "x")],
        "relations": [
            rel("system:s", "contains", "g"), rel("g", "contains", "a"), rel("h", "contains", "a"),
            rel("a", "uses", "a"), rel("a", "uses", "g"), rel("a", "uses", "ghost"), rel("a", "uses", "b")
        ]
    }));
    let b = build(&doc);
    assert_eq!(b.graph.title.as_deref(), Some("system:s"));
    let a = b.graph.nodes.iter().find(|n| n.id == "a").unwrap();
    assert_eq!(a.group.as_deref(), Some("g"));
    assert_eq!(b.graph.edges.iter().map(|e| (e.from.as_str(), e.to.as_str())).collect::<Vec<_>>(), [("a", "b")]);
    assert_eq!(b.warnings, [
        "a contained by both g and h, keeping g",
        "dropped a uses a: self-loop",
        "dropped a uses g: a member cannot point at its own group",
        "dropped a uses ghost: unknown node ghost",
    ]);
    assert!(c43_layout::layout(&b.graph, &b.hints, &Default::default()).is_ok());
}

#[test]
fn labels_only_with_several_kinds() {
    let nodes = json!([node("a", "x"), node("b", "x"), node("c", "x")]);
    let b = build(&doc_from_json(&json!({"nodes": nodes, "relations": [rel("a", "uses", "b"), rel("a", "reads", "b"), rel("b", "uses", "c")]})));
    // a label wider than a lane is split into lines
    assert_eq!(b.graph.edges.iter().map(|e| e.label.as_deref().map(|l| l.replace('\n', " "))).collect::<Vec<_>>(), [Some("uses, reads".to_string()), Some("uses".to_string())]);
    let b = build(&doc_from_json(&json!({"nodes": nodes, "relations": [rel("a", "uses", "b"), rel("b", "uses", "c")]})));
    assert!(b.graph.edges.iter().all(|e| e.label.is_none()));
}

#[test]
fn containment_cycles_are_dropped_with_a_warning() {
    let doc = doc_from_json(&json!({
        "nodes": [node("a", "x"), node("b", "x"), node("c", "x"), node("d", "x")],
        "relations": [rel("a", "contains", "b"), rel("b", "contains", "a"), rel("b", "contains", "c"), rel("c", "uses", "d"), rel("d", "contains", "d")]
    }));
    let b = build(&doc);
    assert_eq!(b.warnings, ["dropped b contains a: containment cycle", "dropped d contains d: containment cycle"]);
    assert!(c43::drawio::render(&doc).is_ok());
}
