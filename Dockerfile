# Build stage for frontend
FROM node:20-alpine AS frontend-builder

WORKDIR /app

COPY frontend/package*.json ./
RUN npm ci

COPY frontend/ .
RUN npm run build

# Build stage for backend
FROM rust:1-slim AS backend-builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY backend/Cargo.toml backend/
COPY backend/src ./src

# Build without running to cache dependencies
RUN cargo build --release 2>&1

# Copy source and build again (better caching)
COPY backend/ .

RUN cargo build --release

# Runtime stage - serve both frontend and backend
FROM node:20-alpine AS runner

WORKDIR /app

# Copy built frontend
COPY --from=frontend-builder /app/dist ./dist
COPY --from=frontend-builder /app/index.html ./

# Copy built backend binary
COPY --from=backend-builder /app/target/release/secutest-backend ./

EXPOSE 8080
EXPOSE 5173

ENV PORT=8080
ENV NODE_ENV=production

# Start both services
CMD sh -c '
  # Start the backend
  ./secutest-backend &

  # Wait a moment for backend to start
  sleep 2

  # Start the frontend dev server in background (or serve static)
  npx vite --host --port 5173
'