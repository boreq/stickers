FROM archlinux AS build

RUN pacman -Syyu --noconfirm
RUN pacman -S base-devel rustup nvm yarn imagemagick --noconfirm
RUN rustup default nightly
RUN . /usr/share/nvm/init-nvm.sh && nvm install 22.15.0 && nvm use 22.15.0

COPY . /stickers
WORKDIR /stickers

RUN ./extract_stickers.sh
RUN . /usr/share/nvm/init-nvm.sh && yarn install --frozen-lockfile && yarn build --release

FROM nginx

COPY <<EOF /etc/nginx/nginx.conf
user  nginx;
worker_processes  auto;

error_log  /var/log/nginx/error.log notice;
pid        /run/nginx.pid;


events {
    worker_connections  1024;
}


http {
    include       /etc/nginx/mime.types;
    default_type  application/octet-stream;

    log_format  main  '$remote_addr - $remote_user [$time_local] "$request" '
                      '$status $body_bytes_sent "$http_referer" '
                      '"$http_user_agent" "$http_x_forwarded_for"';

    access_log  /var/log/nginx/access.log  main;

    sendfile        on;
    #tcp_nopush     on;

    keepalive_timeout  65;

    #gzip  on;

    include /etc/nginx/conf.d/*.conf;

    server {
        listen       80;
        server_name  localhost;

        location / {
            root /usr/share/nginx/html/;
            try_files $uri $uri/ /index.html;
        }
    }
}
EOF

COPY --from=build /stickers/dist /usr/share/nginx/html
