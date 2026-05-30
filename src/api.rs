use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::{OpenApi, ToSchema};

/// This service's identity. `srvcs-cardinality` is a leaf: it depends on no
/// other service. It counts the distinct elements of a list of integers
/// entirely with local logic.
pub const SERVICE: &str = "srvcs-cardinality";
pub const CONCERN: &str = "sets: number of distinct elements";
pub const DEPENDS_ON: &[&str] = &[];

#[derive(Serialize, ToSchema)]
pub struct Info {
    pub service: &'static str,
    pub concern: &'static str,
    pub depends_on: Vec<&'static str>,
}

/// `GET /` — service identity (srvcs service standard).
#[utoipa::path(get, path = "/", responses((status = 200, body = Info)))]
pub async fn index() -> Json<Info> {
    Json(Info {
        service: SERVICE,
        concern: CONCERN,
        depends_on: DEPENDS_ON.to_vec(),
    })
}

#[derive(Deserialize, ToSchema)]
pub struct EvalRequest {
    /// The list of integers to count. Every element must be a JSON integer.
    #[schema(value_type = Object)]
    pub values: Vec<Value>,
}

#[derive(Serialize, ToSchema)]
pub struct CardinalityResponse {
    #[schema(value_type = Object)]
    pub values: Vec<Value>,
    pub result: i64,
}

/// The single concern: the number of distinct integers in `values`.
///
/// Returns `None` if any element is not a JSON integer; otherwise `Some` of the
/// count of distinct elements read as `i64`. The empty list has cardinality 0.
pub fn cardinality(values: &[Value]) -> Option<i64> {
    let mut seen = std::collections::HashSet::new();
    for v in values {
        match v.as_i64() {
            Some(n) => {
                seen.insert(n);
            }
            None => return None,
        }
    }
    Some(seen.len() as i64)
}

/// `POST /` — the number of distinct integers in the list `values`.
///
/// Reads each element as a JSON integer. If any element is not an integer the
/// request is rejected with `422`. Otherwise the count of distinct integers is
/// returned as `result`. The empty list yields `0`.
#[utoipa::path(
    post,
    path = "/",
    request_body = EvalRequest,
    responses(
        (status = 200, body = CardinalityResponse),
        (status = 422, description = "an element is not a valid integer")
    )
)]
pub async fn evaluate(Json(req): Json<EvalRequest>) -> Response {
    match cardinality(&req.values) {
        Some(result) => (
            StatusCode::OK,
            Json(json!({ "values": req.values, "result": result })),
        )
            .into_response(),
        None => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "error": "values must be integers" })),
        )
            .into_response(),
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(index, evaluate),
    components(schemas(Info, EvalRequest, CardinalityResponse))
)]
pub struct ApiDoc;

/// Serve OpenAPI document
pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_documents_routes() {
        let doc = ApiDoc::openapi();
        let root = doc.paths.paths.get("/").expect("path / present");
        assert!(root.get.is_some(), "GET / documented");
        assert!(root.post.is_some(), "POST / documented");
    }

    #[test]
    fn index_reports_identity() {
        // Identity constants are the public contract of this leaf service.
        assert_eq!(SERVICE, "srvcs-cardinality");
        assert_eq!(CONCERN, "sets: number of distinct elements");
        assert!(DEPENDS_ON.is_empty());
    }

    #[test]
    fn counts_distinct_elements() {
        assert_eq!(
            cardinality(&[json!(1), json!(2), json!(2), json!(3)]),
            Some(3)
        );
        assert_eq!(cardinality(&[json!(1)]), Some(1));
        assert_eq!(cardinality(&[]), Some(0));
    }

    #[test]
    fn order_does_not_matter() {
        assert_eq!(cardinality(&[json!(3), json!(1), json!(2)]), Some(3));
        assert_eq!(cardinality(&[json!(2), json!(3), json!(1)]), Some(3));
    }

    #[test]
    fn all_identical_collapses_to_one() {
        assert_eq!(cardinality(&[json!(4), json!(4), json!(4)]), Some(1));
    }

    #[test]
    fn handles_negatives_and_duplicates() {
        assert_eq!(
            cardinality(&[json!(0), json!(-5), json!(3), json!(-5), json!(0)]),
            Some(3)
        );
    }

    #[test]
    fn non_integer_element_is_rejected() {
        for bad in [
            json!("1"),
            json!(1.5),
            json!(true),
            json!(null),
            json!([1]),
            json!({ "v": 1 }),
        ] {
            assert_eq!(
                cardinality(&[json!(1), bad.clone()]),
                None,
                "{bad} should be rejected"
            );
        }
    }

    #[tokio::test]
    async fn evaluate_returns_200_with_result() {
        let resp = evaluate(Json(EvalRequest {
            values: vec![json!(1), json!(2), json!(2), json!(3)],
        }))
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn evaluate_returns_422_for_non_integer() {
        let resp = evaluate(Json(EvalRequest {
            values: vec![json!(1), json!(1.5)],
        }))
        .await;
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn index_reports_identity_over_http() {
        let Json(info) = index().await;
        assert_eq!(info.service, "srvcs-cardinality");
        assert!(info.depends_on.is_empty());
    }
}
