.PHONY : install

install:
	cargo build --release
	sudo cp ./target/release/ssh-container /opt/ssh-container
	sudo mkdir -p /etc/ssh-container
	sudo cp ssh-container.conf /etc/ssh-container