use axum::{
    extract::{State, Query},
    Json,
    response::IntoResponse,
};
use crate::models::{
    HealthStatus, TicketAnalysisRequest,
    StoredTicket, PaginatedTicketsResponse, PaginationInfo,
};

pub async fn health_check() -> impl IntoResponse {
    Json(HealthStatus {
        status: "ok".to_string(),
    })
}

pub async fn analyze_ticket(
    State(state): State<crate::AppState>,
    Json(payload): Json<TicketAnalysisRequest>,
) -> impl IntoResponse {
    let response = crate::investigator::run_investigation(&payload).await;

    // Persist to Postgres database if available
    if let Some(ref pool) = state.db_pool {
        let req_clone = payload.clone();
        let res_clone = response.clone();
        let pool_clone = pool.clone();
        
        let reason_codes_json = res_clone.reason_codes.as_ref().map(|rc| {
            serde_json::to_value(rc).unwrap_or(serde_json::Value::Null)
        });
        
        let evidence_verdict_str = serde_json::to_value(&res_clone.evidence_verdict)
            .unwrap_or_default()
            .as_str()
            .unwrap_or("")
            .to_string();
        let case_type_str = serde_json::to_value(&res_clone.case_type)
            .unwrap_or_default()
            .as_str()
            .unwrap_or("")
            .to_string();
        let severity_str = serde_json::to_value(&res_clone.severity)
            .unwrap_or_default()
            .as_str()
            .unwrap_or("")
            .to_string();
        let department_str = serde_json::to_value(&res_clone.department)
            .unwrap_or_default()
            .as_str()
            .unwrap_or("")
            .to_string();

        let insert_res = sqlx::query(
            "INSERT INTO analyzed_tickets (
                ticket_id, complaint, language, channel, user_type, campaign_context,
                relevant_transaction_id, evidence_verdict, case_type, severity, department,
                agent_summary, recommended_next_action, customer_reply, human_review_required,
                confidence, reason_codes
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            ON CONFLICT (ticket_id) DO UPDATE SET
                complaint = EXCLUDED.complaint,
                language = EXCLUDED.language,
                channel = EXCLUDED.channel,
                user_type = EXCLUDED.user_type,
                campaign_context = EXCLUDED.campaign_context,
                relevant_transaction_id = EXCLUDED.relevant_transaction_id,
                evidence_verdict = EXCLUDED.evidence_verdict,
                case_type = EXCLUDED.case_type,
                severity = EXCLUDED.severity,
                department = EXCLUDED.department,
                agent_summary = EXCLUDED.agent_summary,
                recommended_next_action = EXCLUDED.recommended_next_action,
                customer_reply = EXCLUDED.customer_reply,
                human_review_required = EXCLUDED.human_review_required,
                confidence = EXCLUDED.confidence,
                reason_codes = EXCLUDED.reason_codes"
        )
        .bind(&req_clone.ticket_id)
        .bind(&req_clone.complaint)
        .bind(req_clone.language.as_ref())
        .bind(req_clone.channel.as_ref())
        .bind(req_clone.user_type.as_ref())
        .bind(req_clone.campaign_context.as_ref())
        .bind(res_clone.relevant_transaction_id.as_ref())
        .bind(evidence_verdict_str)
        .bind(case_type_str)
        .bind(severity_str)
        .bind(department_str)
        .bind(&res_clone.agent_summary)
        .bind(&res_clone.recommended_next_action)
        .bind(&res_clone.customer_reply)
        .bind(res_clone.human_review_required)
        .bind(res_clone.confidence)
        .bind(reason_codes_json)
        .execute(&pool_clone)
        .await;

        if let Err(e) = insert_res {
            tracing::error!("Failed to save ticket {} to database: {e}", req_clone.ticket_id);
        }
    }

    Json(response)
}

#[derive(serde::Deserialize)]
pub struct ListTicketsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_tickets(
    State(state): State<crate::AppState>,
    Query(params): Query<ListTicketsQuery>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);

    if let Some(ref pool) = state.db_pool {
        let fetch_res = sqlx::query_as::<_, StoredTicket>(
            "SELECT * FROM analyzed_tickets ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await;

        let count_res: Result<(i64,), sqlx::Error> = sqlx::query_as(
            "SELECT COUNT(*) FROM analyzed_tickets"
        )
        .fetch_one(pool)
        .await;

        match (fetch_res, count_res) {
            (Ok(tickets), Ok((total,))) => {
                Json(PaginatedTicketsResponse {
                    tickets,
                    pagination: PaginationInfo {
                        limit,
                        offset,
                        total,
                    },
                }).into_response()
            }
            (Err(e), _) | (_, Err(e)) => {
                tracing::error!("Database query failed in list_tickets: {e}");
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Database query failed" })),
                ).into_response()
            }
        }
    } else {
        (
            axum::http::StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({ "error": "Database not configured" })),
        ).into_response()
    }
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

    #[tokio::test]
    async fn test_db_persistence_and_pagination() {
        use crate::config::AppConfig;
        use crate::AppState;
        use crate::models::{TicketAnalysisRequest, PaginatedTicketsResponse};
        use axum::extract::State;
        use axum::Json;

        let config = AppConfig::load().await;
        if let Some(pool) = config.db_pool {
            sqlx::query("TRUNCATE TABLE analyzed_tickets").execute(&pool).await.unwrap();

            let state = AppState {
                db_pool: Some(pool.clone()),
            };

            for i in 1..=3 {
                let req = TicketAnalysisRequest {
                    ticket_id: format!("TKT-DB-TEST-00{}", i),
                    complaint: format!("My issue description {}", i),
                    language: Some("en".to_string()),
                    channel: Some("in_app_chat".to_string()),
                    user_type: Some("customer".to_string()),
                    campaign_context: Some("test_campaign".to_string()),
                    transaction_history: None,
                    metadata: None,
                };
                
                let _ = analyze_ticket(State(state.clone()), Json(req)).await;
            }

            let query_1 = ListTicketsQuery {
                limit: Some(2),
                offset: Some(0),
            };
            let resp_1 = list_tickets(State(state.clone()), Query(query_1)).await.into_response();
            assert_eq!(resp_1.status(), axum::http::StatusCode::OK);
            
            let body_bytes_1 = axum::body::to_bytes(resp_1.into_body(), usize::MAX).await.unwrap();
            let page_1: PaginatedTicketsResponse = serde_json::from_slice(&body_bytes_1).unwrap();
            assert_eq!(page_1.tickets.len(), 2);
            assert_eq!(page_1.pagination.total, 3);
            assert_eq!(page_1.pagination.limit, 2);
            assert_eq!(page_1.pagination.offset, 0);

            let query_2 = ListTicketsQuery {
                limit: Some(2),
                offset: Some(2),
            };
            let resp_2 = list_tickets(State(state.clone()), Query(query_2)).await.into_response();
            assert_eq!(resp_2.status(), axum::http::StatusCode::OK);
            
            let body_bytes_2 = axum::body::to_bytes(resp_2.into_body(), usize::MAX).await.unwrap();
            let page_2: PaginatedTicketsResponse = serde_json::from_slice(&body_bytes_2).unwrap();
            assert_eq!(page_2.tickets.len(), 1);
            assert_eq!(page_2.pagination.total, 3);
            assert_eq!(page_2.pagination.limit, 2);
            assert_eq!(page_2.pagination.offset, 2);
        } else {
            println!("Skipping db integration test: DATABASE_URL not set or connection failed");
        }
    }
}
