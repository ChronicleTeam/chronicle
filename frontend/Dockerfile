FROM node:24.8.0-alpine3.21 AS builder
WORKDIR /app
COPY package*.json .
RUN npm ci
COPY . .
RUN npm run build
RUN npm prune --production

FROM node:24.8.0-alpine3.21
WORKDIR /app
COPY --from=builder /app/build build/
COPY --from=builder /app/node_modules node_modules/
COPY package.json .
EXPOSE 3000
ENV NODE_ENV=production
CMD [ "node", "build" ]
