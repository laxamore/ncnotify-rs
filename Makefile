build:
	cargo build --release

install:
	install -m 755 target/release/ncnotify-rs /usr/local/bin/ncnotify
	install -m 644 etc/systemd/ncnotify.service /etc/systemd/system/
	mkdir -p /etc/ncnotify/
	install -m 644 etc/ncnotify/config.toml /etc/ncnotify/

uninstall:
	rm -f /usr/local/bin/ncnotify \
	  /etc/systemd/system/ncnotify.service \
	  /etc/ncnotify/config.toml; \
	rmdir /etc/ncnotify/; \
	exit 0
