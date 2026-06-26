use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct HealthStatus {
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceVerdict {
    Consistent,
    Inconsistent,
    InsufficientData,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaseType {
    WrongTransfer,
    PaymentFailed,
    RefundRequest,
    DuplicatePayment,
    MerchantSettlementDelay,
    AgentCashInIssue,
    PhishingOrSocialEngineering,
    Other,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Department {
    CustomerSupport,
    DisputeResolution,
    PaymentsOps,
    MerchantOperations,
    AgentOperations,
    FraudRisk,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub transaction_id: String,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub amount: f64,
    pub counterparty: String,
    pub status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TicketAnalysisRequest {
    pub ticket_id: String,
    pub complaint: String,
    pub language: Option<String>,
    pub channel: Option<String>,
    pub user_type: Option<String>,
    pub campaign_context: Option<String>,
    pub transaction_history: Option<Vec<Transaction>>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TicketAnalysisResponse {
    pub ticket_id: String,
    pub relevant_transaction_id: Option<String>,
    pub evidence_verdict: EvidenceVerdict,
    pub case_type: CaseType,
    pub severity: Severity,
    pub department: Department,
    pub agent_summary: String,
    pub recommended_next_action: String,
    pub customer_reply: String,
    pub human_review_required: bool,
    pub confidence: Option<f64>,
    pub reason_codes: Option<Vec<String>>,
}
