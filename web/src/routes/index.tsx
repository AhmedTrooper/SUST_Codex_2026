import { createFileRoute } from "@tanstack/react-router";
import * as React from "react";
import { 
  Plus, Trash2, Loader2, Clipboard, Check, RefreshCw, 
  ShieldAlert, User, Layers, Radio, FileText, Database, AlertCircle
} from "lucide-react";
import { Button } from "../components/ui/button";

export const Route = createFileRoute("/")({
  component: Home,
});

interface Transaction {
  transaction_id: string;
  timestamp: string;
  transaction_type: string;
  amount: number;
  counterparty: string;
  status: string;
}

interface StoredTicket {
  ticket_id: string;
  complaint: string;
  language: Option<string>;
  channel: Option<string>;
  user_type: Option<string>;
  campaign_context: Option<string>;
  relevant_transaction_id: Option<String>;
  evidence_verdict: string;
  case_type: string;
  severity: string;
  department: string;
  agent_summary: string;
  recommended_next_action: string;
  customer_reply: string;
  confidence: Option<number>;
  reason_codes: Option<any>;
  created_at: string;
}

type Option<T> = T | null;

const SAMPLES = {
  wrong_transfer: {
    ticket_id: "TKT-001",
    complaint: "I sent 5000 taka to a wrong number around 2pm today. The number was supposed to be 01712345678 but I think I typed it wrong. The person isn't responding to my call. Please help me get my money back.",
    language: "en",
    channel: "in_app_chat",
    user_type: "customer",
    campaign_context: "boishakh_bonanza_day_1",
    transactions: [
      {
        transaction_id: "TXN-9101",
        timestamp: new Date().toISOString(),
        transaction_type: "transfer",
        amount: 5000,
        counterparty: "+8801719876543",
        status: "completed"
      },
      {
        transaction_id: "TXN-9087",
        timestamp: new Date().toISOString(),
        transaction_type: "cash_in",
        amount: 10000,
        counterparty: "AGENT-512",
        status: "completed"
      }
    ]
  },
  failed_payment: {
    ticket_id: "TKT-002",
    complaint: "I tried to pay my bill of 2300 BDT yesterday at the merchant shop but the payment failed. However, my account balance was still deducted. Please refund my money.",
    language: "en",
    channel: "in_app_chat",
    user_type: "customer",
    campaign_context: "boishakh_bonanza_day_1",
    transactions: [
      {
        transaction_id: "TXN-8812",
        timestamp: new Date().toISOString(),
        transaction_type: "payment",
        amount: 2300,
        counterparty: "MERCHANT-89",
        status: "failed"
      }
    ]
  },
  phishing: {
    ticket_id: "TKT-003",
    complaint: "I received a call from someone claiming to be from your customer support. They told me there was a security issue with my account and asked for my PIN and OTP. Is this official?",
    language: "en",
    channel: "in_app_chat",
    user_type: "customer",
    campaign_context: "boishakh_bonanza_day_1",
    transactions: []
  }
};

function Home() {
  const [ticketId, setTicketId] = React.useState("");
  const [complaint, setComplaint] = React.useState("");
  const [language, setLanguage] = React.useState("en");
  const [channel, setChannel] = React.useState("in_app_chat");
  const [userType, setUserType] = React.useState("customer");
  const [campaignContext, setCampaignContext] = React.useState("boishakh_bonanza_day_1");
  const [transactions, setTransactions] = React.useState<Transaction[]>([]);
  
  // UI States
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const [copied, setCopied] = React.useState(false);
  const [result, setResult] = React.useState<any | null>(null);

  // History States
  const [history, setHistory] = React.useState<StoredTicket[]>([]);
  const [historyTotal, setHistoryTotal] = React.useState(0);
  const [historyOffset, setHistoryOffset] = React.useState(0);
  const [isHistoryLoading, setIsHistoryLoading] = React.useState(false);
  const historyLimit = 6;

  const apiBaseUrl = import.meta.env.VITE_API_URL || "http://localhost:8080";

  React.useEffect(() => {
    // Generate initial Ticket ID
    generateRandomId();
    fetchHistory(0);
  }, []);

  const generateRandomId = () => {
    setTicketId(`TKT-${Math.floor(100000 + Math.random() * 900000)}`);
  };

  const fetchHistory = async (offsetVal: number) => {
    setIsHistoryLoading(true);
    try {
      const res = await fetch(`${apiBaseUrl}/tickets?limit=${historyLimit}&offset=${offsetVal}`);
      if (res.ok) {
        const data = await res.json();
        setHistory(data.tickets || []);
        setHistoryTotal(data.pagination?.total || 0);
      } else {
        console.error("Failed to load ticket history");
      }
    } catch (e) {
      console.error("Error fetching tickets history:", e);
    } finally {
      setIsHistoryLoading(false);
    }
  };

  const loadSample = (type: "wrong_transfer" | "failed_payment" | "phishing") => {
    const sample = SAMPLES[type];
    setTicketId(sample.ticket_id);
    setComplaint(sample.complaint);
    setLanguage(sample.language);
    setChannel(sample.channel);
    setUserType(sample.user_type);
    setCampaignContext(sample.campaign_context);
    setTransactions(sample.transactions);
    setError(null);
  };

  const addTransaction = () => {
    const nextId = `TXN-${Math.floor(1000 + Math.random() * 9000)}`;
    setTransactions([
      ...transactions,
      {
        transaction_id: nextId,
        timestamp: new Date().toISOString(),
        transaction_type: "transfer",
        amount: 0,
        counterparty: "",
        status: "completed"
      }
    ]);
  };

  const removeTransaction = (index: number) => {
    setTransactions(transactions.filter((_, i) => i !== index));
  };

  const updateTransaction = (index: number, key: keyof Transaction, val: any) => {
    const updated = [...transactions];
    updated[index] = { ...updated[index], [key]: val };
    setTransactions(updated);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    // 1. Validate Ticket ID
    if (!ticketId.trim()) {
      setError("Ticket ID cannot be empty.");
      return;
    }

    // 2. Validate Complaint Text
    if (!complaint.trim()) {
      setError("Complaint details cannot be empty.");
      return;
    }
    if (complaint.trim().length < 5) {
      setError("Complaint details must be at least 5 characters long.");
      return;
    }

    // 3. Validate Transaction History
    for (let i = 0; i < transactions.length; i++) {
      const t = transactions[i];
      if (!t.transaction_id.trim()) {
        setError(`Transaction #${i + 1} is missing a Transaction ID.`);
        return;
      }
      if (t.amount <= 0 || isNaN(t.amount)) {
        setError(`Transaction #${i + 1} (${t.transaction_id}) amount must be a positive number greater than 0 BDT.`);
        return;
      }
      if (!t.counterparty.trim()) {
        setError(`Transaction #${i + 1} (${t.transaction_id}) is missing a Counterparty ID or Phone Number.`);
        return;
      }
    }

    setError(null);
    setIsLoading(true);

    const payload = {
      ticket_id: ticketId.trim(),
      complaint: complaint.trim(),
      language: language || undefined,
      channel: channel || undefined,
      user_type: userType || undefined,
      campaign_context: campaignContext.trim() || undefined,
      transaction_history: transactions.length > 0 ? transactions.map(t => ({
        transaction_id: t.transaction_id.trim(),
        timestamp: t.timestamp,
        type: t.transaction_type,
        amount: Number(t.amount),
        counterparty: t.counterparty.trim(),
        status: t.status,
      })) : undefined,
    };

    try {
      const res = await fetch(`${apiBaseUrl}/analyze-ticket`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });

      if (res.ok) {
        const data = await res.json();
        setResult(data);
        // Refresh log history and reset offset to see the latest ticket
        setHistoryOffset(0);
        await fetchHistory(0);
      } else {
        const errData = await res.json().catch(() => ({}));
        setError(errData.error || "An error occurred during analysis.");
      }
    } catch (e) {
      setError("Could not connect to the backend API. Please make sure the backend is running.");
    } finally {
      setIsLoading(false);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const loadFromHistory = (ticket: StoredTicket) => {
    setTicketId(ticket.ticket_id);
    setComplaint(ticket.complaint);
    setLanguage(ticket.language || "en");
    setChannel(ticket.channel || "in_app_chat");
    setUserType(ticket.user_type || "customer");
    setCampaignContext(ticket.campaign_context || "");
    setResult(ticket);
    setTransactions([]);
    setError(null);
    window.scrollTo({ top: 0, behavior: "smooth" });
  };

  const getVerdictStyle = (verdict: string) => {
    switch (verdict?.toLowerCase()) {
      case "consistent":
        return "border-emerald-500/20 text-emerald-500 bg-emerald-500/5";
      case "inconsistent":
        return "border-red-500/20 text-red-500 bg-red-500/5";
      default:
        return "border-zinc-500/20 text-zinc-400 bg-zinc-500/5";
    }
  };

  const getSeverityStyle = (severity: string) => {
    switch (severity?.toLowerCase()) {
      case "critical":
        return "border-red-500/40 text-red-400 bg-red-955/20";
      case "high":
        return "border-amber-500/30 text-amber-400 bg-amber-955/10";
      case "medium":
        return "border-zinc-400/30 text-zinc-300 bg-zinc-800/10";
      default:
        return "border-zinc-600/30 text-zinc-400 bg-zinc-900/10";
    }
  };

  return (
    <main className="page-wrap min-h-screen px-4 py-8 md:py-16">
      
      {/* Sleek Hero Header */}
      <section className="mb-12 border-b border-[var(--line)] pb-8 text-left rise-in">
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          <div>
            <span className="font-mono text-xs uppercase tracking-widest text-[var(--sea-ink-soft)] block mb-1">
              SUPPORT OPERATIONS / CO-PILOT INVESTIGATOR
            </span>
            <h1 className="text-3xl md:text-4xl font-extrabold tracking-tight text-[var(--sea-ink)] m-0">
              QueueStorm Investigator
            </h1>
          </div>
          
          <div className="flex flex-wrap gap-2">
            <span className="font-mono text-[11px] self-center text-[var(--sea-ink-soft)] mr-1">Load Case:</span>
            <Button variant="outline" size="xs" onClick={() => loadSample("wrong_transfer")}>
              Wrong Transfer
            </Button>
            <Button variant="outline" size="xs" onClick={() => loadSample("failed_payment")}>
              Failed Payment
            </Button>
            <Button variant="outline" size="xs" onClick={() => loadSample("phishing")}>
              Phishing Report
            </Button>
          </div>
        </div>
      </section>

      {/* Main Grid Workspace */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-12 items-start">
        
        {/* LEFT CARD: Submission Form */}
        <section className="island-shell p-6 rounded-lg border border-[var(--line)] transition-all">
          <div className="flex items-center gap-2 mb-6 border-b border-[var(--line)] pb-3">
            <FileText className="size-4 text-[var(--sea-ink)]" />
            <h3 className="m-0 text-sm font-bold uppercase tracking-wider text-[var(--sea-ink)]">
              Ticket Submission Panel
            </h3>
          </div>

          <form onSubmit={handleSubmit} className="space-y-5">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label className="block text-xs font-mono uppercase tracking-wider text-[var(--sea-ink-soft)] mb-1">
                  Ticket ID
                </label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    required
                    value={ticketId}
                    onChange={e => setTicketId(e.target.value)}
                    className="w-full h-9 px-3 text-xs font-mono border border-[var(--line)] bg-[var(--foam)] text-[var(--sea-ink)] rounded focus:outline-none focus:border-black dark:focus:border-white transition-all"
                  />
                  <Button type="button" variant="outline" size="icon-sm" onClick={generateRandomId} title="Regenerate ID">
                    <RefreshCw className="size-3.5" />
                  </Button>
                </div>
              </div>

              <div>
                <label className="block text-xs font-mono uppercase tracking-wider text-[var(--sea-ink-soft)] mb-1">
                  Campaign Context
                </label>
                <input
                  type="text"
                  value={campaignContext}
                  onChange={e => setCampaignContext(e.target.value)}
                  placeholder="e.g. boishakh_bonanza"
                  className="w-full h-9 px-3 text-xs font-mono border border-[var(--line)] bg-[var(--foam)] text-[var(--sea-ink)] rounded focus:outline-none focus:border-black dark:focus:border-white transition-all"
                />
              </div>

              <div>
                <label className="block text-xs font-mono uppercase tracking-wider text-[var(--sea-ink-soft)] mb-1">
                  User Type
                </label>
                <select
                  value={userType}
                  onChange={e => setUserType(e.target.value)}
                  className="w-full h-9 px-2 text-xs font-mono border border-[var(--line)] bg-[var(--foam)] text-[var(--sea-ink)] rounded focus:outline-none focus:border-black dark:focus:border-white transition-all"
                >
                  <option value="customer">customer</option>
                  <option value="merchant">merchant</option>
                  <option value="agent">agent</option>
                  <option value="unknown">unknown</option>
                </select>
              </div>

              <div>
                <label className="block text-xs font-mono uppercase tracking-wider text-[var(--sea-ink-soft)] mb-1">
                  Submit Channel
                </label>
                <select
                  value={channel}
                  onChange={e => setChannel(e.target.value)}
                  className="w-full h-9 px-2 text-xs font-mono border border-[var(--line)] bg-[var(--foam)] text-[var(--sea-ink)] rounded focus:outline-none focus:border-black dark:focus:border-white transition-all"
                >
                  <option value="in_app_chat">in_app_chat</option>
                  <option value="call_center">call_center</option>
                  <option value="email">email</option>
                  <option value="merchant_portal">merchant_portal</option>
                  <option value="field_agent">field_agent</option>
                </select>
              </div>
            </div>

            <div>
              <label className="block text-xs font-mono uppercase tracking-wider text-[var(--sea-ink-soft)] mb-1">
                Complaint Details
              </label>
              <textarea
                required
                value={complaint}
                onChange={e => setComplaint(e.target.value)}
                placeholder="Enter customer support ticket details here (supports English, Bangla, and Banglish)..."
                rows={4}
                className="w-full p-3 text-xs border border-[var(--line)] bg-[var(--foam)] text-[var(--sea-ink)] rounded focus:outline-none focus:border-black dark:focus:border-white transition-all resize-y"
              />
            </div>

            <div className="border-t border-[var(--line)] pt-4">
              <div className="flex items-center justify-between mb-3">
                <label className="text-xs font-mono uppercase tracking-wider text-[var(--sea-ink-soft)]">
                  Customer Transaction History ({transactions.length})
                </label>
                <Button type="button" variant="outline" size="xs" onClick={addTransaction}>
                  <Plus className="size-3.5 mr-1" /> Add Txn
                </Button>
              </div>

              {transactions.length === 0 ? (
                <div className="border border-dashed border-[var(--line)] p-4 text-center rounded text-xs font-mono text-[var(--sea-ink-soft)] bg-[var(--chip-bg)]/20">
                  No transactions added. Analysis will fall back to safety-checks and rules evaluating text only.
                </div>
              ) : (
                <div className="space-y-3 max-h-[220px] overflow-y-auto pr-1">
                  {transactions.map((txn, index) => (
                    <div key={index} className="p-3 border border-[var(--line)] bg-[var(--chip-bg)]/30 rounded relative flex flex-col gap-2">
                      <div className="flex items-center gap-2 justify-between">
                        <input
                          type="text"
                          value={txn.transaction_id}
                          onChange={e => updateTransaction(index, "transaction_id", e.target.value)}
                          placeholder="Txn ID (e.g. TXN-123)"
                          className="h-7 w-[120px] px-2 text-[11px] font-mono border border-[var(--line)] bg-[var(--foam)] rounded"
                        />
                        <select
                          value={txn.transaction_type}
                          onChange={e => updateTransaction(index, "transaction_type", e.target.value)}
                          className="h-7 px-1 text-[11px] font-mono border border-[var(--line)] bg-[var(--foam)] rounded"
                        >
                          <option value="transfer">transfer</option>
                          <option value="payment">payment</option>
                          <option value="cash_in">cash_in</option>
                          <option value="cash_out">cash_out</option>
                          <option value="settlement">settlement</option>
                          <option value="refund">refund</option>
                        </select>
                        <button
                          type="button"
                          onClick={() => removeTransaction(index)}
                          className="text-red-500 hover:text-red-600 transition"
                        >
                          <Trash2 className="size-4" />
                        </button>
                      </div>
                      
                      <div className="grid grid-cols-3 gap-2">
                        <input
                          type="number"
                          value={txn.amount || ""}
                          onChange={e => updateTransaction(index, "amount", parseFloat(e.target.value) || 0)}
                          placeholder="Amount"
                          className="h-7 px-2 text-[11px] font-mono border border-[var(--line)] bg-[var(--foam)] rounded w-full"
                        />
                        <input
                          type="text"
                          value={txn.counterparty}
                          onChange={e => updateTransaction(index, "counterparty", e.target.value)}
                          placeholder="Counterparty"
                          className="h-7 px-2 text-[11px] font-mono border border-[var(--line)] bg-[var(--foam)] rounded w-full"
                        />
                        <select
                          value={txn.status}
                          onChange={e => updateTransaction(index, "status", e.target.value)}
                          className="h-7 px-1 text-[11px] font-mono border border-[var(--line)] bg-[var(--foam)] rounded w-full"
                        >
                          <option value="completed">completed</option>
                          <option value="failed">failed</option>
                          <option value="pending">pending</option>
                          <option value="reversed">reversed</option>
                        </select>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {error && (
              <div className="p-3 border border-red-500/20 bg-red-500/5 text-red-500 rounded text-xs flex items-center gap-2">
                <AlertCircle className="size-4 shrink-0" />
                <span>{error}</span>
              </div>
            )}

            <div className="border-t border-[var(--line)] pt-4 flex gap-3">
              <Button type="submit" disabled={isLoading} className="flex-1">
                {isLoading ? (
                  <>
                    <Loader2 className="size-4 animate-spin mr-1" />
                    Analyzing Ticket...
                  </>
                ) : (
                  "Run Investigation Pipeline"
                )}
              </Button>
            </div>
          </form>
        </section>

        {/* RIGHT CARD: Copilot Verdict & Analysis Output */}
        <section className="island-shell p-6 rounded-lg border border-[var(--line)] min-h-[500px] flex flex-col transition-all">
          <div className="flex items-center justify-between mb-6 border-b border-[var(--line)] pb-3">
            <div className="flex items-center gap-2">
              <Layers className="size-4 text-[var(--sea-ink)]" />
              <h3 className="m-0 text-sm font-bold uppercase tracking-wider text-[var(--sea-ink)]">
                Copilot Investigator Output
              </h3>
            </div>
            {result && (
              <span className="font-mono text-xs text-[var(--sea-ink-soft)] bg-[var(--chip-bg)] px-2 py-0.5 border border-[var(--chip-line)] rounded">
                confidence: {result.confidence ?? "0.90"}
              </span>
            )}
          </div>

          {!result ? (
            <div className="flex-1 flex flex-col items-center justify-center text-center p-8 border border-dashed border-[var(--line)] bg-[var(--chip-bg)]/10 rounded">
              <ShieldAlert className="size-8 text-[var(--sea-ink-soft)] mb-3 animate-pulse" />
              <p className="text-xs font-mono text-[var(--sea-ink-soft)] max-w-sm m-0">
                Submit a support ticket on the left panel. The copilot will cross-examine the text history and render evidence findings instantly.
              </p>
            </div>
          ) : (
            <div className="space-y-5 text-left flex-1 flex flex-col">
              
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
                <div className="p-2.5 border rounded border-[var(--line)] bg-[var(--foam)] text-center">
                  <span className="block text-[9px] font-mono uppercase tracking-widest text-[var(--sea-ink-soft)] mb-1">
                    Evidence Verdict
                  </span>
                  <span className={`inline-block px-2 py-0.5 border text-[10px] font-bold uppercase rounded ${getVerdictStyle(result.evidence_verdict)}`}>
                    {result.evidence_verdict ?? "insufficient_data"}
                  </span>
                </div>

                <div className="p-2.5 border rounded border-[var(--line)] bg-[var(--foam)] text-center">
                  <span className="block text-[9px] font-mono uppercase tracking-widest text-[var(--sea-ink-soft)] mb-1">
                    Severity
                  </span>
                  <span className={`inline-block px-2 py-0.5 border text-[10px] font-bold uppercase rounded ${getSeverityStyle(result.severity)}`}>
                    {result.severity ?? "low"}
                  </span>
                </div>

                <div className="p-2.5 border rounded border-[var(--line)] bg-[var(--foam)] text-center">
                  <span className="block text-[9px] font-mono uppercase tracking-widest text-[var(--sea-ink-soft)] mb-1">
                    Case Type
                  </span>
                  <span className="text-[11px] font-mono font-bold tracking-tight text-[var(--sea-ink)]">
                    {result.case_type ?? "other"}
                  </span>
                </div>

                <div className="p-2.5 border rounded border-[var(--line)] bg-[var(--foam)] text-center">
                  <span className="block text-[9px] font-mono uppercase tracking-widest text-[var(--sea-ink-soft)] mb-1">
                    Routed Dept.
                  </span>
                  <span className="text-[11px] font-mono font-bold tracking-tight text-[var(--sea-ink)]">
                    {result.department ?? "customer_support"}
                  </span>
                </div>
              </div>

              <div className="grid grid-cols-2 gap-4 p-3 border border-[var(--line)] bg-[var(--chip-bg)]/20 rounded font-mono text-[11px]">
                <div className="flex flex-col">
                  <span className="text-[var(--sea-ink-soft)] uppercase text-[9px] tracking-wider mb-0.5">Matched Transaction</span>
                  <span className="font-bold text-[var(--sea-ink)]">
                    {result.relevant_transaction_id ? (
                      <code className="text-xs bg-[var(--chip-bg)] border border-[var(--line)]">{result.relevant_transaction_id}</code>
                    ) : (
                      "None"
                    )}
                  </span>
                </div>
                <div className="flex flex-col">
                  <span className="text-[var(--sea-ink-soft)] uppercase text-[9px] tracking-wider mb-0.5">Human Review Required</span>
                  <span className={`font-bold ${result.human_review_required ? "text-amber-500" : "text-[var(--sea-ink-soft)]"}`}>
                    {result.human_review_required ? "YES (Escalated)" : "NO (Auto-resolved)"}
                  </span>
                </div>
              </div>

              <div className="space-y-4">
                <div>
                  <label className="block text-xs font-mono uppercase tracking-wider text-[var(--sea-ink-soft)] mb-1">
                    Agent Summary
                  </label>
                  <p className="m-0 text-xs text-[var(--sea-ink)] bg-[var(--foam)] p-3 border border-[var(--line)] rounded leading-relaxed">
                    {result.agent_summary}
                  </p>
                </div>

                <div>
                  <label className="block text-xs font-mono uppercase tracking-wider text-[var(--sea-ink-soft)] mb-1">
                    Recommended Next Action
                  </label>
                  <p className="m-0 text-xs text-[var(--sea-ink)] bg-[var(--foam)] p-3 border border-[var(--line)] rounded leading-relaxed">
                    {result.recommended_next_action}
                  </p>
                </div>
              </div>

              <div className="flex-1 flex flex-col min-h-[140px] pt-2">
                <div className="flex items-center justify-between mb-1.5">
                  <label className="text-xs font-mono uppercase tracking-wider text-[var(--sea-ink-soft)]">
                    Generated Customer Reply Draft
                  </label>
                  <Button
                    type="button"
                    variant="outline"
                    size="xs"
                    onClick={() => copyToClipboard(result.customer_reply)}
                    className="h-7 gap-1"
                  >
                    {copied ? (
                      <>
                        <Check className="size-3 text-emerald-500" />
                        Copied
                      </>
                    ) : (
                      <>
                        <Clipboard className="size-3" />
                        Copy Draft
                      </>
                    )}
                  </Button>
                </div>
                <div className="flex-1 text-xs text-[var(--sea-ink)] bg-[var(--foam)] p-3 border border-[var(--line)] rounded leading-relaxed font-sans border-l-4 border-l-black dark:border-l-white bg-linear-to-r from-zinc-500/5 to-transparent">
                  {result.customer_reply}
                </div>
              </div>

            </div>
          )}
        </section>

      </div>

      {/* ADMIN LOG: Historical Database Grid */}
      <section className="island-shell p-6 rounded-lg border border-[var(--line)] transition-all">
        <div className="flex items-center justify-between mb-6 border-b border-[var(--line)] pb-3">
          <div className="flex items-center gap-2">
            <Database className="size-4 text-[var(--sea-ink)]" />
            <h3 className="m-0 text-sm font-bold uppercase tracking-wider text-[var(--sea-ink)]">
              Historical Ticket Investigation Database Logs
            </h3>
          </div>
          <span className="font-mono text-xs text-[var(--sea-ink-soft)]">
            Total records: {historyTotal}
          </span>
        </div>

        <div className="overflow-x-auto border border-[var(--line)] rounded bg-[var(--foam)]">
          <table className="w-full border-collapse text-left text-xs font-mono">
            <thead>
              <tr className="border-b border-[var(--line)] bg-[var(--chip-bg)]/50 text-[var(--sea-ink-soft)]">
                <th className="p-3 font-semibold uppercase tracking-wider">Ticket ID</th>
                <th className="p-3 font-semibold uppercase tracking-wider max-w-[200px] truncate">Complaint</th>
                <th className="p-3 font-semibold uppercase tracking-wider text-center">Verdict</th>
                <th className="p-3 font-semibold uppercase tracking-wider text-center">Case Type</th>
                <th className="p-3 font-semibold uppercase tracking-wider text-center">Severity</th>
                <th className="p-3 font-semibold uppercase tracking-wider">Department</th>
                <th className="p-3 font-semibold uppercase tracking-wider text-right">Created At</th>
              </tr>
            </thead>
            <tbody>
              {isHistoryLoading ? (
                <tr>
                  <td colSpan={7} className="p-8 text-center text-[var(--sea-ink-soft)]">
                    <div className="flex justify-center items-center gap-2">
                      <Loader2 className="size-4 animate-spin" />
                      Fetching tickets logs...
                    </div>
                  </td>
                </tr>
              ) : history.length === 0 ? (
                <tr>
                  <td colSpan={7} className="p-8 text-center text-[var(--sea-ink-soft)]">
                    No analyzed tickets found in the database. Tickets analyzed using "Run Investigation Pipeline" will be saved and displayed here.
                  </td>
                </tr>
              ) : (
                history.map(ticket => (
                  <tr
                    key={ticket.ticket_id}
                    onClick={() => loadFromHistory(ticket)}
                    className="border-b border-[var(--line)] hover:bg-[var(--chip-bg)]/40 cursor-pointer transition-all"
                  >
                    <td className="p-3 font-bold text-[var(--sea-ink)]">{ticket.ticket_id}</td>
                    <td className="p-3 max-w-[200px] truncate font-sans text-xs" title={ticket.complaint}>
                      {ticket.complaint}
                    </td>
                    <td className="p-3 text-center">
                      <span className={`inline-block px-1.5 py-0.5 border text-[9px] font-bold uppercase rounded ${getVerdictStyle(ticket.evidence_verdict)}`}>
                        {ticket.evidence_verdict}
                      </span>
                    </td>
                    <td className="p-3 text-center font-bold text-[var(--sea-ink)]">{ticket.case_type}</td>
                    <td className="p-3 text-center">
                      <span className={`inline-block px-1.5 py-0.5 border text-[9px] font-bold uppercase rounded ${getSeverityStyle(ticket.severity)}`}>
                        {ticket.severity}
                      </span>
                    </td>
                    <td className="p-3 text-[var(--sea-ink-soft)]">{ticket.department}</td>
                    <td className="p-3 text-right text-[var(--sea-ink-soft)]">
                      {new Date(ticket.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        {historyTotal > historyLimit && (
          <div className="flex justify-between items-center mt-4 pt-3 border-t border-[var(--line)]">
            <span className="text-xs font-mono text-[var(--sea-ink-soft)]">
              Showing {historyOffset + 1} - {Math.min(historyOffset + historyLimit, historyTotal)} of {historyTotal}
            </span>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                disabled={historyOffset === 0}
                onClick={() => {
                  const nextOffset = Math.max(0, historyOffset - historyLimit);
                  setHistoryOffset(nextOffset);
                  fetchHistory(nextOffset);
                }}
              >
                Previous
              </Button>
              <Button
                variant="outline"
                size="sm"
                disabled={historyOffset + historyLimit >= historyTotal}
                onClick={() => {
                  const nextOffset = historyOffset + historyLimit;
                  setHistoryOffset(nextOffset);
                  fetchHistory(nextOffset);
                }}
              >
                Next
              </Button>
            </div>
          </div>
        )}
      </section>

    </main>
  );
}
