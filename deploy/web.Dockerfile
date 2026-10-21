FROM node:22-alpine AS builder
WORKDIR /app
COPY frontend/package*.json ./
RUN npm ci --registry=https://registry.npmjs.org
COPY frontend/ ./
RUN npm run build

FROM caddy:2-alpine
COPY --from=builder /app/dist /srv
COPY deploy/Caddyfile /etc/caddy/Caddyfile
