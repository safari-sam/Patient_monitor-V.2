# ============================================================================
# ML Service Dockerfile
# ============================================================================
# Python-based ML prediction service for activity classification
# ============================================================================

FROM python:3.11-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements first (for caching)
COPY requirements.txt .

# Install Python dependencies
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY . .

# Generate training data and train model
RUN python training_data/generate_data.py && \
    python train_model.py

# Expose port
EXPOSE 5001

# Environment variables
ENV ML_PORT=5001
ENV PYTHONUNBUFFERED=1

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:5001/health || exit 1

# Run the service
CMD ["python", "ml_service.py"]
