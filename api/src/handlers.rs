use axum::{Json, response::IntoResponse};
use crate::models::{HealthStatus, TicketAnalysisRequest};

pub async fn health_check() -> impl IntoResponse {
    Json(HealthStatus {
        status: "ok".to_string(),
    })
}

pub async fn analyze_ticket(
    Json(payload): Json<TicketAnalysisRequest>,
) -> impl IntoResponse {
    let response = crate::investigator::run_investigation(&payload).await;
    Json(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("\"status\":\"ok\""));
    }

    #[tokio::test]
    async fn test_preli_sample_cases() {
        use std::fs;
        use crate::investigator::run_investigation;
        use crate::models::TicketAnalysisResponse;

        let file_content = fs::read_to_string("../SUST_Preli_Sample_Cases.json")
            .expect("Failed to read sample cases file");
        let cases_json: serde_json::Value = serde_json::from_str(&file_content)
            .expect("Failed to parse sample cases JSON");

        let cases = cases_json["cases"].as_array().expect("cases should be an array");

        for case in cases {
            let id = case["id"].as_str().unwrap();
            let input_val = case["input"].clone();
            let expected_val = case["expected_output"].clone();

            let req: TicketAnalysisRequest = serde_json::from_value(input_val).unwrap();
            let resp = run_investigation(&req).await;

            let expected_resp: TicketAnalysisResponse = serde_json::from_value(expected_val).unwrap();

            assert_eq!(resp.ticket_id, expected_resp.ticket_id, "Mismatch in case {}", id);
            assert_eq!(resp.relevant_transaction_id, expected_resp.relevant_transaction_id, "Mismatch in case {}", id);
            assert_eq!(resp.evidence_verdict, expected_resp.evidence_verdict, "Mismatch in case {}", id);
            assert_eq!(resp.case_type, expected_resp.case_type, "Mismatch in case {}", id);
            assert_eq!(resp.severity, expected_resp.severity, "Mismatch in case {}", id);
            assert_eq!(resp.department, expected_resp.department, "Mismatch in case {}", id);
            assert_eq!(resp.human_review_required, expected_resp.human_review_required, "Mismatch in case {}", id);
        }
    }
}
