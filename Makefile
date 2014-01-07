default:
	cd src && rustc --rlib lib.rs

test:
	cd src && \
	rustc --test lib.rs && \
	./lib

