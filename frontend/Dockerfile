FROM nginx:alpine-slim AS runner

COPY ./dist/* /usr/share/nginx/html/
COPY ./nginx.conf /etc/nginx/conf.d/default.conf
