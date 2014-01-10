default:
	cd src && rustc --rlib --opt-level=3 lib.rs

test:
	cd src && \
	rustc --test --opt-level=3 lib.rs && \
	./route_recognizer

