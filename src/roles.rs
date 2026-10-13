//! Roles: what a node is in the architecture, by layer. Set once here, read by every consumer
//! (drawio, ascii, agents) from `attributes.role`.
use crate::extract::ConstructKind;

/// the repo group around the system view's nodes
pub const BOUNDARY: &str = "boundary";
/// a person using the solution (a declared entity)
pub const PERSON: &str = "person";
/// a system outside the solution (a declared entity)
pub const EXTERNAL_SYSTEM: &str = "external system";
/// a system declared inside the solution (an entity of kind system under a regular Architecture)
pub const SYSTEM: &str = "system";
/// an Architecture that provides an api
pub const SERVICE: &str = "service";
/// a final consumer detected as a web application
pub const WEB: &str = "web";
/// any other final consumer
pub const CLIENT: &str = "client";

pub const API: &str = "api";
pub const FUNCTION: &str = "function";
pub const DATABASE: &str = "database";
pub const OBJECT_STORE: &str = "object store";
pub const EVENT_BUS: &str = "event bus";
/// a container of no known kind
pub const CONTAINER: &str = "container";

/// Role of a container-layer construct. cdk-arch has base classes only for the api container and
/// the function; stores and buses are the solution's own classes, recognised by what their class
/// name says (first match wins: bus, object store, database). A store served through an api
/// container base class (a JSON store) is still a database; anything else is an api or a plain container.
pub fn container_role(kind: ConstructKind, class_name: &str) -> &'static str {
    const BY_NAME: &[(&[&str], &str)] = &[
        (&["queue", "bus", "topic", "stream"], EVENT_BUS),
        (&["bucket", "blob", "objectstore", "filestore"], OBJECT_STORE),
        (&["store", "storage", "database", "db", "graph", "table", "repository"], DATABASE),
    ];
    if kind == ConstructKind::Function {
        return FUNCTION;
    }
    let name = class_name.to_lowercase();
    BY_NAME
        .iter()
        .find(|(words, _)| words.iter().any(|w| name.contains(w)))
        .map_or(if kind == ConstructKind::ApiContainer { API } else { CONTAINER }, |(_, role)| role)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_roles() {
        let role = container_role;
        assert_eq!(role(ConstructKind::Function, "Function"), FUNCTION);
        assert_eq!(role(ConstructKind::ApiContainer, "ApiContainer"), API);
        // a store served through an api container base class is still a database
        assert_eq!(role(ConstructKind::ApiContainer, "JsonStore"), DATABASE);
        assert_eq!(role(ConstructKind::Construct, "DataStore"), DATABASE);
        assert_eq!(role(ConstructKind::Construct, "Graph"), DATABASE);
        assert_eq!(role(ConstructKind::Construct, "TaskQueue"), EVENT_BUS);
        assert_eq!(role(ConstructKind::Construct, "MessageBus"), EVENT_BUS);
        assert_eq!(role(ConstructKind::Construct, "BlobBucket"), OBJECT_STORE);
        assert_eq!(role(ConstructKind::Construct, "McpContainer"), CONTAINER);
    }
}
