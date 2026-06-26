use axum::{Json, response::IntoResponse};
use crate::models::{
    HealthStatus, TicketAnalysisRequest, TicketAnalysisResponse, EvidenceVerdict, CaseType, Severity, Department
};

pub async fn health_check() -> impl IntoResponse {
    Json(HealthStatus {
        status: "ok".to_string(),
    })
}

pub async fn analyze_ticket(
    Json(payload): Json<TicketAnalysisRequest>,
) -> impl IntoResponse {
    // Stub response conforming to output schema
    let response = TicketAnalysisResponse {
        ticket_id: payload.ticket_id,
        relevant_transaction_id: None,
        evidence_verdict: EvidenceVerdict::InsufficientData,
        case_type: CaseType::Other,
        severity: Severity::Low,
        department: Department::CustomerSupport,
        agent_summary: "Stub summary".to_string(),
        recommended_next_action: "Stub action".to_string(),
        customer_reply: "Stub reply".to_string(),
        human_review_required: false,
        confidence: Some(1.0),
        reason_codes: Some(vec!["stub".to_string()]),
    };
    
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
    async fn test_analyze_ticket_stub() {
        let req = TicketAnalysisRequest {
            ticket_id: "TKT-TEST".to_string(),
            complaint: "Testing ticket handler".to_string(),
            language: None,
            channel: None,
            user_type: None,
            campaign_context: None,
            transaction_history: None,
            metadata: None,
        };
        let response = analyze_ticket(Json(req)).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let resp_struct: TicketAnalysisResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(resp_struct.ticket_id, "TKT-TEST");
        assert_eq!(resp_struct.evidence_verdict, EvidenceVerdict::InsufficientData);
    }
}
