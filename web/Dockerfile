FROM docker.io/oven/bun:1-slim AS builder

# Accept build arguments for environment variables
ARG PUBLIC_API_URL
ARG PUBLIC_JUDGE_URL

WORKDIR /app
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile --production
COPY . .

# Set environment variables for the build
ENV PUBLIC_API_URL=${PUBLIC_API_URL}
ENV PUBLIC_JUDGE_URL=${PUBLIC_JUDGE_URL}

RUN bun run build

FROM nginx:alpine

COPY --from=builder /app/build /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
