.PHONY: build, solution, clean

build: generator solution

clean:
	@cargo clean
	@rm -rf target

solution:
	@echo "No solution yet"

generator:
	@cd src/bin && cargo build --release --bin generator
