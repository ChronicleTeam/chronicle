FROM node:24.8.0-alpine3.21 AS builder
COPY package.json package-lock.json /app/
WORKDIR /app
RUN npm ci
COPY . .
RUN npm run build

FROM node:24.8.0-alpine3.21
COPY --from=builder /app/build /build
EXPOSE 3000
CMD ["node", "build"]

