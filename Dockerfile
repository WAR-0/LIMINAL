# LIMINAL Docker Configuration
# Multi-stage build for optimized image size

# Stage 1: CUDA base with Python
FROM nvidia/cuda:11.8.0-cudnn8-runtime-ubuntu22.04 AS base

# Set environment variables
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    CUDA_VISIBLE_DEVICES=0 \
    DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    python3.10 \
    python3.10-dev \
    python3-pip \
    git \
    wget \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Create working directory
WORKDIR /app

# Stage 2: Dependencies installation
FROM base AS dependencies

# Copy requirements first for better caching
COPY requirements.txt .

# Upgrade pip and install Python dependencies
RUN python3 -m pip install --upgrade pip && \
    python3 -m pip install --no-cache-dir -r requirements.txt

# Stage 3: Application
FROM dependencies AS application

# Copy application code
COPY liminal/ ./liminal/
COPY tests/ ./tests/
COPY scripts/ ./scripts/
COPY config/ ./config/
COPY setup.py pytest.ini ./

# Install the package in development mode
RUN python3 -m pip install -e .

# Create directories for data and models
RUN mkdir -p /app/data /app/models /app/logs

# Stage 4: Runtime
FROM application AS runtime

# Create non-root user for security
RUN useradd -m -u 1000 liminal && \
    chown -R liminal:liminal /app

USER liminal

# Expose ports for monitoring dashboard and API
EXPOSE 5000 8000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD python3 -c "import liminal; print('healthy')" || exit 1

# Default command (can be overridden)
CMD ["python3", "-m", "liminal.main", "--config", "config/default.yaml"]

# Stage 5: Development (includes additional tools)
FROM application AS development

USER root

# Install development tools
RUN python3 -m pip install --no-cache-dir \
    ipython \
    jupyter \
    jupyterlab \
    notebook

# Switch back to non-root user
USER liminal

# Expose Jupyter port
EXPOSE 8888

# Alternative command for development
CMD ["jupyter", "lab", "--ip=0.0.0.0", "--port=8888", "--no-browser", "--allow-root"]