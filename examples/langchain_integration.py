"""
LangChain Integration Example
==============================

Demonstrates Hope Genome integration with LangChain for
accountable LLM agent operations.

Install:
    pip install hope-genome langchain langchain-openai

Run:
    export OPENAI_API_KEY=sk-...
    python langchain_integration.py
"""

from langchain.agents import AgentExecutor, create_openai_functions_agent
from langchain.tools import Tool
from langchain_openai import ChatOpenAI
from langchain.prompts import ChatPromptTemplate, MessagesPlaceholder
import hope_genome as hg
import os


# Initialize Hope Genome components
genome = hg.SealedGenome(rules=[
    "Do no harm to users",
    "Respect user privacy",
    "Provide accurate information",
    "Refuse harmful requests"
])
genome.seal()

auditor = hg.ProofAuditor()
logger = hg.AuditLogger("/var/log/hope-genome/langchain-audit.log")


def accountable_tool_wrapper(tool_func, tool_name: str):
    """
    Wraps a LangChain tool with Hope Genome accountability.

    All tool invocations require cryptographic proof and are audited.
    """
    def wrapped_tool(query: str) -> str:
        # Create action for this tool invocation
        action = hg.Action.custom(
            action_type=f"langchain_tool_{tool_name}",
            target=query,
            metadata=f'{{"tool": "{tool_name}"}}'
        )

        # Get cryptographic proof
        proof = genome.verify_action(action)

        if not proof.approved:
            return f"❌ Action denied by genome: {proof.denial_reason()}"

        # Verify proof
        try:
            auditor.verify_proof(proof)
        except hg.AuditorError as e:
            return f"❌ Proof verification failed: {e}"

        # Log to audit trail
        logger.log_proof(
            proof,
            action_description=f"LangChain tool '{tool_name}' invoked with: {query[:100]}"
        )

        # Execute actual tool
        result = tool_func(query)

        # Log result
        print(f"✅ Tool '{tool_name}' executed | Proof: {proof.nonce_hex()[:16]}...")

        return result

    return wrapped_tool


# Define tools with Hope Genome accountability
def search_database(query: str) -> str:
    """Search internal database"""
    return f"Database search results for: {query}"


def send_email(recipient_and_message: str) -> str:
    """Send email to user"""
    return f"Email sent: {recipient_and_message}"


def delete_user_data(user_id: str) -> str:
    """Delete user data (GDPR compliance)"""
    return f"User data deleted for: {user_id}"


# Wrap tools with accountability
tools = [
    Tool(
        name="search_database",
        func=accountable_tool_wrapper(search_database, "search_database"),
        description="Search internal database. Input should be a search query."
    ),
    Tool(
        name="send_email",
        func=accountable_tool_wrapper(send_email, "send_email"),
        description="Send email to user. Input should be 'recipient: message'."
    ),
    Tool(
        name="delete_user_data",
        func=accountable_tool_wrapper(delete_user_data, "delete_user_data"),
        description="Delete user data for GDPR compliance. Input should be user_id."
    ),
]

# Initialize LLM
llm = ChatOpenAI(
    model="gpt-4",
    temperature=0,
    openai_api_key=os.getenv("OPENAI_API_KEY")
)

# Create prompt
prompt = ChatPromptTemplate.from_messages([
    ("system", """You are a helpful AI assistant with accountability guarantees.

All your actions are cryptographically logged via Hope Genome.
Every tool invocation requires a cryptographic proof that is audited and logged.

When a tool fails verification, explain to the user that the action was denied
by the ethical genome rules."""),
    ("human", "{input}"),
    MessagesPlaceholder(variable_name="agent_scratchpad"),
])

# Create agent
agent = create_openai_functions_agent(llm, tools, prompt)
agent_executor = AgentExecutor(
    agent=agent,
    tools=tools,
    verbose=True,
    handle_parsing_errors=True
)


def run_accountable_agent(user_query: str):
    """
    Run LangChain agent with Hope Genome accountability.

    All tool invocations are cryptographically proven and audited.
    """
    print(f"\n{'='*60}")
    print(f"User Query: {user_query}")
    print(f"{'='*60}\n")

    result = agent_executor.invoke({"input": user_query})

    print(f"\n{'='*60}")
    print(f"Agent Response: {result['output']}")
    print(f"Audit Trail Entries: {logger.entry_count()}")
    print(f"Chain Valid: {logger.verify_chain()}")
    print(f"{'='*60}\n")

    return result


if __name__ == "__main__":
    # Example queries
    queries = [
        "Search the database for user John Doe",
        "Send an email to support@example.com saying 'Hello from accountable AI'",
        "Delete user data for user_12345 (GDPR request)",
    ]

    for query in queries:
        try:
            run_accountable_agent(query)
        except Exception as e:
            print(f"Error: {e}")

    # Final audit report
    print("\n" + "="*60)
    print("FINAL AUDIT REPORT")
    print("="*60)
    print(f"Total proofs verified: {auditor.verified_count()}")
    print(f"Total audit entries: {logger.entry_count()}")
    print(f"Audit chain integrity: {'✅ VALID' if logger.verify_chain() else '❌ BROKEN'}")
    print("="*60)
