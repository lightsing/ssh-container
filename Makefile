.PHONY : install

install:
	cargo build --release
	sudo cp ./target/release/ssh-container /opt/ssh-container