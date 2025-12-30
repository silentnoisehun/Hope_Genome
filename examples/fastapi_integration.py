"""
FastAPI Integration Example
============================

Demonstrates Hope Genome integration with FastAPI for
accountable REST API endpoints.

Install:
    pip install hope-genome fastapi uvicorn

Run:
    uvicorn fastapi_integration:app --reload
"""

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import hope_genome as hg

app = FastAPI(
    title="Hope Genome API",
    description="Accountable AI decision API with cryptographic proofs",
    version="1.5.0"
)

# Initialize genome and auditor
genome = hg.SealedGenome(rules=[
    "Do no harm",
    "Respect privacy",
    "Ensure fairness",
    "Maintain transparency"
])
genome.seal()

auditor = hg.ProofAuditor()
logger = hg.AuditLogger("/var/log/hope-genome/api-audit.log")


class ActionRequest(BaseModel):
    action_type: str
    target: str
    metadata: str | None = None


class ProofResponse(BaseModel):
    approved: bool
    signature: str
    nonce: str
    timestamp: str
    denial_reason: str | None = None


@app.get("/")
def read_root():
    return {
        "service": "Hope Genome Accountable API",
        "version": "1.5.0",
        "genome_sealed": genome.is_sealed(),
        "auditor_verified": auditor.verified_count()
    }


@app.post("/verify-action", response_model=ProofResponse)
def verify_action(request: ActionRequest):
    """
    Verify an action and return cryptographic proof.

    Example:
        POST /verify-action
        {
            "action_type": "delete_user",
            "target": "user_123"
        }
    """
    # Create action based on type
    if request.action_type == "delete_user":
        action = hg.Action.delete_user(request.target)
    elif request.action_type == "delete_file":
        action = hg.Action.delete_file(request.target)
    elif request.action_type == "execute_command":
        action = hg.Action.execute_command(request.target)
    else:
        action = hg.Action.custom(request.action_type, request.target, request.metadata)

    # Get cryptographic proof
    proof = genome.verify_action(action)

    # Audit the proof
    try:
        auditor.verify_proof(proof)
    except hg.AuditorError as e:
        raise HTTPException(status_code=400, detail=f"Proof verification failed: {e}")

    # Log to blockchain-style audit trail
    logger.log_proof(
        proof,
        action_description=f"{request.action_type} on {request.target}"
    )

    return ProofResponse(
        approved=proof.approved,
        signature=proof.signature_hex(),
        nonce=proof.nonce_hex(),
        timestamp=proof.timestamp_string(),
        denial_reason=proof.denial_reason()
    )


@app.get("/audit/verify-chain")
def verify_audit_chain():
    """Verify blockchain-style audit trail integrity"""
    try:
        is_valid = logger.verify_chain()
        return {
            "chain_valid": is_valid,
            "entry_count": logger.entry_count()
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Chain verification failed: {e}")


@app.get("/audit/entries")
def get_audit_entries(limit: int = 100):
    """Get recent audit log entries"""
    entries = logger.get_entries()[-limit:]  # Last N entries

    return {
        "total_entries": logger.entry_count(),
        "returned": len(entries),
        "entries": [
            {
                "index": entry.index,
                "timestamp": entry.timestamp,
                "action_description": entry.action_description,
                "approved": entry.proof.approved
            }
            for entry in entries
        ]
    }


@app.get("/genome/info")
def genome_info():
    """Get genome information"""
    diag = genome.diagnostics()
    return {
        "rules": genome.rules(),
        "sealed": genome.is_sealed(),
        "genome_hash": genome.genome_hash() if genome.is_sealed() else None,
        "diagnostics": diag
    }


@app.get("/health")
def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "genome_sealed": genome.is_sealed(),
        "audit_chain_valid": logger.verify_chain(),
        "verified_proofs": auditor.verified_count()
    }


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
