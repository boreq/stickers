.PHONY: update
update:
	rsync -a --progress --delete --exclude node_modules --exclude extractor-rust/target ./ server:/home/filip/server/docker/stickers/repo/
	ssh -t server 'cd /home/filip/server/docker/stickers; make update; ~/scripts/clear_nginx_cache'
