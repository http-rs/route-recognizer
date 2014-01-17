LIB_DIR = ./lib
BUILD_DIR = ./build

all: route-recognizer

route-recognizer:
	rustc -L $(LIB_DIR) --opt-level=3 src/route_recognizer/lib.rs --out-dir $(BUILD_DIR)

test:
	rustc --test --opt-level=3 src/route_recognizer/lib.rs --out-dir $(BUILD_DIR) && $(BUILD_DIR)/route_recognizer

clean:
	rm -rf $(BUILD_DIR)/*

.PHONY: all routed-http test clean