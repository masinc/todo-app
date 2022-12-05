FROM node:lts-buster-slim AS builder
WORKDIR /frontend
COPY . /frontend
RUN npm install -g pnpm &&\
    pnpm install &&\
    pnpm build


FROM nginx:stable
COPY --from=builder /frontend/build/ /usr/share/nginx/html
