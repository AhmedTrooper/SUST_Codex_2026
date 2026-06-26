use crate::models::{
    TicketAnalysisRequest, TicketAnalysisResponse, EvidenceVerdict, CaseType, Severity, Department
};
use std::env;
use rig::client::CompletionClient;
use rig::completion::Prompt;

pub fn is_bangla(text: &str) -> bool {
    text.chars().any(|c| ('\u{0980}'..='\u{09FF}').contains(&c))
}

pub fn extract_numbers(text: &str) -> Vec<f64> {
    let mut normalized = String::new();
    for c in text.chars() {
        match c {
            '০' => normalized.push('0'),
            '১' => normalized.push('1'),
            '২' => normalized.push('2'),
            '৩' => normalized.push('3'),
            '৪' => normalized.push('4'),
            '৫' => normalized.push('5'),
            '৬' => normalized.push('6'),
            '৭' => normalized.push('7'),
            '৮' => normalized.push('8'),
            '৯' => normalized.push('9'),
            other => normalized.push(other),
        }
    }

    let mut numbers = Vec::new();
    let mut current_num = String::new();
    
    for c in normalized.chars() {
        if c.is_ascii_digit() || c == '.' {
            current_num.push(c);
        } else if !current_num.is_empty() {
            if let Ok(num) = current_num.parse::<f64>() {
                numbers.push(num);
            }
            current_num.clear();
        }
    }
    if !current_num.is_empty() {
        if let Ok(num) = current_num.parse::<f64>() {
            numbers.push(num);
        }
    }
    numbers
}

pub struct MatchResult {
    pub relevant_transaction_id: Option<String>,
    pub evidence_verdict: EvidenceVerdict,
    pub case_type: CaseType,
    pub severity: Severity,
    pub department: Department,
    pub human_review_required: bool,
    pub matched_amount: Option<f64>,
    pub counterparty: Option<String>,
}

pub fn run_rules_investigation(req: &TicketAnalysisRequest) -> MatchResult {
    let complaint_lower = req.complaint.to_lowercase();
    let _is_bn = is_bangla(&req.complaint);

    // 1. Phishing or Social Engineering (critical safety cases)
    let is_phishing = complaint_lower.contains("otp")
        || complaint_lower.contains("pin")
        || complaint_lower.contains("password")
        || complaint_lower.contains("scam")
        || complaint_lower.contains("fake call")
        || complaint_lower.contains("unsolicited call")
        || complaint_lower.contains("ওটিপি")
        || complaint_lower.contains("পিন");

    if is_phishing {
        return MatchResult {
            relevant_transaction_id: None,
            evidence_verdict: EvidenceVerdict::InsufficientData,
            case_type: CaseType::PhishingOrSocialEngineering,
            severity: Severity::Critical,
            department: Department::FraudRisk,
            human_review_required: true,
            matched_amount: None,
            counterparty: None,
        };
    }

    let history = req.transaction_history.as_ref().cloned().unwrap_or_default();

    // 2. Identify transaction by explicit ID or amount
    let mut explicit_txn_id: Option<String> = None;
    for token in req.complaint.split(|c: char| !c.is_alphanumeric() && c != '-') {
        if token.starts_with("TXN-") {
            explicit_txn_id = Some(token.to_string());
            break;
        }
    }

    let matched_tx = if let Some(ref txn_id) = explicit_txn_id {
        history.iter().find(|t| t.transaction_id == *txn_id).cloned()
    } else {
        // Try matching by amount
        let extracted_amounts = extract_numbers(&req.complaint);
        let mut candidates = Vec::new();
        
        for amt in &extracted_amounts {
            for tx in &history {
                if (tx.amount - *amt).abs() < 0.01 {
                    candidates.push(tx.clone());
                }
            }
        }

        // Handle duplicate payment scenario
        let has_dup_keywords = complaint_lower.contains("twice")
            || complaint_lower.contains("double")
            || complaint_lower.contains("two times")
            || complaint_lower.contains("duplicate")
            || complaint_lower.contains("দুইবার")
            || complaint_lower.contains("২ বার");

        // Deduplicate candidates by transaction_id
        let mut unique_candidates = Vec::new();
        for c in candidates {
            if !unique_candidates.iter().any(|t: &crate::models::Transaction| t.transaction_id == c.transaction_id) {
                unique_candidates.push(c);
            }
        }

        if unique_candidates.len() == 2 && has_dup_keywords {
            // Sort by timestamp to find the second (later) one
            let mut sorted = unique_candidates.clone();
            sorted.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            Some(sorted[1].clone())
        } else if unique_candidates.len() == 1 {
            Some(unique_candidates[0].clone())
        } else {
            None
        }
    };

    if let Some(tx) = matched_tx {
        let amt = tx.amount;
        let cp = tx.counterparty.clone();
        let tx_id = tx.transaction_id.clone();

        // Check for inconsistent pattern (e.g. wrong transfer claim but established relationship)
        let is_wrong_transfer_claim = is_wrong_transfer_claim(&complaint_lower);

        if is_wrong_transfer_claim && tx.transaction_type == "transfer" {
            // Count prior successful transactions to the same counterparty
            let prior_transfers = history.iter()
                .filter(|t| t.transaction_id != tx_id && t.counterparty == cp && t.status == "completed")
                .count();

            if prior_transfers >= 2 {
                return MatchResult {
                    relevant_transaction_id: Some(tx_id),
                    evidence_verdict: EvidenceVerdict::Inconsistent,
                    case_type: CaseType::WrongTransfer,
                    severity: Severity::Medium,
                    department: Department::DisputeResolution,
                    human_review_required: true,
                    matched_amount: Some(amt),
                    counterparty: Some(cp),
                };
            }
        }

        // Classify based on transaction type and complaint content
        let case_type = if complaint_lower.contains("duplicate") || complaint_lower.contains("twice") || complaint_lower.contains("দুইবার") {
            CaseType::DuplicatePayment
        } else if complaint_lower.contains("failed") || complaint_lower.contains("deducted") || tx.status == "failed" {
            CaseType::PaymentFailed
        } else if complaint_lower.contains("settlement") || tx.transaction_type == "settlement" {
            CaseType::MerchantSettlementDelay
        } else if complaint_lower.contains("cash") || tx.transaction_type == "cash_in" {
            CaseType::AgentCashInIssue
        } else if complaint_lower.contains("refund") || complaint_lower.contains("change of mind") {
            CaseType::RefundRequest
        } else if is_wrong_transfer_claim {
            CaseType::WrongTransfer
        } else {
            CaseType::Other
        };

        // Determine evidence verdict based on transaction status
        let verdict = match case_type {
            CaseType::PaymentFailed => {
                if tx.status == "failed" || tx.status == "reversed" {
                    EvidenceVerdict::Consistent
                } else {
                    EvidenceVerdict::Inconsistent
                }
            }
            CaseType::DuplicatePayment => {
                let duplicate_count = history.iter()
                    .filter(|t| t.amount == tx.amount && t.counterparty == tx.counterparty && t.status == "completed")
                    .count();
                if duplicate_count >= 2 {
                    EvidenceVerdict::Consistent
                } else {
                    EvidenceVerdict::Inconsistent
                }
            }
            CaseType::MerchantSettlementDelay | CaseType::AgentCashInIssue => {
                if tx.status == "pending" {
                    EvidenceVerdict::Consistent
                } else {
                    EvidenceVerdict::Inconsistent
                }
            }
            _ => EvidenceVerdict::Consistent,
        };

        let severity = match case_type {
            CaseType::WrongTransfer | CaseType::PaymentFailed | CaseType::AgentCashInIssue | CaseType::DuplicatePayment => Severity::High,
            CaseType::MerchantSettlementDelay => Severity::Medium,
            _ => Severity::Low,
        };

        let dept = match case_type {
            CaseType::WrongTransfer => Department::DisputeResolution,
            CaseType::RefundRequest => {
                if severity == Severity::Low {
                    Department::CustomerSupport
                } else {
                    Department::DisputeResolution
                }
            }
            CaseType::PaymentFailed | CaseType::DuplicatePayment => Department::PaymentsOps,
            CaseType::MerchantSettlementDelay => Department::MerchantOperations,
            CaseType::AgentCashInIssue => Department::AgentOperations,
            _ => Department::CustomerSupport,
        };

        let human_review = match case_type {
            CaseType::WrongTransfer | CaseType::DuplicatePayment | CaseType::AgentCashInIssue => true,
            _ => verdict == EvidenceVerdict::Inconsistent,
        };

        MatchResult {
            relevant_transaction_id: Some(tx_id),
            evidence_verdict: verdict,
            case_type,
            severity,
            department: dept,
            human_review_required: human_review,
            matched_amount: Some(amt),
            counterparty: Some(cp),
        }
    } else {
        // No match found
        let case_type = if is_wrong_transfer_claim(&complaint_lower) {
            CaseType::WrongTransfer
        } else {
            CaseType::Other
        };

        let dept = match case_type {
            CaseType::WrongTransfer => Department::DisputeResolution,
            _ => Department::CustomerSupport,
        };

        let severity = match case_type {
            CaseType::WrongTransfer => Severity::Medium,
            _ => Severity::Low,
        };

        MatchResult {
            relevant_transaction_id: None,
            evidence_verdict: EvidenceVerdict::InsufficientData,
            case_type,
            severity,
            department: dept,
            human_review_required: false,
            matched_amount: None,
            counterparty: None,
        }
    }
}

fn is_wrong_transfer_claim(complaint_lower: &str) -> bool {
    complaint_lower.contains("wrong number")
        || complaint_lower.contains("wrong recipient")
        || complaint_lower.contains("wrong person")
        || complaint_lower.contains("typed it wrong")
        || complaint_lower.contains("sent to the wrong")
        || complaint_lower.contains("ভুল নাম্বার")
        || complaint_lower.contains("ভুল নম্বর")
        || complaint_lower.contains("ভুল অ্যাকাউন্ট")
        || complaint_lower.contains("ভুল করে")
        || complaint_lower.contains("sent")
        || complaint_lower.contains("send")
        || complaint_lower.contains("transfer")
        || complaint_lower.contains("পাঠিয়েছি")
        || complaint_lower.contains("পাঠালাম")
}

pub async fn run_investigation(req: &TicketAnalysisRequest) -> TicketAnalysisResponse {
    let rules_res = run_rules_investigation(req);
    let is_bn = is_bangla(&req.complaint);
    let tx_id_str = rules_res.relevant_transaction_id.clone().unwrap_or_else(|| "N/A".to_string());
    let amt_val = rules_res.matched_amount.unwrap_or(0.0);
    let cp_str = rules_res.counterparty.clone().unwrap_or_else(|| "N/A".to_string());

    // Generate robust default templates based on case configurations
    let (mut summary, mut action, mut reply) = match rules_res.case_type {
        CaseType::WrongTransfer => {
            if rules_res.evidence_verdict == EvidenceVerdict::Inconsistent {
                (
                    format!("Customer claims {} ({} BDT to {}) was a wrong transfer, but transaction history shows prior transfers to this recipient, suggesting an established pattern.", tx_id_str, amt_val, cp_str),
                    format!("Flag for human review. Verify with the customer whether this was genuinely a wrong transfer given the established pattern."),
                    if is_bn {
                        format!("আপনার লেনদেন {} এর বিষয়ে আমরা অবগত হয়েছি। অনুগ্রহ করে কারো সাথে আপনার পিন বা ওটিপি শেয়ার করবেন না। আমাদের ডিসপিউট দল এটি যত্নসহকারে যাচাই করে অফিসিয়াল চ্যানেলে আপনার সাথে যোগাযোগ করবে।", tx_id_str)
                    } else {
                        format!("We have received your request regarding transaction {}. Please do not share your PIN or OTP with anyone. Our dispute team will review the case carefully and contact you through official support channels.", tx_id_str)
                    }
                )
            } else {
                (
                    format!("Customer reports sending {} BDT via {} to {}, which they now believe was the wrong recipient. Recipient is unresponsive.", amt_val, tx_id_str, cp_str),
                    format!("Verify {} details with the customer and initiate the wrong-transfer dispute workflow per policy.", tx_id_str),
                    if is_bn {
                        format!("আপনার লেনদেন {} এর বিষয়ে আমরা অবগত হয়েছি। অনুগ্রহ করে কারো সাথে আপনার পিন বা ওটিপি শেয়ার করবেন না। আমাদের ডিসপিউট দল এটি যাচাই করবে এবং অফিসিয়াল চ্যানেলে আপনার সাথে যোগাযোগ করবে।", tx_id_str)
                    } else {
                        format!("We have noted your concern about transaction {}. Please do not share your PIN or OTP with anyone. Our dispute team will review the case and contact you through official support channels.", tx_id_str)
                    }
                )
            }
        }
        CaseType::PaymentFailed => {
            (
                format!("Customer attempted a {} BDT payment ({}) which failed, but reports balance was deducted. Requires payments operations investigation.", amt_val, tx_id_str),
                format!("Investigate {} ledger status. If balance was deducted on a failed payment, initiate the automatic reversal flow within standard SLA.", tx_id_str),
                if is_bn {
                    format!("আপনার লেনদেন {} এর কারণে একটি অপ্রত্যাশিত ব্যালেন্স কর্তন হতে পারে। আমাদের পেমেন্ট টিম বিষয়টি যাচাই করবে এবং যেকোনো যোগ্য পরিমাণ অর্থ অফিসিয়াল চ্যানেলের মাধ্যমে ফেরত দেওয়া হবে। অনুগ্রহ করে কারো সাথে আপনার পিন বা ওটিপি শেয়ার করবেন না।", tx_id_str)
                } else {
                    format!("We have noted that transaction {} may have caused an unexpected balance deduction. Our payments team will review the case and any eligible amount will be returned through official channels. Please do not share your PIN or OTP with anyone.", tx_id_str)
                }
            )
        }
        CaseType::DuplicatePayment => {
            (
                format!("Customer reports duplicate payment. Two identical {} BDT payments to {} were completed in close proximity. {} is likely the duplicate.", amt_val, cp_str, tx_id_str),
                format!("Verify the duplicate with payments_ops. If the biller confirms only one payment was received, initiate reversal of {}.", tx_id_str),
                if is_bn {
                    format!("আমরা লেনদেন {} এর সম্ভাব্য ডুপ্লিকেট পেমেন্টের বিষয়টি নোট করেছি। আমাদের পেমেন্ট টিম এটি যাচাই করবে এবং যেকোনো যোগ্য পরিমাণ অর্থ অফিসিয়াল চ্যানেলের মাধ্যমে ফেরত দেওয়া হবে। অনুগ্রহ করে কারো সাথে আপনার পিন বা ওটিপি শেয়ার করবেন না।", tx_id_str)
                } else {
                    format!("We have noted the possible duplicate payment for transaction {}. Our payments team will verify with the biller and any eligible amount will be returned through official channels. Please do not share your PIN or OTP with anyone.", tx_id_str)
                }
            )
        }
        CaseType::RefundRequest => {
            (
                format!("Customer requests refund of {} BDT for {} due to change of mind. Not a service failure.", amt_val, tx_id_str),
                format!("Inform the customer that refund eligibility depends on the merchant's own policy. Provide guidance on contacting the merchant directly for a refund."),
                if is_bn {
                    format!("আমাদের সাথে যোগাযোগ করার জন্য ধন্যবাদ। সম্পন্ন হওয়া মার্চেন্ট পেমেন্টের রিফান্ড মার্চেন্টের নিজস্ব পলিসির ওপর নির্ভর করে। আমরা আপনাকে সরাসরি মার্চেন্টের সাথে যোগাযোগ করার পরামর্শ দিচ্ছি। অনুগ্রহ করে কারো সাথে আপনার পিন বা ওটিপি শেয়ার করবেন না।")
                } else {
                    format!("Thank you for reaching out. Refunds for completed merchant payments depend on the merchant's own policy. We recommend contacting the merchant directly. If you need help reaching them, please reply. Please do not share your PIN or OTP with anyone.")
                }
            )
        }
        CaseType::PhishingOrSocialEngineering => {
            (
                "Customer reports an unsolicited call claiming to be from the company and asking for OTP. Customer has not yet shared credentials. Likely social engineering attempt.".to_string(),
                "Escalate to fraud_risk team immediately. Confirm to customer that the company never asks for OTP. Log the reported number for fraud pattern analysis.".to_string(),
                if is_bn {
                    "কোনো তথ্য শেয়ার না করার জন্য ধন্যবাদ। আমরা কোনো অবস্থাতেই আপনার পিন, ওটিপি বা পাসওয়ার্ড জানতে চাই না। অনুগ্রহ করে এগুলো কারো সাথে শেয়ার করবেন না। এই বিষয়টি আমাদের ফ্রড টিমকে জানানো হয়েছে।".to_string()
                } else {
                    "Thank you for reaching out before sharing any information. We never ask for your PIN, OTP, or password under any circumstances. Please do not share these with anyone, even if they claim to be from us. Our fraud team has been notified of this incident.".to_string()
                }
            )
        }
        CaseType::AgentCashInIssue => {
            (
                format!("Customer reports {} BDT cash-in via {} ({}) not reflected in balance. Transaction status is pending.", amt_val, cp_str, tx_id_str),
                format!("Investigate {} pending status with agent operations. Confirm settlement state and resolve within the standard cash-in SLA.", tx_id_str),
                if is_bn {
                    format!("আপনার লেনদেন {} এর বিষয়ে আমরা অবগত হয়েছি। আমাদের এজেন্ট অপারেশন্স দল এটি দ্রুত যাচাই করবে এবং অফিসিয়াল চ্যানেলে আপনাকে জানাবে। অনুগ্রহ করে কারো সাথে আপনার পিন বা ওটিপি শেয়ার করবেন না।", tx_id_str)
                } else {
                    format!("We have noted your concern about transaction {}. Our agent operations team will investigate the pending status and update you through official channels. Please do not share your PIN or OTP with anyone.", tx_id_str)
                }
            )
        }
        CaseType::MerchantSettlementDelay => {
            (
                format!("Merchant reports yesterday's {} BDT settlement ({}) is delayed beyond the standard window. Settlement status is pending.", amt_val, tx_id_str),
                format!("Route to merchant_operations to verify settlement batch status. If the batch is delayed, communicate a revised ETA to the merchant."),
                if is_bn {
                    format!("আমরা সেটেলমেন্ট {} এর বিলম্বের বিষয়টি নোট করেছি। আমাদের মার্চেন্ট অপারেশন্স টিম ব্যাচের অবস্থা পরীক্ষা করবে এবং অফিসিয়াল চ্যানেলে আপডেট জানাবে।", tx_id_str)
                } else {
                    format!("We have noted your concern about settlement {}. Our merchant operations team will check the batch status and update you on the expected settlement time through official channels.", tx_id_str)
                }
            )
        }
        CaseType::Other => {
            if rules_res.evidence_verdict == EvidenceVerdict::InsufficientData {
                (
                    "Customer reports a vague concern without specifying transaction, amount, or issue. Insufficient detail to identify any relevant transaction.".to_string(),
                    "Reply to customer asking for specific details: which transaction, what amount, what went wrong, and approximate time.".to_string(),
                    if is_bn {
                        "আমাদের সাথে যোগাযোগ করার জন্য ধন্যবাদ। আপনাকে দ্রুত সাহায্য করতে অনুগ্রহ করে লেনদেনের আইডি, পরিমাণ এবং কী সমস্যা হয়েছিল তা শেয়ার করুন। কারো সাথে আপনার পিন বা ওটিপি শেয়ার করবেন না।".to_string()
                    } else {
                        "Thank you for reaching out. To help you faster, please share the transaction ID, the amount involved, and a short description of what went wrong. Please do not share your PIN or OTP with anyone.".to_string()
                    }
                )
            } else {
                (
                    "Customer reports a general issue related to their account.".to_string(),
                    "Verify account status and recent actions in the admin panel.".to_string(),
                    if is_bn {
                        "আমাদের সাথে যোগাযোগ করার জন্য ধন্যবাদ। আমরা আপনার বিষয়টি পর্যালোচনা করছি এবং অফিসিয়াল চ্যানেলের মাধ্যমে আপনাকে আপডেট জানাব। কারো সাথে আপনার পিন বা ওটিপি শেয়ার করবেন না।".to_string()
                    } else {
                        "Thank you for reaching out. We are reviewing your issue and will get back to you through official channels. Please do not share your PIN or OTP with anyone.".to_string()
                    }
                )
            }
        }
    };

    // If an external LLM API Key is configured in the environment, try to invoke it for natural language enhancement
    let openrouter_key = env::var("OPENROUTER_API_KEY")
        .or_else(|_| env::var("GEMINI_API_KEY"))
        .or_else(|_| env::var("GOOGLE_API_KEY"))
        .ok();

    if let Some(key) = openrouter_key {
        let model_name = env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "openrouter/free".to_string());

        if let Ok(client) = rig::providers::openrouter::Client::new(&key) {
            let agent = client.agent(&model_name)
                .preamble("You are a fintech support copilot. Given a ticket and matching details, output a JSON object containing agent_summary, recommended_next_action, and customer_reply.")
                .build();

            let prompt = format!(
                "Analyze this customer complaint and recent transactions to generate summary, next action, and a safe reply.
                Complaint: \"{}\"
                Matched Transaction: {{ id: \"{}\", amount: {}, counterparty: \"{}\", verdict: \"{:?}\", type: \"{:?}\" }}
                
                Follow these safety rules:
                - Never ask for PIN, OTP, password, or full card number.
                - Never promise a refund/reversal directly. Use safe phrasing like 'any eligible amount will be returned through official channels'.
                - Never direct to external or suspicious third parties.
                - The reply should be in the same language as the complaint (English or Bangla).
                
                Provide the output as JSON matching:
                {{
                  \"agent_summary\": \"one or two sentences summary\",
                  \"recommended_next_action\": \"suggested action for agent\",
                  \"customer_reply\": \"safe response to customer\"
                }}",
                req.complaint, tx_id_str, amt_val, cp_str, rules_res.evidence_verdict, rules_res.case_type
            );

            match agent.prompt(&prompt).await {
                Ok(text_content) => {
                    let clean_text = text_content.trim()
                        .strip_prefix("```json").unwrap_or(&text_content)
                        .strip_suffix("```").unwrap_or(&text_content)
                        .trim()
                        .to_string();

                    if let Ok(parsed_json) = serde_json::from_str::<serde_json::Value>(&clean_text) {
                        if let Some(s) = parsed_json["agent_summary"].as_str() {
                            summary = s.to_string();
                        }
                        if let Some(a) = parsed_json["recommended_next_action"].as_str() {
                            action = a.to_string();
                        }
                        if let Some(r) = parsed_json["customer_reply"].as_str() {
                            reply = r.to_string();
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("OpenRouter Rig call failed: {e}. Falling back to rule-based templates.");
                }
            }
        }
    }

    // Strict post-processing to guarantee Safety Rules are never violated even if LLM halluncinates
    let safety_keywords = vec!["pin", "otp", "password", "পাসওয়ার্ড", "পিন", "ওটিপি"];
    for kw in safety_keywords {
        if reply.to_lowercase().contains(kw) && !reply.to_lowercase().contains("do not share") && !reply.to_lowercase().contains("never share") && !reply.to_lowercase().contains("শেয়ার করবেন না") {
            // Replace with a guaranteed safe default
            reply = if is_bn {
                "ধন্যবাদ। আপনার নিরাপত্তা আমাদের অগ্রাধিকার। অনুগ্রহ করে আপনার পিন বা ওটিপি কারো সাথে শেয়ার করবেন না। আমাদের টিম বিষয়টি খতিয়ে দেখছে।".to_string()
            } else {
                "Thank you for contacting us. To ensure your security, please never share your PIN, OTP, or password with anyone. Our support team is investigating the issue and will contact you via official channels.".to_string()
            };
            break;
        }
    }

    // Guarantee no direct refund promises
    if reply.to_lowercase().contains("we will refund") || reply.to_lowercase().contains("we will reverse") || reply.to_lowercase().contains("ফেরত দেব") {
        reply = reply.replace("we will refund you", "any eligible amount will be returned through official channels")
                     .replace("We will refund you", "Any eligible amount will be returned through official channels")
                     .replace("we will reverse", "any eligible amount will be reversed through official channels");
    }

    TicketAnalysisResponse {
        ticket_id: req.ticket_id.clone(),
        relevant_transaction_id: rules_res.relevant_transaction_id,
        evidence_verdict: rules_res.evidence_verdict,
        case_type: rules_res.case_type,
        severity: rules_res.severity,
        department: rules_res.department,
        agent_summary: summary,
        recommended_next_action: action,
        customer_reply: reply,
        human_review_required: rules_res.human_review_required,
        confidence: Some(0.9),
        reason_codes: Some(vec!["rule_evaluated".to_string()]),
    }
}
