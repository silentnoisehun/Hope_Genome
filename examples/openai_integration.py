"""
OpenAI API Integration Example
================================

Demonstrates Hope Genome integration with OpenAI API for
accountable LLM interactions with function calling.

Install:
    pip install hope-genome openai

Run:
    export OPENAI_API_KEY=sk-...
    python openai_integration.py
"""

import openai
import hope_genome as hg
import json
import os
from typing import List, Dict, Any


# Initialize Hope Genome
genome = hg.SealedGenome(rules=[
    "Do no harm",
    "Respect user privacy",
    "Ensure data security",
    "Follow GDPR regulations"
])
genome.seal()

auditor = hg.ProofAuditor()
logger = hg.AuditLogger("/var/log/hope-genome/openai-audit.log")


# Define accountable functions
def get_user_data(user_id: str) -> Dict[str, Any]:
    """Get user data (requires proof)"""
    # Create Hope Genome action
    action = hg.Action.custom(
        action_type="get_user_data",
        target=user_id,
        metadata='{"operation": "read", "pii": true}'
    )

    # Get cryptographic proof
    proof = genome.verify_action(action)

    if not proof.approved:
        return {
            "error": "Access denied by genome",
            "reason": proof.denial_reason()
        }

    # Verify and audit proof
    auditor.verify_proof(proof)
    logger.log_proof(proof, action_description=f"Read user data: {user_id}")

    # Execute (simulated)
    return {
        "user_id": user_id,
        "name": "John Doe",
        "email": "john@example.com",
        "proof_signature": proof.signature_hex()[:16] + "...",
        "audit_entry": logger.entry_count()
    }


def delete_user_account(user_id: str, reason: str) -> Dict[str, Any]:
    """Delete user account (GDPR compliance, requires proof)"""
    # Create Hope Genome action
    action = hg.Action.delete_user(user_id)

    # Get cryptographic proof
    proof = genome.verify_action(action)

    if not proof.approved:
        return {
            "error": "Deletion denied by genome",
            "reason": proof.denial_reason()
        }

    # Verify and audit proof
    auditor.verify_proof(proof)
    logger.log_proof(
        proof,
        action_description=f"GDPR deletion: {user_id} (reason: {reason})"
    )

    # Execute (simulated)
    return {
        "status": "deleted",
        "user_id": user_id,
        "reason": reason,
        "proof_signature": proof.signature_hex()[:16] + "...",
        "audit_entry": logger.entry_count(),
        "compliance": "GDPR Article 17 compliant"
    }


def send_notification(user_id: str, message: str) -> Dict[str, Any]:
    """Send notification to user (requires proof)"""
    # Create Hope Genome action
    action = hg.Action.custom(
        action_type="send_notification",
        target=user_id,
        metadata=json.dumps({"message": message[:50]})
    )

    # Get cryptographic proof
    proof = genome.verify_action(action)

    if not proof.approved:
        return {
            "error": "Notification denied by genome",
            "reason": proof.denial_reason()
        }

    # Verify and audit proof
    auditor.verify_proof(proof)
    logger.log_proof(proof, action_description=f"Notification sent to: {user_id}")

    # Execute (simulated)
    return {
        "status": "sent",
        "user_id": user_id,
        "message": message,
        "proof_signature": proof.signature_hex()[:16] + "...",
        "audit_entry": logger.entry_count()
    }


# Map function names to actual functions
FUNCTIONS_MAP = {
    "get_user_data": get_user_data,
    "delete_user_account": delete_user_account,
    "send_notification": send_notification,
}

# Define OpenAI function schemas
FUNCTIONS_SCHEMA = [
    {
        "name": "get_user_data",
        "description": "Retrieve user data from the system. All accesses are cryptographically logged.",
        "parameters": {
            "type": "object",
            "properties": {
                "user_id": {
                    "type": "string",
                    "description": "The user ID to retrieve data for"
                }
            },
            "required": ["user_id"]
        }
    },
    {
        "name": "delete_user_account",
        "description": "Delete a user account for GDPR compliance. Requires cryptographic proof and audit logging.",
        "parameters": {
            "type": "object",
            "properties": {
                "user_id": {
                    "type": "string",
                    "description": "The user ID to delete"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for deletion (e.g., 'GDPR Article 17 request')"
                }
            },
            "required": ["user_id", "reason"]
        }
    },
    {
        "name": "send_notification",
        "description": "Send a notification to a user. Cryptographically audited.",
        "parameters": {
            "type": "object",
            "properties": {
                "user_id": {
                    "type": "string",
                    "description": "The user ID to send notification to"
                },
                "message": {
                    "type": "string",
                    "description": "The notification message"
                }
            },
            "required": ["user_id", "message"]
        }
    }
]


def run_accountable_openai_agent(user_query: str):
    """
    Run OpenAI agent with Hope Genome accountability.

    All function calls are cryptographically proven and audited.
    """
    client = openai.OpenAI(api_key=os.getenv("OPENAI_API_KEY"))

    messages = [
        {
            "role": "system",
            "content": """You are an AI assistant with built-in accountability via Hope Genome.

Every action you take (data access, deletions, notifications) requires a cryptographic proof
that is verified and logged to a tamper-evident blockchain-style audit trail.

When users request GDPR data deletion, use the delete_user_account function.
All user data access is logged with Ed25519 signatures."""
        },
        {
            "role": "user",
            "content": user_query
        }
    ]

    print(f"\n{'='*70}")
    print(f"User Query: {user_query}")
    print(f"{'='*70}\n")

    # First API call
    response = client.chat.completions.create(
        model="gpt-4",
        messages=messages,
        tools=[{"type": "function", "function": f} for f in FUNCTIONS_SCHEMA],
        tool_choice="auto"
    )

    # Process function calls
    while response.choices[0].finish_reason == "tool_calls":
        message = response.choices[0].message
        messages.append(message)

        for tool_call in message.tool_calls:
            function_name = tool_call.function.name
            function_args = json.loads(tool_call.function.arguments)

            print(f"🔧 Calling function: {function_name}")
            print(f"   Arguments: {function_args}")

            # Execute function with Hope Genome accountability
            function_response = FUNCTIONS_MAP[function_name](**function_args)

            print(f"   Response: {json.dumps(function_response, indent=2)}")

            # Add function response to conversation
            messages.append({
                "role": "tool",
                "tool_call_id": tool_call.id,
                "content": json.dumps(function_response)
            })

        # Continue conversation
        response = client.chat.completions.create(
            model="gpt-4",
            messages=messages,
            tools=[{"type": "function", "function": f} for f in FUNCTIONS_SCHEMA],
            tool_choice="auto"
        )

    # Final response
    final_response = response.choices[0].message.content

    print(f"\n{'='*70}")
    print(f"AI Response:\n{final_response}")
    print(f"{'='*70}")
    print(f"✅ Audit Trail Entries: {logger.entry_count()}")
    print(f"✅ Blockchain Integrity: {'VALID' if logger.verify_chain() else 'BROKEN'}")
    print(f"✅ Proofs Verified: {auditor.verified_count()}")
    print(f"{'='*70}\n")

    return final_response


if __name__ == "__main__":
    # Example queries
    queries = [
        "Get the data for user user_12345",
        "User user_67890 has requested account deletion under GDPR Article 17",
        "Send a notification to user user_11111 saying 'Your account has been verified'",
    ]

    for query in queries:
        try:
            run_accountable_openai_agent(query)
        except Exception as e:
            print(f"❌ Error: {e}\n")

    # Final audit report
    print("\n" + "="*70)
    print("FINAL AUDIT REPORT")
    print("="*70)
    print(f"Total function calls: {auditor.verified_count()}")
    print(f"Total audit entries: {logger.entry_count()}")
    print(f"Audit chain status: {'✅ VALID' if logger.verify_chain() else '❌ BROKEN'}")
    print(f"Genome hash: {genome.genome_hash()}")
    print("="*70)

    # Get all audit entries
    print("\nAUDIT TRAIL:")
    for entry in logger.get_entries():
        print(f"  [{entry.index}] {entry.timestamp} - {entry.action_description}")
    print("="*70)
