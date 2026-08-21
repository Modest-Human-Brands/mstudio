# Use Bun official image
FROM oven/bun:latest@sha256:5ff609364c049b54eb0ff560ec96319729a972078ef2c755d758f0c6ef89c2d6

# Set working directory
WORKDIR /app

# Copy everything
COPY . .

# Install dependencies
RUN bun install --frozen-lockfile

# Run the same steps as CI
CMD bash -c "\
  echo '⚙️ Build' && bun run build && \
  echo '🧹 Lint' && bun run lint || true && \
  echo '🧼 Format' && bun run format \
"