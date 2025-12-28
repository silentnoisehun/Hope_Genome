FROM python:3.9-slim

# Install required Python packages
RUN pip install --no-cache-dir requests

# Copy the mock LLM script
COPY main.py /app/main.py

WORKDIR /app

# Run the "malicious" AI
CMD ["python", "main.py"]
