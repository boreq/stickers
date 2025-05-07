FROM archlinux AS build

RUN pacman -Syyu --noconfirm
RUN pacman -S base-devel rustup nvm yarn imagemagick --noconfirm
RUN rustup default nightly
RUN . /usr/share/nvm/init-nvm.sh && nvm install 23.9.0 && nvm use 23.9.0

COPY . /stickers
WORKDIR /stickers

RUN ./extract_stickers.sh
RUN . /usr/share/nvm/init-nvm.sh && yarn build --release

FROM nginx
COPY --from=build /stickers/dist /usr/share/nginx/html
