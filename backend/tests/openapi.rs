//! Sanity checks on the generated OpenAPI document. These run offline
//! (no NCBI calls) and are the second layer of "did you forget to annotate?"
//! enforcement on top of the compile-time `routes!()` macro.

use pubmed_backend::openapi;

#[test]
fn every_expected_route_is_in_the_spec() {
    let spec = openapi();
    let paths: Vec<&str> = spec.paths.paths.keys().map(|s| s.as_str()).collect();

    for expected in [
        "/api/search",
        "/api/article/{pmid}",
        "/api/mesh",
        "/api/cite/{pmid}",
    ] {
        assert!(
            paths.contains(&expected),
            "missing OpenAPI path `{expected}`; spec has {paths:?}"
        );
    }
}

#[test]
fn search_response_schema_includes_elapsed_ms() {
    let spec = openapi();
    let json = serde_json::to_value(&spec).expect("spec to json");
    let props = &json["components"]["schemas"]["SearchResponse"]["properties"];
    assert!(
        props.get("elapsed_ms").is_some(),
        "SearchResponse schema missing `elapsed_ms` field"
    );
    assert!(props.get("results").is_some());
    assert!(props.get("count").is_some());
}

#[test]
fn every_path_declares_an_error_response() {
    let json = serde_json::to_value(&openapi()).expect("spec to json");
    let paths = json["paths"].as_object().expect("paths object");
    for (path, item) in paths {
        let methods = item.as_object().expect("path item object");
        for (method, op) in methods {
            // Path items also contain non-operation fields like "parameters",
            // "summary", "description"; restrict to HTTP verbs.
            if !matches!(method.as_str(), "get" | "post" | "put" | "delete" | "patch") {
                continue;
            }
            let responses = op["responses"].as_object().expect("responses object");
            assert!(
                responses.contains_key("500"),
                "{method} {path} should document a 500 response (has {:?})",
                responses.keys().collect::<Vec<_>>()
            );
        }
    }
}
